use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Comment {
    pub id: String,
    pub content: String,
}