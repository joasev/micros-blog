use serde::Deserialize;
use actix_web::{post, web, HttpResponse, Responder};
use crate::{models::comment::Comment, AppState};
use rand::{distributions::Alphanumeric, Rng};

#[derive(Deserialize)]
struct CreateCommentRequest {
    content: String,
}


#[post("/posts/{id}/comments")]
async fn create_comment(data: web::Data<AppState>, path: web::Path<(String,)>, comment_data: web::Json<CreateCommentRequest>) -> impl Responder {
    let mut comments_by_post_id = match data.comments_by_post_id.write() {
        Ok(lock) => lock,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to write data"),
    };
    let comment_id: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(8)
        .map(char::from)
        .collect();

    let comment = Comment {
        id: comment_id.clone(),
        content: comment_data.content.clone(),
    };

    let comments = comments_by_post_id.entry(path.0.clone()).or_insert_with(Vec::new);
    comments.push(comment);

    let event = serde_json::json!({
        "type": "CommentCreated",
        "data": {
            "id": comment_id,
            "content": comment_data.content,
            "postId": path.0
        }
    });

    // Publish the event to RabbitMQ
    let publish_result = {
        match data.producer.lock() {
            Ok(mut producer) => producer.publish(event).await,
            Err(_) => Err("Failed to acquire RabbitMQ lock".into()),
        }
    };
    if let Err(e) = publish_result {
        return HttpResponse::InternalServerError().body(format!("Failed to publish event: {}", e));
    }

    HttpResponse::Created().json(comments)
}
