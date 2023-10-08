use uuid::Uuid;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct CreateSecretPayload {
    pub secret_content: String,
    pub password: String
}

#[derive(Deserialize, Serialize)]
pub struct CreateSecretResponse {
    pub id: Uuid,
    pub url: String
}

#[derive(Deserialize, Serialize)]
pub struct QueryPassword {
    pub password: String,
}
