use crate::context::Context;
use crate::model::error::FormatError;
use crate::model::id::Id;
use crate::model::token::Claims;
use crate::model::token::{
  CreateTokenByPasswordInput, CreateTokenOutput, RefreshToken, ACCESS_TOKEN_EXP,
};
use crate::model::user::{FindOneUserCondition, User};
use crate::repository::refresh_token_repo::RefreshTokenRepo;
use crate::repository::user_repo::UserRepo;
use anyhow::{bail, Result};
use config::Config;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn create_user_claims(ctx: &Context, user: &User, expires_in: usize) -> Result<Claims> {
  let aud = ctx.settings.get::<String>("aud").expect("aud must set");
  let unix_time = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .map_err(|err| FormatError::ServerError(err.to_string()))?;

  let unix_time_secs = unix_time.as_secs();
  let exp = unix_time_secs as usize + expires_in;

  Ok(Claims::new(user.id.clone(), aud, exp))
}

pub fn create_access_token(ctx: &Context, claims: &Claims) -> String {
  let private_key = ctx
    .settings
    .get::<String>("private_key")
    .expect("private_key must set");
  let encoding_key = &EncodingKey::from_rsa_pem(private_key.as_bytes()).unwrap();

  let header = Header::new(Algorithm::RS256);
  let token = encode::<Claims>(&header, claims, &encoding_key).unwrap();

  token
}

pub fn decode_access_token(settings: &Config, jwt: &String) -> Result<Claims> {
  let public_key = settings
    .get::<String>("public_key")
    .expect("public_key must set");
  let aud = settings.get::<String>("aud").expect("aud must set");
  let decode_key = &DecodingKey::from_rsa_pem(public_key.as_bytes()).unwrap();
  let mut validation = Validation::new(Algorithm::RS256);
  validation.set_audience(&[aud]);

  let token = decode::<Claims>(jwt, decode_key, &validation)
    .map_err(|err| FormatError::Unauthenticated(format!("invalid jwt: {:?}", err.to_string())))?;

  Ok(token.claims)
}

// TODO: hash token before save
pub async fn create_token_output<K: RefreshTokenRepo>(
  ctx: &Context,
  token_repo: &K,
  user: &User,
  swap_refresh_token: Option<String>,
) -> Result<CreateTokenOutput> {
  let claims = create_user_claims(ctx, user, ACCESS_TOKEN_EXP)?;
  let access_token = create_access_token(ctx, &claims);
  let refresh_token = RefreshToken::new(&user.id);

  if swap_refresh_token.is_some() {
    token_repo
      .swap_token(&user.id, swap_refresh_token.unwrap(), &refresh_token)
      .await?;
  } else {
    token_repo.insert(&refresh_token).await?;
  }

  Ok(CreateTokenOutput {
    access_token,
    refresh_token: refresh_token.token,
    token_type: "bearer".to_string(),
    expires_in: ACCESS_TOKEN_EXP as i32,
  })
}

pub async fn auth_by_password<T: UserRepo, K: RefreshTokenRepo>(
  ctx: &Context,
  user_repo: T,
  token_repo: K,
  sign_in_input: &CreateTokenByPasswordInput,
) -> Result<CreateTokenOutput> {
  let email = sign_in_input.email.clone();
  let password = sign_in_input.password.clone();

  let filter = FindOneUserCondition::Email(email);
  let user = user_repo.find_one(&filter).await?;

  // TODO: edit 404 error
  user.check_password(&password)?;
  if !user.is_confirmed() {
    bail!(FormatError::Unauthenticated(
      "Your account is not confirmed yet".to_string()
    ));
  }

  create_token_output(ctx, &token_repo, &user, None).await
}

pub async fn auth_by_refresh_token<T: UserRepo, K: RefreshTokenRepo>(
  ctx: &Context,
  user_repo: T,
  token_repo: K,
  refresh_token_value: String,
) -> Result<CreateTokenOutput> {
  let refresh_token = token_repo.find_one_by_token(refresh_token_value).await?;
  if refresh_token.is_revoked() {
    bail!(FormatError::Unauthenticated(
      "refresh token is revoked".to_string()
    ));
  }
  let find_one_user_condition = FindOneUserCondition::Id(refresh_token.user_id);
  let user = user_repo.find_one(&find_one_user_condition).await?;

  create_token_output(ctx, &token_repo, &user, Some(refresh_token.token)).await
}

pub async fn logout<T: RefreshTokenRepo>(
  _ctx: &Context,
  token_repo: &T,
  user_id: &Id,
) -> Result<()> {
  token_repo.delete_by_user_id(user_id).await?;

  Ok(())
}
