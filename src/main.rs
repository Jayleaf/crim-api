#[macro_use]
extern crate took_macro;

mod db;
mod generics;
mod routes;

use axum::{
    routing::{get, post}, Router
};
use tokio;

#[tokio::main]
async fn main()
{
    // build routes
    // TODO: rate limiting
    println!("{}", String::new());
    db::mongo::ping().await;
    let app = Router::new()
        .route("/api/auth/create", post(routes::auth::create::create_user))
        .route("/api/auth/delete", post(routes::auth::delete::delete_user))
        .route("/api/auth/login", post(routes::auth::login::login_user))
        .route("/api/auth/get", post(routes::auth::get::get)) // TODO: this could be a get instead of a post
        .route("/api/auth/update", post(routes::auth::update::update))
        .route("/api/message/recieve", post(routes::message::recieve::recieve));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}
