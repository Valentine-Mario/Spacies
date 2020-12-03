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
                    .route(
                        "/updatepassword",
                        web::post().to(user_handler::update_password),
                    )
                    .route("/addimg", web::post().to(user_handler::update_profile_img))
                    .route(
                        "/forgotpassword",
                        web::post().to(user_handler::forgot_password),
                    )
                    .route("/login", web::post().to(user_handler::login)),
            )
            .service(
                web::scope("/spacies")
                    .route("/get/{info}", web::get().to(space_handler::get_space))
                    .route("/myspaces", web::get().to(space_handler::get_user_space))
                    .route("/invitepage", web::get().to(space_handler::invite_page))
                    .route("/leave/{info}", web::get().to(space_handler::leave_space))
                    .route(
                        "/getusers/{info}",
                        web::get().to(space_handler::get_users_in_space),
                    )
                    .route(
                        "/getstatus/{info}",
                        web::get().to(space_handler::get_user_space_status),
                    )
                    .route(
                        "/updateimg/{info}",
                        web::post().to(space_handler::update_space_logo),
                    )
                    .route(
                        "/updatespace/{info}",
                        web::post().to(space_handler::update_space),
                    )
                    .route(
                        "/acceptinvite",
                        web::post().to(space_handler::add_invited_user),
                    )
                    .route(
                        "changeuserstatus/{info}",
                        web::post().to(space_handler::change_user_priviledge_status),
                    )
                    .route("/invite/{info}", web::post().to(space_handler::invite_user))
                    .route(
                        "/kickout/{info}",
                        web::post().to(space_handler::remove_user_from_space),
                    )
                    .route("/add", web::post().to(space_handler::add_space)),
            )
            .service(
                web::scope("/channel")
                    .route(
                        "/get/{info}",
                        web::get().to(channel_handler::get_channels_in_space),
                    )
                    .route(
                        "/delete/{info}/{channel}",
                        web::get().to(channel_handler::delete_channel),
                    )
                    .route(
                        "/add/{info}",
                        web::post().to(channel_handler::create_new_channel),
                    )
                    .route(
                        "/update/{info}/{channel}",
                        web::post().to(channel_handler::edit_channel_name),
                    ),
            )
            .service(
                web::scope("/maillist")
                    .route(
                        "/get/{info}",
                        web::get().to(mail_folder_handler::get_space_mail_folder),
                    )
                    .route(
                        "/getuser/{info}/{name}",
                        web::get().to(mail_folder_handler::get_mail_folder_details),
                    )
                    .route(
                        "/add/{info}",
                        web::post().to(mail_folder_handler::add_mail_folder),
                    )
                    .route(
                        "/delete/{info}",
                        web::post().to(mail_folder_handler::delete_mail_folder),
                    ),
            )
            .service(
                web::scope("/usermail")
                    .route(
                        "/add/{info}/{id}",
                        web::post().to(mail_user_handler::add_user_folder),
                    )
                    .route(
                        "/send/{id}",
                        web::post().to(mail_user_handler::send_mail_to_folder),
                    )
                    .route(
                        "/delete/{info}/{id}",
                        web::post().to(mail_user_handler::remove_user_folder),
                    ),
            )
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
