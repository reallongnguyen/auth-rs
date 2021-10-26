use actix_web::{get, middleware, App, HttpResponse, HttpServer, Responder};
use dotenv::dotenv;
use sqlx::PgPool;
use std::io::Result;

mod config;
mod context;
mod handler;
mod model;
mod presentation;
mod repository;
mod usecase;

#[get("/")]
async fn health_check() -> impl Responder {
    HttpResponse::Ok().body("OK")
}

#[actix_web::main]
async fn main() -> Result<()> {
    dotenv().ok();
    env_logger::init();
    let settings = config::init_config();
    let port = settings.get::<String>("port").expect("port must set");
    let addr = format!("localhost:{}", port);
    let auth_db_url = settings
        .get::<String>("database_url")
        .expect("database_url must set");
    let auth_db_pool = PgPool::connect(auth_db_url.as_str())
        .await
        .expect("Connect to authdb must success");

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .data(auth_db_pool.clone())
            .data(settings.clone())
            .configure(handler::register)
            .service(health_check)
    })
    .bind(addr)?
    .run()
    .await
}
