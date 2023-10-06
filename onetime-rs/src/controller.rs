use crate::payload::CreateSecretPayload;
use crate::model::Secret;
use crate::bb8::PooledConnection;
use crate::bb8::Pool;

use uuid::Uuid;

use bb8_redis::{
    redis::{AsyncCommands, RedisError},
    RedisConnectionManager
};

use axum::{
    async_trait,
    http::{request::Parts, StatusCode},
    response::IntoResponse,
    Json,
    extract::{Path, FromRequestParts, FromRef},
};

type ConnectionPool = Pool<RedisConnectionManager>;

pub struct DatabaseConnection(PooledConnection<'static, RedisConnectionManager>);

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

pub async fn get_secret(Path(secret_id): Path<Uuid>,
        DatabaseConnection(mut conn): DatabaseConnection,) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let query_response: Result<String, RedisError> = conn.get_del(secret_id.to_string()).await;
    // let secret_content = conn.get_del(secret_id.to_string()).await.map_err(not_found).unwrap();
    match query_response {
        Ok(secret_content) => {
            let secret = Secret {
                id: secret_id,
                secret_content: secret_content
            };

            Ok((StatusCode::OK, Json(secret)))
        },
        Err(_) => Err((StatusCode::NOT_FOUND, Json(serde_json::Value::String(String::from("The secret you are looking for could not be located")))))
    }

}

pub async fn create_secret(DatabaseConnection(mut conn): DatabaseConnection,
    Json(payload): Json<CreateSecretPayload>,) -> (StatusCode, Json<Secret>) {
    let secret = Secret {
        id: Uuid::new_v4(),
        secret_content: payload.secret_content
    };

    let _ : () = conn.set(secret.id.to_string(), &secret.secret_content).await.unwrap();
    let _ : () = conn.expire(secret.id.to_string(), 86400).await.unwrap();
    (StatusCode::CREATED, Json(secret))
}

/// Utility function for mapping any error into a `500 Internal Server Error`
/// response.
pub fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
