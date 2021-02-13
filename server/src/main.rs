#[macro_use]
extern crate diesel;
use crate::helpers::schedule::Scheduler;
use actix::prelude::*;
use actix_cors::Cors;
use actix_web::{middleware::Logger, App, HttpServer};
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use listenfd::ListenFd;
use std::env;
#[macro_use]
extern crate magic_crypt;

pub mod auth;
mod config;
pub mod handlers;
pub mod helpers;
pub mod model;
mod schema;

//connection pool type
pub type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    std::env::set_var("RUST_LOG", "actix_web=debug");
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    std::fs::create_dir_all("./tmp").unwrap();
    // create db connection pool
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool: Pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    let host = env::var("HOST").expect("Please set host in .env");
    let port = env::var("PORT").expect("Please set port in .env");

    //start cron jobs
    Scheduler.start();

    println!("running on host {} on port {}", host, port);
    let mut listenfd = ListenFd::from_env();

    let mut server = HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive())
            .wrap(Logger::default())
            .data(pool.clone())
            .configure(config::config_routes)
    });

    server = match listenfd.take_tcp_listener(0)? {
        Some(listener) => server.listen(listener)?,
        None => {
            let host = env::var("HOST").expect("Please set host in .env");
            let port = env::var("PORT").expect("Please set port in .env");
            server.bind(format!("{}:{}", host, port))?
        }
    };

    server.run().await
}
//diesel print-schema > src/schema.rs
//systemfd --no-pid -s http::5000 -- cargo watch -x run
