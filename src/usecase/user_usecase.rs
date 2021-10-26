use super::token_usecase::create_token_output;
use crate::context::Context;
use crate::model::error::{FormatError, SpecificError};
use crate::model::token::CreateTokenOutput;
use crate::model::user::{
  CreateUserInput, FindAUserInput, FindOneUserCondition, UpdateOneUserField, User, UserMetaData,
  Users, VerifyUserInput,
};
use crate::repository::refresh_token_repo::RefreshTokenRepo;
use crate::repository::user_repo::UserRepo;
use anyhow::{bail, Result};
use bson::bson;
use chrono::Utc;

pub struct UserUsecase<T>
where
  T: UserRepo,
{
  user_repo: T,
}

impl<T> UserUsecase<T>
where
  T: UserRepo,
{
  pub fn new(user_repo: T) -> UserUsecase<T> {
    UserUsecase { user_repo }
  }

  pub async fn find_many_users(&mut self) -> Result<Users> {
    let users = self.user_repo.find().await?;

    Ok(Users { items: users })
  }

  pub async fn find_user(&mut self, find_a_user_input: FindAUserInput) -> Result<User> {
    let condition: FindOneUserCondition;

    if find_a_user_input.id.is_some() {
      condition = FindOneUserCondition::Id(find_a_user_input.id.unwrap());
    } else if find_a_user_input.email.is_some() {
      condition = FindOneUserCondition::Email(find_a_user_input.email.unwrap());
    } else {
      bail!(FormatError::ValidationFailed(
        "FindAUserInput invalid".to_string()
      ));
    }

    let user = self.user_repo.find_one(&condition).await?;
    Ok(user)
  }
}

pub async fn create_user<R: UserRepo>(
  aud: String,
  user_repo: &R,
  create_user_req: CreateUserInput,
) -> Result<User> {
  let find_one_user_condition = FindOneUserCondition::Email(create_user_req.email.clone());
  let check_exist_user_res = user_repo.find_one(&find_one_user_condition).await;

  match check_exist_user_res {
    Ok(exist_user) => {
      if exist_user.is_confirmed() {
        bail!(FormatError::DuplicateError(
          "A user with this email address has already been registered".to_string()
        ));
      } else {
        // delete not confirm user then create new one
        user_repo.delete_one(&find_one_user_condition).await?;
      }
    }
    Err(err) => {
      // NOT_FOUND error mean this email address is available,
      // then user can use this email address to sign up
      if err.get_code() != "NOT_FOUND" {
        return Err(err);
      }
    }
  }

  let user_data = UserMetaData::new(bson!({}));

  let user = User::new(
    aud,
    create_user_req.email,
    create_user_req.password,
    user_data,
  )?;

  user_repo.insert(&user).await?;
  // TODO: format error
  // TODO: Send a verify email
  println!(
    "confirmation_token: {}",
    user.confirmation_token.clone().unwrap()
  );

  Ok(user)
}

// TODO: remove ctx
pub async fn verify_user<T: UserRepo, K: RefreshTokenRepo>(
  ctx: &Context,
  user_repo: &T,
  refresh_token_repo: &K,
  verify_user_input: &VerifyUserInput,
) -> Result<CreateTokenOutput> {
  // TODO: recover
  let find_one_user_condition =
    FindOneUserCondition::ConfirmationToken(verify_user_input.token.clone());
  let mut user = user_repo.find_one(&find_one_user_condition).await?;

  // update confirmed_at
  let mut update_confirmation_data = vec![
    UpdateOneUserField::ConfirmationToken(None),
    UpdateOneUserField::ConfirmedAt(Some(Utc::now())),
  ];

  if verify_user_input.password.is_some() {
    user.change_password(&verify_user_input.password.clone().unwrap())?;
    update_confirmation_data.push(UpdateOneUserField::EncryptedPassword(
      user.encrypted_password.clone(),
    ));
  }
  user_repo
    .update_one(&find_one_user_condition, &update_confirmation_data)
    .await?;

  create_token_output(ctx, refresh_token_repo, &user, None).await
}
