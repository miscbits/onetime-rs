use axum::extract::FromRequestParts;
use axum::extract::FromRef;
use crate::bb8::PooledConnection;
use crate::bb8::Pool;

use uuid::Uuid;
use std::env;

use bb8_redis::{
    bb8,
    redis::{AsyncCommands},
    RedisConnectionManager
};

use axum::{
    async_trait,
    routing::{get, post},
    http::{request::Parts, StatusCode},
    Json, Router,
    extract::Path,
};

use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let redis_conn_string = env::var("REDIS_CONN").expect("Missing environment variable REDIS_CONN");

    let manager = RedisConnectionManager::new(redis_conn_string).unwrap();
    let pool = bb8::Pool::builder().build(manager).await.unwrap();

    let app = Router::new()
        .route("/", get(root))
        .route("/secrets/:secret_id", get(get_secret))
        .route("/secrets", post(create_secret))
        .with_state(pool);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

type ConnectionPool = Pool<RedisConnectionManager>;

struct DatabaseConnection(PooledConnection<'static, RedisConnectionManager>);

#[async_trait]
impl<S> FromRequestParts<S> for DatabaseConnection
where
    ConnectionPool: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(_parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let pool = ConnectionPool::from_ref(state);

        let conn = pool.get_owned().await.map_err(internal_error)?;

        Ok(Self(conn))
    }
}

async fn root() -> &'static str {
    "Hello, World!"
}

async fn get_secret(Path(secret_id): Path<Uuid>,
        DatabaseConnection(mut conn): DatabaseConnection,) -> (StatusCode, Json<Secret>) {
    let secret_content = conn.get_del(secret_id.to_string()).await.map_err(not_found).unwrap();
    let secret = Secret {
        id: secret_id,
        secret_content: secret_content
    };

    (StatusCode::OK, Json(secret))
}

async fn create_secret(DatabaseConnection(mut conn): DatabaseConnection,
    Json(payload): Json<CreateSecretPayload>,) -> (StatusCode, Json<Secret>) {
    let secret = Secret {
        id: Uuid::new_v4(),
        secret_content: payload.secret_content
    };

    let _ : () = conn.set(secret.id.to_string(), &secret.secret_content).await.unwrap();
    let _ : () = conn.expire(secret.id.to_string(), 86400).await.unwrap();
    (StatusCode::CREATED, Json(secret))
}


#[derive(Deserialize)]
struct CreateSecretPayload {
    secret_content: String
}

#[derive(Serialize)]
struct Secret {
    id: Uuid,
    secret_content: String
}

/// Utility function for mapping any error into a `500 Internal Server Error`
/// response.
fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}

/// Utility function for mapping Redis get errors to a '404 Not Found Error'
/// response.
fn not_found<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::NOT_FOUND, String::from("The secret you are looking for could not be located"))
}
