use super::crypto::create_unique_token;
use super::id::Id;
use crate::model::error::FormatError;
use anyhow::{bail, Result};
use bcrypt::{hash, verify, DEFAULT_COST};
use bson::{bson, doc, Bson};
use chrono::{DateTime, Utc};
use juniper::{
  graphql_object, graphql_scalar, GraphQLEnum, GraphQLInputObject, GraphQLObject,
  ParseScalarResult, ParseScalarValue, Value,
};
use std::fmt;

#[derive(Debug)]
pub enum DefaultUserRole {
  GeneralUser,
  SuperAdmin,
}

impl fmt::Display for DefaultUserRole {
  fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
    write!(fmt, "{:?}", self)
  }
}

#[derive(Debug, Clone)]
pub struct UserMetaData(Bson);

impl UserMetaData {
  pub fn new(doc: Bson) -> UserMetaData {
    UserMetaData(doc)
  }

  pub fn get_data(&self) -> &Bson {
    &self.0
  }
}

#[graphql_scalar(description = "User meta data")]
impl<S> GraphQLScalar<S> for UserMetaData
where
  S: ScalarValue,
{
  fn resolve(&self) -> Value {
    Value::scalar(self.0.to_string())
  }

  fn from_input_value(v: &InputValue) -> Option<Self> {
    let val = v.as_string_value();
    println!("val {:?}", val);
    let bson: Bson = bson!(val);

    Some(UserMetaData(bson))
  }

  fn from_str<'a>(value: ScalarToken<'a>) -> ParseScalarResult<'a, S> {
    <String as ParseScalarValue<S>>::from_str(value)
  }
}

#[derive(Debug, Clone)]
pub struct User {
  pub id: Id,
  pub aud: String,
  pub email: String,
  pub role: String,
  pub encrypted_password: String,
  pub user_meta_data: UserMetaData,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
  pub invited_at: Option<DateTime<Utc>>,
  pub confirmed_at: Option<DateTime<Utc>>,
  pub confirmation_token: Option<String>,
  pub confirmation_sent_at: Option<DateTime<Utc>>,
  pub last_sign_in_at: Option<DateTime<Utc>>,
}

impl User {
  pub fn new(
    aud: String,
    email: String,
    password: String,
    user_meta_data: UserMetaData,
  ) -> Result<User> {
    let encrypted_password = hash_password(&password)?;
    let id = Id::create_uuid_v4();
    let confirmation_token = create_unique_token();

    let user = User {
      id,
      aud,
      email,
      encrypted_password,
      role: DefaultUserRole::GeneralUser.to_string(),
      user_meta_data,
      created_at: Utc::now(),
      updated_at: Utc::now(),
      invited_at: None,
      confirmed_at: None,
      confirmation_token: Some(confirmation_token),
      confirmation_sent_at: Some(Utc::now()),
      last_sign_in_at: None,
    };

    Ok(user)
  }

  pub fn raw_user_meta_data(&self) -> String {
    self.user_meta_data.0.to_string()
  }

  pub fn change_password(&mut self, password: &String) -> Result<&mut Self> {
    self.encrypted_password = hash_password(password)?;
    Ok(self)
  }

  pub fn check_password(&self, password: &String) -> Result<()> {
    if verify(password, self.encrypted_password.as_str())? {
      return Ok(());
    }

    bail!(FormatError::Unauthenticated(
      "Username or Password is incorrect".to_string()
    ))
  }

  pub fn is_confirmed(&self) -> bool {
    self.confirmed_at.is_some()
  }
}

fn hash_password(password: &String) -> Result<String> {
  match hash(password, DEFAULT_COST) {
    Ok(password) => Ok(password),
    Err(err) => bail!(FormatError::ServerError(err.to_string())),
  }
}

#[derive(Debug, GraphQLObject)]
pub struct Users {
  pub items: Vec<User>,
}

#[graphql_object]
impl User {
  fn id(&self) -> &Id {
    &self.id
  }
  fn email(&self) -> &String {
    &self.email
  }
  fn role(&self) -> &String {
    &self.role
  }
  fn user_meta_data(&self) -> &UserMetaData {
    &self.user_meta_data
  }
  fn created_at(&self) -> &DateTime<Utc> {
    &self.created_at
  }
  fn updated_at(&self) -> &DateTime<Utc> {
    &self.updated_at
  }
}

#[derive(GraphQLInputObject, Debug)]
#[graphql(description = "Create user input")]
pub struct CreateUserInput {
  pub email: String,
  pub password: String,
}

#[derive(GraphQLInputObject, Debug)]
#[graphql(description = "Find a user")]
pub struct FindAUserInput {
  pub id: Option<Id>,
  pub email: Option<String>,
}

#[derive(Debug, GraphQLEnum)]
pub enum VerifyUserType {
  SignUp,
  Recover,
}

#[derive(Debug, GraphQLInputObject)]
pub struct VerifyUserInput {
  pub verify_type: VerifyUserType,
  pub token: String,
  pub password: Option<String>,
}

#[derive(Debug, GraphQLInputObject)]
pub struct UpdateUserInput {
  pub data: Option<UserMetaData>,
}

#[derive(Debug, Clone)]
pub enum FindOneUserCondition {
  Id(Id),
  Email(String),
  ConfirmationToken(String),
}

#[derive(Debug, Clone)]
pub enum UpdateOneUserField {
  ConfirmationToken(Option<String>),
  ConfirmedAt(Option<DateTime<Utc>>),
  EncryptedPassword(String),
}
