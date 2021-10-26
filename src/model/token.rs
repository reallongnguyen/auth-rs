use super::crypto::create_unique_token;
use super::id::Id;
use chrono::{DateTime, Utc};
use juniper::{GraphQLEnum, GraphQLInputObject, GraphQLObject};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct RefreshToken {
  pub id: Id,
  pub token: String,
  pub user_id: Id,
  pub revoked: bool,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

impl RefreshToken {
  pub fn new(user_id: &Id) -> Self {
    RefreshToken {
      id: Id::create_uuid_v4(),
      token: create_unique_token(),
      user_id: user_id.clone(),
      revoked: false,
      created_at: Utc::now(),
      updated_at: Utc::now(),
    }
  }

  pub fn is_revoked(&self) -> bool {
    self.revoked
  }
}

pub const ACCESS_TOKEN_EXP: usize = 3600;

// pub enum PermissionScope {
//   AdminUserRead,
//   AdminUserWrite,
//   UserRead,
//   UserWrite,
// }

// impl PermissionScope {
//   pub fn get_val(&self) -> &str {
//     match self {
//       Self::AdminUserRead => "admin.user:read",
//       Self::AdminUserWrite => "admin.user:write",
//       Self::UserRead => "user:read",
//       Self::UserWrite => "user:write",
//     }
//   }
// }

// pub const GENERAL_USER_SCOPE: [PermissionScope; 2] = [
//   PermissionScope::UserRead,
//   PermissionScope::UserWrite,
// ];

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
  pub user_id: Id,
  pub aud: String,
  pub exp: usize,
  pub iat: usize,
}

impl Claims {
  pub fn new(user_id: Id, aud: String, exp: usize) -> Self {
    let iat = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .unwrap()
      .as_secs() as usize;

    Claims {
      user_id,
      aud,
      exp,
      iat,
    }
  }
}

#[derive(Debug, GraphQLEnum)]
pub enum CreateTokenGrantType {
  Password,
  RefreshToken,
}

#[derive(GraphQLInputObject, Debug)]
#[graphql(description = "Create token (sign in) input")]
pub struct CreateTokenInput {
  pub grant_type: CreateTokenGrantType,
  pub email: Option<String>,
  pub password: Option<String>,
  pub refresh_token: Option<String>,
}

#[derive(Debug, GraphQLObject)]
#[graphql(description = "Create token (sign in) result")]
pub struct CreateTokenOutput {
  pub access_token: String,
  pub token_type: String,
  pub expires_in: i32,
  pub refresh_token: String,
}

pub struct CreateTokenByPasswordInput {
  pub email: String,
  pub password: String,
}
