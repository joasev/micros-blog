use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;

#[derive(Serialize, Debug)]
struct Post {
    id: String,
    title: String,
    comments: Vec<Comment>,
}

#[derive(Serialize, Debug)]
struct Comment {
    id: String,
    content: String,
}

struct AppState {
    posts: RwLock<HashMap<String, Post>>,
}

#[derive(Deserialize, Debug)]
struct EventData {
    id: String,
    title: Option<String>,
    content: Option<String>,
    #[serde(rename = "postId")]
    post_id: Option<String>,
}

#[derive(Deserialize, Debug)]
struct Event {
    #[serde(rename = "type")]
    event_type: String,
    data: EventData,
}

#[get("/posts")]
async fn get_posts(data: web::Data<AppState>) -> impl Responder { 
    let posts = match data.posts.read() {
        Ok(lock) => lock,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to get data"),
    };
    HttpResponse::Ok().json(&*posts)
}

#[post("/events")]
async fn receive_event(data: web::Data<AppState>, event: web::Json<Event>) -> impl Responder {
    let mut posts = match data.posts.write() {
        Ok(lock) => lock,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to write data"),
    };
    match event.event_type.as_str() {
        "PostCreated" => {
            println!("PostCreated event: {:?}", event);
            if let (id, Some(title)) = (&event.data.id, &event.data.title) {
                let new_post = Post { id: id.clone(), title: title.clone(), comments: Vec::new() };
                posts.insert(id.clone(), new_post);
            }
        },
        "CommentCreated" => {
            println!("CommentCreated event: {:?}", event);
            if let (Some(post_id), id, Some(content)) = (&event.data.post_id, &event.data.id, &event.data.content) {
                if let Some(post) = posts.get_mut(post_id) {
                    post.comments.push(Comment { id: id.clone(), content: content.clone() });
                }
            }
        },
        _ => {}
    }

    println!("{:?}", posts);  
    HttpResponse::Ok().json({})
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Listening at 8082");

    let data = web::Data::new(AppState {
        posts: RwLock::new(HashMap::new()),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .service(get_posts)
            .service(receive_event)
    })
    .bind("0.0.0.0:8082")?
    .run()
    .await
}