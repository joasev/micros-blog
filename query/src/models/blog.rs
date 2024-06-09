use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Post {
    pub id: String,
    pub title: String,
    pub comments: Vec<Comment>,
}
impl Unpin for Post {}

#[derive(Serialize, Deserialize, Debug)]
pub struct Comment {
    pub id: String,
    pub content: String,
}