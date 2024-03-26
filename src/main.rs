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
use tracing::log::{debug, error, info};
use crate::generics::structs::ClientStore;

#[tokio::main]
async fn main()
{
    tracing_subscriber::registry()
        .with(
            EnvFilter::from("info")
                .add_directive("hyper=info".parse().unwrap())
                .add_directive("axum=info".parse().unwrap())
                .add_directive("example_websockets=debug".parse().unwrap())
                .add_directive("tower_http=debug".parse().unwrap())
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    if let Err(e) = db::mongo::ping().await
    {
        error!("Failed to connect to MongoDB! {e}");
        return;
    }
    else { info!("Connected to MongoDB!") }

    let state = ClientStore::default();


    let app = Router::new()
        .route("/api/auth/create", post(routes::auth::create::create_user))
        .route("/api/auth/delete", post(routes::auth::delete::delete_user))
        .route("/api/auth/login", post(routes::auth::login::login_user))
        .route("/api/auth/get/:{sid}", get(routes::auth::get::get))
        .route("/api/ws", get(routes::ws::ws::ws_handler))
        .with_state(state)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default()),
        );


    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();
    debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap()

}

