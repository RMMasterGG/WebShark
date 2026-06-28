pub mod filters;
pub mod helpers;
pub mod utils;
pub mod controller;

use crate::controller::user_controller:: UserController;
use crate::filters::auth_filter::AuthFilter;
use crate::filters::log_filter::LoggerFilter;
use webshark::routing::scope::Scope;
use webshark::{tokio};
use webshark::{Router, Server};
use crate::controller::stores_controller::StoresController;


#[webshark::main]
async fn main() {
    let mut router = Router::new();

    let users_controller = Scope::new("/users").configure(UserController::configure);

    let stores_controller = Scope::new("/stores").configure(StoresController::configure);

    let api_v1 = Scope::new("/api/v1")
        .with_filter(LoggerFilter)
        .with_filter(AuthFilter)
        .nest(users_controller)
        .nest(stores_controller);

    router.add_scope(api_v1);

    let server = Server::new(router)
        .http1(true)
        .http2(true)
        .http3(true);

    server.start_server().await.expect("Failed to start server");
}