use crate::context::Context;
use crate::model::{
  error::to_juniper_field_error,
  user::{FindAUserInput, User, Users},
};
use crate::repository::sql::user_repo_sql::UserRepoSql;
use crate::usecase::user_usecase::UserUsecase;
use juniper;
use juniper::FieldResult;

pub struct QueryRoot;

#[juniper::graphql_object(Context = Context)]
impl QueryRoot {
  #[graphql(description = "Find many users")]
  async fn users(ctx: &Context) -> FieldResult<Users> {
    ctx.get_current_user_claims()?;
    let mut user_usecase = UserUsecase::new(UserRepoSql::new(ctx));

    user_usecase
      .find_many_users()
      .await
      .map_err(to_juniper_field_error)
  }

  #[graphql(description = "Find one user")]
  async fn user(ctx: &Context, find_a_user_input: FindAUserInput) -> FieldResult<User> {
    ctx.get_current_user_claims()?;
    let mut user_usecase = UserUsecase::new(UserRepoSql::new(ctx));

    user_usecase
      .find_user(find_a_user_input)
      .await
      .map_err(to_juniper_field_error)
  }

  #[graphql(description = "Get user information for logged in user")]
  async fn me(ctx: &Context) -> FieldResult<User> {
    let claim = ctx.get_current_user_claims()?;
    let mut user_usecase = UserUsecase::new(UserRepoSql::new(ctx));

    let find_me = FindAUserInput {
      id: Some(claim.user_id),
      email: None,
    };

    user_usecase
      .find_user(find_me)
      .await
      .map_err(to_juniper_field_error)
  }
}
