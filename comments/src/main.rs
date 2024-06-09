use actix_web::{web, App, HttpServer};
use producer::EventProducer;
use models::comment::Comment;
use std::collections::HashMap;
use std::sync::{Mutex, RwLock};

mod api;
mod models;
mod producer;


struct AppState {
    comments_by_post_id: RwLock<HashMap<String, Vec<Comment>>>,
    producer: Mutex<EventProducer>
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    // Setup RabbitMQ
    let producer = EventProducer::initialize("amqp://rabbitmq-srv:5672/%2f").await
        .map_err(|e| {
            eprintln!("Failed to setup RabbitMQ: {}", e);
            std::io::Error::new(std::io::ErrorKind::Other, "Failed to setup RabbitMQ")
        })?;

    let app_data = web::Data::new(AppState {
        comments_by_post_id: RwLock::new(HashMap::new()),
        producer: Mutex::new(producer),
    });

    println!("Listening at 8081");
    HttpServer::new(move || {
        App::new()
            .app_data(app_data.clone())
            .service(api::comments::create_comment)
    })
    .bind(("0.0.0.0", 8081))?
    .run()
    .await
}