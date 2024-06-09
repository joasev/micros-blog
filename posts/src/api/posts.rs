use serde::Deserialize;
use actix_web::{post, web, HttpResponse, Responder};
use rand::{distributions::Alphanumeric, Rng};
use crate::{models::post::Post, AppState};


#[derive(Deserialize)]
struct CreatePostRequest {
    title: String,
}

#[post("/posts/create")]
async fn create_post(data: web::Data<AppState>, item: web::Json<CreatePostRequest>) -> impl Responder {
    let mut posts = match data.posts.write() {
        Ok(lock) => lock,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to write data"),
    };
    let id: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(8)
        .map(char::from)
        .collect();

    let post = Post {
        id: id.clone(),
        title: item.title.clone(),
    };

    posts.insert(id.clone(), post);

    // Post an event to the message queue
    let event = serde_json::json!({
        "type": "PostCreated",
        "data": {
            "id": id,
            "title": item.title
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

    HttpResponse::Created().json(&posts[&id])
}
