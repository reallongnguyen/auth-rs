use crate::context::Context;
use crate::presentation::auth::graphql::{create_schema, Schema};
use actix_web::{web, Error, HttpRequest, HttpResponse};
use config::Config;
use juniper::http::{playground::playground_source, GraphQLRequest};
use sqlx::PgPool;

const APP_GRAPHQL_ENDPOINT: &'static str = "/auth/graphql";
const APP_PLAYGROUND_ENDPOINT: &'static str = "/auth/playground";

pub async fn graphql(
  pg_pool: web::Data<PgPool>,
  settings: web::Data<Config>,
  schema: web::Data<Schema>,
  req: HttpRequest,
  data: web::Json<GraphQLRequest>,
) -> Result<HttpResponse, Error> {
  let ctx = Context::new(
    pg_pool.get_ref().to_owned(),
    settings.get_ref().to_owned(),
    &req,
  );

  let f = || async move {
    let res = data.execute(&schema, &ctx).await;
    Ok::<_, serde_json::error::Error>(serde_json::to_string(&res)?)
  };

  let res = f().await.map_err(Error::from)?;

  Ok(
    HttpResponse::Ok()
      .content_type("application/json")
      .body(res),
  )
}

pub async fn graphql_playground() -> HttpResponse {
  HttpResponse::Ok()
    .content_type("text/html; charset=utf-8")
    .body(playground_source(APP_GRAPHQL_ENDPOINT, None))
}

pub fn register(config: &mut web::ServiceConfig) {
  config
    .data(create_schema())
    .route(APP_GRAPHQL_ENDPOINT, web::post().to(graphql))
    .route(APP_PLAYGROUND_ENDPOINT, web::get().to(graphql_playground));
}
