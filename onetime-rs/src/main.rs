mod controller;
mod model;
mod payload;

use tower_http::cors::Any;
use tower_http::cors::CorsLayer;
use crate::controller::get_secret;
use crate::controller::create_secret;
use axum::http::{Method};

use std::env;

use bb8_redis::{
    bb8,
    RedisConnectionManager
};

use axum::{
    routing::{get, post}, Router,
};
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let redis_conn_string = env::var("REDIS_CONN").expect("Missing environment variable REDIS_CONN");
    let cors = CorsLayer::new()
        // allow `GET` and `POST` when accessing the resource
        .allow_methods([Method::GET, Method::POST])
        // allow requests from any origin
        .allow_origin(Any);

    let manager = RedisConnectionManager::new(redis_conn_string).unwrap();
    let pool = bb8::Pool::builder().build(manager).await.unwrap();

    let app = Router::new()
        .route("/", get(root))
        .route("/secrets/:secret_id", get(get_secret))
        .route("/secrets", post(create_secret))
        .layer(cors)
        .with_state(pool);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn root() -> &'static str {
    "Hello, World!"
}
