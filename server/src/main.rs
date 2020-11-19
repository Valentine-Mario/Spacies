#[macro_use]
extern crate diesel;

use actix_cors::Cors;
use actix_web::{middleware::Logger, web, App, HttpServer};
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use listenfd::ListenFd;
use std::env;

pub mod auth;
pub mod handlers;
pub mod helpers;
pub mod model;
mod schema;

//connection pool type
pub type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;

use crate::handlers::*;
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

    println!("running on host {} on port {}", host, port);
    let mut listenfd = ListenFd::from_env();

    let mut server = HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive())
            .wrap(Logger::default())
            .data(pool.clone())
            .service(
                web::scope("/user")
                    .route("/verify", web::get().to(user_handler::verify_user))
                    .route(
                        "/resend/verification",
                        web::get().to(user_handler::resend_verification),
                    )
                    .route("/profile", web::get().to(user_handler::get_profile))
                    .route("/add", web::post().to(user_handler::add_user))
                    .route("/updatename", web::post().to(user_handler::update_name))
                    .route("/addimg", web::post().to(user_handler::update_profile_img))
                    .route("/login", web::post().to(user_handler::login)),
            )
            .service(web::scope("/spacies"))
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
//systemfd --no-pid -s http::PORT -- cargo watch -x run
