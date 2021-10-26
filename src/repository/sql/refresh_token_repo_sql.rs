use super::super::refresh_token_repo::RefreshTokenRepo;
use crate::context::Context;
use crate::model::error::FormatError;
use crate::model::id::Id;
use crate::model::token::RefreshToken;
use anyhow::{anyhow, bail, Result};
use async_trait::async_trait;
use sqlx::{postgres::PgRow, Row};

pub struct RefreshTokenRepoSQL<'a> {
  ctx: &'a Context,
}

impl<'a> RefreshTokenRepoSQL<'a> {
  pub fn new(ctx: &'a Context) -> Self {
    RefreshTokenRepoSQL { ctx }
  }
}

#[async_trait]
impl RefreshTokenRepo for RefreshTokenRepoSQL<'_> {
  async fn insert(&self, refresh_token: &RefreshToken) -> Result<()> {
    let query = format!(
      "
      INSERT INTO refresh_tokens (id, token, user_id, revoked)
      VALUES ('{id}', '{token}', '{user_id}', false)
    ",
      id = refresh_token.id,
      token = refresh_token.token,
      user_id = refresh_token.user_id
    );

    sqlx::query(query.as_str())
      .execute(self.ctx.auth_db_pool_ref())
      .await
      .map_err(sqlx_err_to_anyhow)?;

    Ok(())
  }

  async fn delete_by_user_id(&self, user_id: &Id) -> Result<()> {
    sqlx::query(
      "
      DELETE FROM refresh_tokens
      WHERE user_id = $1::text
    ",
    )
    .bind(user_id.to_string())
    .execute(self.ctx.auth_db_pool_ref())
    .await
    .map_err(sqlx_err_to_anyhow)?;

    Ok(())
  }

  async fn swap_token(
    &self,
    user_id: &Id,
    old_refresh_token_value: String,
    new_refresh_token: &RefreshToken,
  ) -> Result<()> {
    let old_refresh_token = sqlx::query(
      r#"
        SELECT revoked, id, created_at, updated_at
        FROM refresh_tokens
        WHERE user_id = $1::text and token = $2::text
      "#,
    )
    .bind(user_id.to_string())
    .bind(old_refresh_token_value.clone())
    .map(|row: PgRow| RefreshToken {
      id: Id::new(row.get("id")),
      revoked: row.get("revoked"),
      token: old_refresh_token_value.clone(),
      user_id: user_id.clone(),
      created_at: row.get("created_at"),
      updated_at: row.get("updated_at"),
    })
    .fetch_one(self.ctx.auth_db_pool_ref())
    .await
    .map_err(sqlx_err_to_anyhow)?;

    if old_refresh_token.is_revoked() {
      bail!(FormatError::Unauthenticated(
        "Refresh token has expired".to_string()
      ));
    }

    sqlx::query(
      r#"
        UPDATE refresh_tokens
        SET revoked = TRUE
        WHERE id = $1::text
      "#,
    )
    .bind(old_refresh_token.id.to_string())
    .execute(self.ctx.auth_db_pool_ref())
    .await
    .map_err(sqlx_err_to_anyhow)?;

    self.insert(new_refresh_token).await?;

    Ok(())
  }

  async fn find_one_by_token(&self, token_value: String) -> Result<RefreshToken> {
    sqlx::query(
      r#"
        SELECT id, user_id, revoked, created_at, updated_at
        FROM refresh_tokens
        WHERE token = $1::text
      "#,
    )
    .bind(token_value.clone())
    .map(|row: PgRow| RefreshToken {
      id: Id::new(row.get("id")),
      user_id: Id::new(row.get("user_id")),
      token: token_value.clone(),
      revoked: row.get("revoked"),
      created_at: row.get("created_at"),
      updated_at: row.get("updated_at"),
    })
    .fetch_one(self.ctx.auth_db_pool_ref())
    .await
    .map_err(sqlx_err_to_anyhow)
  }
}

fn sqlx_err_to_anyhow(e: sqlx::Error) -> anyhow::Error {
  let message = e.to_string();
  if message.starts_with("no rows returned") {
    return anyhow!(FormatError::NotFoundError(message));
  }

  return anyhow!(FormatError::ServerError(message));
}
