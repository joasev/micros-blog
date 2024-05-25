use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;
use rand::{distributions::Alphanumeric, Rng};
use reqwest::Client;
use serde_json::Value;
use serde_json::json;


#[derive(Serialize, Deserialize)]
struct Comment {
    id: String,
    content: String,
}

#[derive(Deserialize)]
struct CreateCommentRequest {
    content: String,
}

struct AppState {
    comments_by_post_id: RwLock<HashMap<String, Vec<Comment>>>,
    client: Client,
}

#[get("/posts/{id}/comments")]
async fn get_comments(data: web::Data<AppState>, path: web::Path<(String,)>) -> impl Responder {
    let comments_by_post_id = match data.comments_by_post_id.read() {
        Ok(lock) => lock,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to get data"),
    };
    let empty_vec = vec![];
    let comments = comments_by_post_id.get(&path.0).unwrap_or(&empty_vec);
    HttpResponse::Ok().json(comments)
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

    let client = &data.client;
    let _ = client.post("http://rst-events-srv:8085/events")
        .json(&event)
        .send()
        .await
        .map_err(|err| eprintln!("Error sending event: {}", err));

    HttpResponse::Created().json(comments)
}

#[post("/events")]
async fn receive_event(item: web::Json<Value>) -> impl Responder {
    println!("Received Event: {:?}", item);
    HttpResponse::Ok().json(json!({"status": "received"}))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Listening at 8081");

    let app_data = web::Data::new(AppState {
        comments_by_post_id: RwLock::new(HashMap::new()),
        client: Client::new(),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(app_data.clone())
            .service(get_comments)
            .service(create_comment)
            .service(receive_event)
    })
    .bind(("0.0.0.0", 8081))?
    .run()
    .await
}