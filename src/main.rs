use actix_web::{web, App, HttpServer};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use dotenv::dotenv;
use std::env;

mod model;
mod repository;
mod service;
mod handler;
mod routes;
mod error;
// mod traits;
// mod utils;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool: Pool<Postgres> = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create pool.");

    let repository = Arc::new(SqlxClientRepository::new(pool));

    let service = Arc::new(ClientService::new(repository))

    HttpServer::new(move || {
        App::new()
           // .app_data(web::Data::new(pool.clone())) -> versão antes do service
           .app_data(web::Data::new(service))
           .configure(routes::client_routes::config)
           
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

#[cfg(test)]
pub mod tests_util;