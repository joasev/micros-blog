use actix_web::web;
use serde::Deserialize;
use lapin::{
    options::{BasicConsumeOptions, ExchangeDeclareOptions, QueueBindOptions, QueueDeclareOptions}, types::FieldTable, Connection, ConnectionProperties, Consumer, ExchangeKind
};
use futures::StreamExt;
use mongodb::bson::{self, doc};
use crate::{models::blog::{Comment, Post}, AppState};


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

const EXCHANGE_NAME: &str = "blog_fanout_exchange";
const QUEUE_NAME: &str = "blog_queue";
const CONSUMER_TAG: &str = "blog_consumer";

pub async fn run(addr: &str, data: web::Data<AppState>) -> Result<(), Box<dyn std::error::Error>> {
    
    let mut consumer = initialize(addr).await?;

    println!("Waiting for messages...");

    while let Some(delivery) = consumer.next().await {
        let delivery = delivery?;
        println!("Received: {:?}", std::str::from_utf8(&delivery.data)?);

        // Deserialize the delivery data
        let event: Event = serde_json::from_slice(&delivery.data)?;

        // Get posts collection from mongoDB
        let posts_collection = data.db_query.collection::<Post>("posts");

        match event.event_type.as_str() {
            "PostCreated" => {
                println!("PostCreated event: {:?}", event);
                if let (id, Some(title)) = (&event.data.id, &event.data.title) {
                    let new_post = Post { id: id.clone(), title: title.clone(), comments: Vec::new() };
                    posts_collection.insert_one(&new_post, None).await?;
                }
            },
            "CommentCreated" => {
                println!("CommentCreated event: {:?}", event);
                if let (Some(post_id), id, Some(content)) = (&event.data.post_id, &event.data.id, &event.data.content) {
                    let comment = Comment { id: id.clone(), content: content.clone() };
                    let comment_bson = bson::to_bson(&comment)?;

                    if let bson::Bson::Document(comment_doc) = comment_bson {
                        posts_collection.update_one(
                            doc! { "id": post_id },
                            doc! { "$push": { "comments": comment_doc }},
                            None,
                        ).await?;
                    }
                }
            },
            _ => {}
        }

        delivery.ack(lapin::options::BasicAckOptions::default()).await?;
    }

    Ok(())

}

async fn initialize(addr: &str) -> Result<Consumer, Box<dyn std::error::Error>> { 
    let connection = Connection::connect(addr, ConnectionProperties::default()).await?;
    let channel = connection.create_channel().await?;

    channel.exchange_declare(
        EXCHANGE_NAME,
        ExchangeKind::Fanout,
        ExchangeDeclareOptions::default(),
        FieldTable::default(),
    ).await?;

    let queue = channel.queue_declare(
        QUEUE_NAME,
        QueueDeclareOptions::default(),
        FieldTable::default(),
    ).await?;

    channel.queue_bind(
        queue.name().as_str(),
        EXCHANGE_NAME,
        "",
        QueueBindOptions::default(),
        FieldTable::default(),
    ).await?;

    // Step 6: Consume messages from the queue
    let consumer = channel.basic_consume(
        queue.name().as_str(),
        CONSUMER_TAG,
        BasicConsumeOptions::default(),
        FieldTable::default(),
    ).await?;

    Ok(consumer)
}