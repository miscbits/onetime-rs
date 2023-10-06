use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize)]
pub struct Secret {
    pub id: Uuid,
    pub secret_content: String
}
