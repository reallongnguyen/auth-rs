use crate::model::user::{FindOneUserCondition, UpdateOneUserField, User};
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait UserRepo {
  async fn insert(&self, user: &User) -> Result<()>;
  async fn find_one(&self, condition: &FindOneUserCondition) -> Result<User>;
  async fn find(&self) -> Result<Vec<User>>;
  async fn update_one(
    &self,
    condition: &FindOneUserCondition,
    update_data: &Vec<UpdateOneUserField>,
  ) -> Result<()>;
  async fn delete_one(&self, condition: &FindOneUserCondition) -> Result<()>;
}
