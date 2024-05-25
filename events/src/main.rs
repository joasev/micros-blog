use actix_web::{post, web, App, HttpResponse, HttpServer, Responder};
use reqwest::Client;
use serde_json::Value;
use serde_json::json;

#[post("/events")]
async fn forward_event(event: web::Json<Value>, client: web::Data<Client>) -> impl Responder {
    let event = event.into_inner();
    println!("Event bus: {:?}", event);

    let urls = vec![
        "http://rst-posts-srv:8080/events",
        "http://rst-comments-srv:8081/events",
        "http://rst-query-srv:8082/events",
    ];

    // Send the event to each endpoint
    for url in urls.iter() {
        let _ = client.post(*url)
            .json(&event)
            .send()
            .await
            .map_err(|err| eprintln!("Error posting event: {}", err));
    }

    HttpResponse::Ok().json(json!({"status": "OK"}))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Listening at 8085");
    
    let client = web::Data::new(Client::new());

    HttpServer::new(move || {
        App::new()
            .app_data(client.clone())
            .service(forward_event)
    })
    .bind("0.0.0.0:8085")?
    .run()
    .await
}
