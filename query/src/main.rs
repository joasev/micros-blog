use actix_web::{web, App, HttpServer};
use mongodb::{options::ClientOptions, Client, Database};

mod api;
mod models;
mod consumer;


struct AppState {
    db_query: Database,
}


#[tokio::main]
async fn main() -> std::io::Result<()> {

    // Initialize MongoDB client
    let client_options = ClientOptions::parse("mongodb://query-mongo-srv:27017").await.map_err(|e| {
        eprintln!("Failed to parse MongoDB URI: {}", e);
        std::io::Error::new(std::io::ErrorKind::Other, "Failed to parse MongoDB URI")
    })?;
    let client = Client::with_options(client_options).map_err(|e| {
        eprintln!("Failed to initialize MongoDB client: {}", e);
        std::io::Error::new(std::io::ErrorKind::Other, "Failed to initialize MongoDB client")
    })?;
    let db_query = client.database("query");
        

    let data = web::Data::new(AppState {
        db_query,
    });

    // Spawn the RabbitMQ consumer in a separate task
    let consumer_data_handle = data.clone();
    tokio::spawn(async move {
        if let Err(e) = consumer::run("amqp://rabbitmq-srv:5672/%2f", consumer_data_handle).await {
            eprintln!("RabbitMQ consumer failed: {}", e);
        }
    });

    println!("Listening at 8082");
    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .service(api::posts::get_posts)
    })
    .bind("0.0.0.0:8082")?
    .run()
    .await
    
}