mod db;
mod generics;
mod routes;
use tokio;
use axum::{
    routing::get,
    routing::post,
    Router,
};
use std::net::SocketAddr;
use tower_http::trace::{DefaultMakeSpan, TraceLayer};

use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

use crate::generics::structs::ClientStore;

#[tokio::main]
async fn main()
{
    // build routes
    // TODO: rate limiting

    if let Err(e) = db::mongo::ping().await
    {
        eprintln!("Failed to connect to MongoDB: {}", e);
        return;
    }
    else { println!("Connected to MongoDB.");}


    // what the fuck is this?
    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "example_websockets=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let state = ClientStore::default();


    let app = Router::new()
        .route("/api/auth/create", post(routes::auth::create::create_user))
        .route("/api/auth/delete", post(routes::auth::delete::delete_user))
        .route("/api/auth/login", post(routes::auth::login::login_user))
        .route("/api/auth/get", post(routes::auth::get::get)) // TODO: this could be a get instead of a post
        .route("/api/auth/update", post(routes::auth::update::update))
        .route("/api/message/recieve", post(routes::message::recieve::recieve))
        .route("/api/ws", get(routes::ws::ws::ws_handler))
        .with_state(state)
        // logging so we can see whats going on
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        );


    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap()

}

