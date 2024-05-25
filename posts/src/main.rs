use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;
use rand::{distributions::Alphanumeric, Rng};
use reqwest::Client;
use serde_json::Value;
use serde_json::json;


#[derive(Serialize, Deserialize)]
struct Post {
    id: String,
    title: String,
}

#[derive(Deserialize)]
struct CreatePostRequest {
    title: String,
}

struct AppState {
    posts: RwLock<HashMap<String, Post>>,
    client: Client,
}

#[post("/posts/create")]
async fn create_post(data: web::Data<AppState>, item: web::Json<CreatePostRequest>) -> impl Responder {
    let mut posts = match data.posts.write() {
        Ok(lock) => lock,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to writex data"),
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

    // Post an event to events bus
    let event = serde_json::json!({
        "type": "PostCreated",
        "data": {
            "id": id,
            "title": item.title
        }
    });

    let client = &data.client;
    let _ = client.post("http://rst-events-srv:8085/events")
        .json(&event)
        .send()
        .await
        .map_err(|err| eprintln!("Error sending event: {}", err));

    HttpResponse::Created().json(&posts[&id])
}

#[post("/events")]
async fn receive_event(item: web::Json<Value>) -> impl Responder {
    println!("Received Event: {:?}", item);
    HttpResponse::Ok().json(json!({"status": "received"}))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Listening at 8080");

    let app_data = web::Data::new(AppState {
        posts: RwLock::new(HashMap::new()),
        client: Client::new(),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(app_data.clone())
            .service(create_post)
            .service(receive_event)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}