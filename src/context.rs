use crate::model::error::FormatError;
use crate::model::token::Claims;
use crate::usecase::token_usecase::decode_access_token;
use actix_web::HttpRequest;
use anyhow::{bail, Result};
use config::Config;
use juniper;
use sqlx::PgPool;

pub struct Context {
  auth_db_pool: PgPool,
  pub settings: Config,
  pub access_token: Option<String>,
}

impl Context {
  pub fn new(auth_db_pool: PgPool, settings: Config, req: &HttpRequest) -> Context {
    let authorization = req.headers().get::<String>("Authorization".to_string());
    let authorization = authorization.and_then(|x| x.to_str().ok());
    let access_token =
      authorization.and_then(|bearer_string| extract_bearer_token(&bearer_string.to_string()));

    Context {
      auth_db_pool,
      settings,
      access_token,
    }
  }

  // check access token (jwt).
  // If access token is valid, it return a Claims struct.
  // In other words, it return FormatError::Unauthorized error.
  pub fn get_current_user_claims(&self) -> Result<Claims> {
    if self.access_token.is_none() {
      bail!(FormatError::Unauthenticated(
        "access_token not set. You should put access token into request's headers: 'Authorization: bearer TOKEN'.".to_string()
      ));
    }

    let access_token = self.access_token.clone().unwrap();
    let verified_claims = decode_access_token(&self.settings, &access_token);
    println!("claims: {:?}", verified_claims);

    verified_claims
  }

  pub fn auth_db_pool_ref(&self) -> &PgPool {
    &self.auth_db_pool
  }
}

impl juniper::Context for Context {}

fn extract_bearer_token(bearer_token: &String) -> Option<String> {
  if !bearer_token.starts_with("bearer ") {
    return None;
  }

  Some(bearer_token[7..].to_string())
}
