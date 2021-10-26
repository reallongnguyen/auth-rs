use crate::context::Context;
use crate::model::error::to_juniper_field_error;
use crate::model::id::Id;
use crate::model::token::{
  CreateTokenByPasswordInput, CreateTokenGrantType, CreateTokenInput, CreateTokenOutput,
};
use crate::model::user::{CreateUserInput, UpdateUserInput, User, VerifyUserInput};
use crate::repository::sql::refresh_token_repo_sql::RefreshTokenRepoSQL;
use crate::repository::sql::user_repo_sql::UserRepoSql;
use crate::usecase::token_usecase::{auth_by_password, auth_by_refresh_token, logout};
use crate::usecase::user_usecase::{create_user, verify_user};
use juniper;
use juniper::FieldResult;

pub struct MutationRoot;

#[juniper::graphql_object(Context = Context)]
impl MutationRoot {
  async fn crate_user(ctx: &Context, create_user_input: CreateUserInput) -> FieldResult<User> {
    let aud = ctx.settings.get::<String>("aud").expect("aud must set");
    let user_repo = UserRepoSql::new(ctx);
    create_user(aud, &user_repo, create_user_input)
      .await
      .map_err(to_juniper_field_error)
  }

  async fn update_user(
    ctx: &Context,
    id: Id,
    update_user_input: UpdateUserInput,
  ) -> FieldResult<String> {
    println!(
      "id: {}, name: {:?}",
      id,
      update_user_input.data.unwrap().get_data()
    );
    Ok("ok".to_string())
  }

  async fn create_token(
    ctx: &Context,
    create_token_input: CreateTokenInput,
  ) -> FieldResult<CreateTokenOutput> {
    let user_repo = UserRepoSql::new(ctx);
    let token_repo = RefreshTokenRepoSQL::new(ctx);
    match create_token_input.grant_type {
      CreateTokenGrantType::Password => {
        // TODO: validate input
        let create_token_by_password_input = CreateTokenByPasswordInput {
          email: create_token_input.email.unwrap(),
          password: create_token_input.password.unwrap(),
        };

        auth_by_password(ctx, user_repo, token_repo, &create_token_by_password_input)
          .await
          .map_err(to_juniper_field_error)
      }
      CreateTokenGrantType::RefreshToken => {
        // TODO: validate refresh_token
        let refresh_token_value = create_token_input.refresh_token.unwrap();
        auth_by_refresh_token(ctx, user_repo, token_repo, refresh_token_value)
          .await
          .map_err(to_juniper_field_error)
      }
    }
  }

  async fn logout(ctx: &Context) -> FieldResult<String> {
    let claim = ctx
      .get_current_user_claims()
      .map_err(to_juniper_field_error)?;

    let refresh_token_repo = RefreshTokenRepoSQL::new(ctx);
    logout(ctx, &refresh_token_repo, &claim.user_id).await?;

    // TODO: write specific result
    Ok("dummy response".to_string())
  }

  async fn verify(
    ctx: &Context,
    verify_user_input: VerifyUserInput,
  ) -> FieldResult<CreateTokenOutput> {
    let user_repo = UserRepoSql::new(ctx);
    let refresh_token_repo = RefreshTokenRepoSQL::new(ctx);

    verify_user(ctx, &user_repo, &refresh_token_repo, &verify_user_input)
      .await
      .map_err(to_juniper_field_error)
  }
}
