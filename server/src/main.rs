pub mod auth;
pub mod handlers;
pub mod helpers;
pub mod model;
mod schema;

// use crate::handlers::*;
fn main() {
    dotenv::dotenv().ok();
    println!("Hello, world!");
}
//diesel print-schema > src/schema.rs
//systemfd --no-pid -s http::5000 -- cargo watch -x run
