use lapin::{
    options::{QueueBindOptions, QueueDeclareOptions, BasicPublishOptions},
    types::FieldTable,
    BasicProperties, Connection, ConnectionProperties, ExchangeKind, Channel,
};
use serde_json::Value;


const EXCHANGE_NAME: &str = "blog_fanout_exchange";
const QUEUE_NAME: &str = "blog_queue";

pub struct EventProducer {
    addr: String,
    connection: Connection,
    channel: Channel,
}
impl EventProducer {
    pub async fn initialize(queue_addr: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let connection = Connection::connect(queue_addr, ConnectionProperties::default()).await?;
        let channel = connection.create_channel().await?;

        channel.exchange_declare(
            EXCHANGE_NAME,
            ExchangeKind::Fanout,
            Default::default(),
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

        Ok(EventProducer { addr: queue_addr.to_string(), connection, channel })

    }

    pub async fn publish(&mut self, event: Value) -> Result<(), Box<dyn std::error::Error>> {
        let payload = serde_json::to_vec(&event)?;

        let result = self.channel.basic_publish(
            EXCHANGE_NAME,
            "",
            BasicPublishOptions::default(),
            &payload,
            BasicProperties::default(),
        ).await;

        if let Err(_) = result {
            // Attempt to reconnect and retry
            println!("Reconnecting to RabbitMQ...");
            let reconnected = Self::initialize(&self.addr).await?;
            self.connection = reconnected.connection;
            self.channel = reconnected.channel;

            self.channel.basic_publish(
                EXCHANGE_NAME,
                "",
                BasicPublishOptions::default(),
                &payload,
                BasicProperties::default(),
            ).await?;
        }

        println!("Event published to RabbitMQ");

        Ok(())
        
    }

}