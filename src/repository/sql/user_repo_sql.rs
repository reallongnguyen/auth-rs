use crate::context::Context;
use crate::model::error::FormatError;
use crate::model::{
  id::Id,
  user::{FindOneUserCondition, UpdateOneUserField, User, UserMetaData},
};
use crate::repository::user_repo::UserRepo;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use bson::Bson;
use sqlx::{postgres::PgRow, types::Json, Row};

pub struct UserRepoSql<'a> {
  ctx: &'a Context,
}

impl UserRepoSql<'_> {
  pub fn new(ctx: &Context) -> UserRepoSql {
    UserRepoSql { ctx }
  }
}

#[async_trait]
impl UserRepo for UserRepoSql<'_> {
  async fn insert(&self, user: &User) -> Result<()> {
    sqlx::query(
      r#"
        INSERT INTO users (id, aud, email, role, encrypted_password, raw_user_meta_data, confirmation_token)
        VALUES ($1, $2, $3, $4, $5, $6::jsonb, $7)
      "#,
    )
    .bind(user.id.to_string())
    .bind(user.aud.clone())
    .bind(user.email.clone())
    .bind(user.role.clone())
    .bind(user.encrypted_password.clone())
    .bind(user.raw_user_meta_data().clone())
    .bind(user.confirmation_token.clone())
    .fetch_all(self.ctx.auth_db_pool_ref())
    .await
    .map_err(sqlx_err_to_anyhow)?;

    Ok(())
  }

  async fn find(&self) -> Result<Vec<User>> {
    let users = sqlx::query(
      r#"
        SELECT
          id, aud, email, role, encrypted_password, raw_user_meta_data,
          created_at, updated_at, confirmation_token, confirmed_at,
          invited_at, last_sign_in_at, confirmation_sent_at
        FROM users;
      "#,
    )
    .map(pg_row_to_user)
    .fetch_all(self.ctx.auth_db_pool_ref())
    .await
    .map_err(sqlx_err_to_anyhow)?;

    Ok(users)
  }

  async fn find_one(&self, condition: &FindOneUserCondition) -> Result<User> {
    let where_clause = find_one_user_condition_to_string(condition);

    let query = format!(
      "
        SELECT
          id, aud, email, role, encrypted_password, raw_user_meta_data,
          created_at, updated_at, confirmation_token, confirmed_at,
          invited_at, last_sign_in_at, confirmation_sent_at
        FROM users
        WHERE {}
      ",
      where_clause
    );
    let user = sqlx::query(query.as_str())
      .map(pg_row_to_user)
      .fetch_one(self.ctx.auth_db_pool_ref())
      .await
      .map_err(sqlx_err_to_anyhow)?;

    Ok(user)
  }

  async fn update_one(
    &self,
    condition: &FindOneUserCondition,
    update_data: &Vec<UpdateOneUserField>,
  ) -> Result<()> {
    let update_content = update_data
      .into_iter()
      .map(update_user_field_to_string)
      .collect::<Vec<String>>()
      .join(", ");
    let where_clause = find_one_user_condition_to_string(condition);

    let query = format!(
      "
        UPDATE users
        SET {}
        WHERE {}
      ",
      update_content, where_clause,
    );

    sqlx::query(query.as_str())
      .execute(self.ctx.auth_db_pool_ref())
      .await
      .map_err(sqlx_err_to_anyhow)?;

    Ok(())
  }

  async fn delete_one(&self, condition: &FindOneUserCondition) -> Result<()> {
    let where_clause = find_one_user_condition_to_string(condition);
    let query = format!(" DELETE from users WHERE {}", where_clause);
    sqlx::query(query.as_str())
      .execute(self.ctx.auth_db_pool_ref())
      .await?;

    Ok(())
  }
}

fn find_one_user_condition_to_string(find_one_condition: &FindOneUserCondition) -> String {
  match find_one_condition {
    FindOneUserCondition::Id(id) => format!("id = '{}'", id.to_string()),
    FindOneUserCondition::Email(email) => format!("email = '{}'", email),
    FindOneUserCondition::ConfirmationToken(token) => {
      format!("confirmation_token = '{}'", token)
    }
  }
}

fn update_user_field_to_string(user_field: &UpdateOneUserField) -> String {
  match user_field {
    UpdateOneUserField::ConfirmationToken(token) => {
      if token.is_some() {
        format!("confirmation_token = '{}'", token.clone().unwrap())
      } else {
        "confirmation_token = NULL".to_string()
      }
    }
    UpdateOneUserField::ConfirmedAt(time) => {
      if time.is_some() {
        "confirmed_at = NOW()".to_string()
      } else {
        "confirmed_at = NULL".to_string()
      }
    }
    UpdateOneUserField::EncryptedPassword(password) => {
      format!("encrypted_password = '{}'", password)
    }
  }
}

fn pg_row_to_user(row: PgRow) -> User {
  User {
    id: Id::new(row.get("id")),
    aud: row.get("aud"),
    email: row.get("email"),
    encrypted_password: row.get("encrypted_password"),
    role: row.get("role"),
    user_meta_data: UserMetaData::new(
      row
        .get::<Json<Bson>, &str>("raw_user_meta_data")
        .as_ref()
        .clone(),
    ),
    created_at: row.get("created_at"),
    updated_at: row.get("updated_at"),
    confirmation_token: row.get("confirmation_token"),
    confirmed_at: row.get("confirmed_at"),
    invited_at: row.get("invited_at"),
    last_sign_in_at: row.get("last_sign_in_at"),
    confirmation_sent_at: row.get("confirmation_sent_at"),
  }
}

fn sqlx_err_to_anyhow(e: sqlx::Error) -> anyhow::Error {
  let message = e.to_string();
  if message.starts_with("no rows returned") {
    return anyhow!(FormatError::NotFoundError(message));
  }

  return anyhow!(FormatError::ServerError(message));
}
