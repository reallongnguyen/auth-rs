use crate::model::id::Id;
use crate::model::token::RefreshToken;
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait RefreshTokenRepo {
  async fn insert(&self, refresh_token: &RefreshToken) -> Result<()>;
  async fn delete_by_user_id(&self, user_id: &Id) -> Result<()>;
  async fn swap_token(
    &self,
    user_id: &Id,
    old_refresh_token_value: String,
    new_refresh_token: &RefreshToken,
  ) -> Result<()>;
  async fn find_one_by_token(&self, token: String) -> Result<RefreshToken>;
}
