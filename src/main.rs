#[macro_use]
extern crate took_macro;

mod routes;
mod structs;
mod db;

use axum::{
    routing::{get, post},
    Router,
};
use tokio;

#[tokio::main]
async fn main() {
    // build our application with a single route
    let app = Router::new()
        .route("/api", get(|| async { "Hello, World!" }))
        .route("/api/auth/create", post(routes::auth::create::create_user))
        .route("/api/auth/delete", post(routes::auth::delete::delete_user))
        .route("/api/auth/login", post(routes::auth::login::login_user));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}