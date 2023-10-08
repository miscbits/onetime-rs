use crate::payload::CreateSecretResponse;
use crate::payload::CreateSecretPayload;
use crate::payload::QueryPassword;
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
    extract::{Path, FromRequestParts, FromRef, Query},
};

use aes_gcm_siv::{
    aead::{Aead, KeyInit},
    Aes256GcmSiv, Nonce
};
extern crate argon2;
use rand::distributions::{Alphanumeric, DistString};

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

pub async fn get_secret(Path(secret_id): Path<Uuid>, DatabaseConnection(mut conn): DatabaseConnection,
                 payload: Query<QueryPassword>,
        ) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let query_response: Result<Vec<u8>, RedisError> = conn.get(secret_id.to_string()).await;
    // let secret_content = conn.get_del(secret_id.to_string()).await.map_err(not_found).unwrap();
    match query_response {
        Ok(encrypted_secret_content) => {
            let hashed_password = hash_password(payload.password.clone());
            let cipher = Aes256GcmSiv::new((&hashed_password).into());

            let nonce_string: String = conn.get(secret_id.to_string() + "_nonce").await.unwrap();
            let nonce = Nonce::from_slice(nonce_string.as_bytes());

            let secret_content = cipher.decrypt(nonce, encrypted_secret_content.as_ref()).unwrap();

            let secret = Secret {
                id: secret_id,
                secret_content: String::from_utf8(secret_content).unwrap()
            };
            let _ : () = conn.del(secret_id.to_string()).await.unwrap();
            Ok((StatusCode::OK, Json(secret)))
        },
        Err(_) => Err((StatusCode::NOT_FOUND, Json(serde_json::Value::String(String::from("The secret you are looking for could not be located")))))
    }

}

pub async fn create_secret(DatabaseConnection(mut conn): DatabaseConnection,
    Json(payload): Json<CreateSecretPayload>,) -> (StatusCode, Json<CreateSecretResponse>) {

    let hashed_password = hash_password(payload.password);
    let cipher = Aes256GcmSiv::new((&hashed_password).into());
    let random_nonce_string = Alphanumeric.sample_string(&mut rand::thread_rng(), 12);
    let nonce = Nonce::from_slice(random_nonce_string.as_bytes());

    let ciphertext = cipher.encrypt(nonce, payload.secret_content.as_ref()).unwrap();

    let secret = CreateSecretResponse {
        id: Uuid::new_v4(),
        url: String::from("path/to/uuid/url")
    };

    let _ : () = conn.set(secret.id.to_string() + "_nonce", random_nonce_string).await.unwrap();    
    let _ : () = conn.set(secret.id.to_string(), ciphertext).await.unwrap();
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

fn argon2_config<'a>() -> argon2::Config<'a> {
    return argon2::Config {
        variant: argon2::Variant::Argon2id,
        hash_length: 32,
        lanes: 8,
        mem_cost: 16 * 1024,
        time_cost: 8,
        ..Default::default()
    };
}

fn hash_password(password: String) -> [u8; 32]  {
    let salt = b"randomsalt";
    let argon2_config = argon2_config();

    pop(&argon2::hash_raw(password.as_bytes(), salt, &argon2_config).unwrap())
}

fn pop(v: &[u8]) -> [u8; 32]{
    v.try_into().expect("slice with incorrect length")
}

