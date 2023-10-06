use serde::{Deserialize};

#[derive(Deserialize)]
pub struct CreateSecretPayload {
    pub secret_content: String
}
