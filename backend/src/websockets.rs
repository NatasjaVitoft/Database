use mongodb::{bson::Bson, Client as MongoClient, bson::{doc}};
use futures_util::{SinkExt, StreamExt};
use redis::aio::PubSub;
use redis::IntoConnectionInfo;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::watch;
use tokio::select;
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_tungstenite::accept_async;
use serde::{Serialize, Deserialize};
use mongodb::bson::oid::ObjectId;

// DOCUMENT

#[derive(Debug, Serialize, Deserialize)]
struct Document {
    #[serde(rename = "_id")]
    pub id: Option<ObjectId>,
    pub title: String,
    pub content: String, 
    pub format: String,
}

// ################################################################################################################################################################

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let arg_matches = clap::command!()
        .args(&[
            clap::Arg::new("redis_addr")
                .default_value("localhost:6379")
                .help("Redis server address")
                .long("redis")
                .value_name("URL"),

            clap::Arg::new("redis_channel")
                .help("Redis PubSub channel")
                .required(true)
                .value_name("CHANNEL"),

            clap::Arg::new("ws_addr")
                .default_value("localhost:8080")
                .help("WebSocket server address")
                .long("ws")
                .value_name("URL"),
        ])
        .get_matches();

    let (sender, receiver) = watch::channel("".to_string());

    // CONNECTION MONGO DB
    let mongo_client = MongoClient::with_uri_str("mongodb://localhost:27017").await?;
    let db = mongo_client.database("pdfunited");
    let collection = db.collection::<Document>("pdfunited_collection");

    // CONNECTION REDIS
    let addr = arg_matches.get_one::<String>("redis_addr").unwrap().to_owned();
    let connection_info = format!("redis://{addr}").into_connection_info()?;
    let channel = arg_matches.get_one::<String>("redis_channel").unwrap();
    let client = redis::Client::open(connection_info).unwrap();
    let mut pubsub = client.get_async_connection().await?.into_pubsub();
    pubsub.subscribe(channel).await?;
    tokio::spawn(handle_redis_pubsub(pubsub, sender));
    println!("Subscribed to Redis Pub/Sub channel: {channel}");

    // WEBSOCKETS CONNECTION
    let addr = arg_matches.get_one::<String>("ws_addr").unwrap();
    let listener = TcpListener::bind(&addr).await?;
    println!("WebSocket server listening on: {addr}");

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(handle_websocket_connection(stream, receiver.clone(), collection.clone()));
    }

    Ok(())
}

async fn handle_redis_pubsub(mut pubsub: PubSub, sender: watch::Sender<String>) {
    while let Some(message) = pubsub.on_message().next().await {
        match message.get_payload::<String>() {
            Ok(value) => {
                sender.send_replace(value);
            },
            Err(e) => {
                eprintln!("Error getting Redis pubsub payload: {e}");
            },
        }
    }
}

async fn handle_websocket_connection(
    stream: TcpStream,
    mut receiver: watch::Receiver<String>,
    collection: mongodb::Collection<Document> 
) {
    let addr = stream.peer_addr().expect("connected stream lacks peer address");

    let (mut ws_sender, mut ws_receiver) = accept_async(stream)
        .await
        .expect("WebSocket handshake failed")
        .split();

    println!("New WebSocket connection: {addr}");

    let client = redis::Client::open("redis://localhost").unwrap();
    let mut redis_conn = client.get_async_connection().await.unwrap();

    loop {
        select! {
            _ = receiver.changed() => {
                let msg = receiver.borrow_and_update().clone();
                if let Err(e) = ws_sender.send(Message::Text(msg)).await {
                    eprintln!("Send error: {e}");
                    break;
                }
            }

            msg = ws_receiver.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        // Try to parse the ObjectId
                        match ObjectId::parse_str(text.trim()) {
                            Ok(object_id) => {
                                // Fetch content from MongoDB
                                match collection.find_one(doc! {"_id": object_id}, None).await.unwrap() {
                                    Some(doc) => {
                                        let content = &doc.content;  // Accessing the content directly from the Document struct
                                        println!("Fetched content: {content}");

                                        // Publish the content to the Redis channel
                                        let _: () = redis::cmd("PUBLISH")
                                            .arg(object_id.to_string()) // Redis channel will be the ObjectId string
                                            .arg(content)
                                            .query_async(&mut redis_conn)
                                            .await
                                            .unwrap();

                                            println!("Published to Redis: Channel: {} Content: {}", object_id, content);
                                    },
                                    None => {
                                        eprintln!("No document found with ObjectId: {}", object_id);
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!("Error parsing ObjectId: {e}");
                            }
                        }
                    }
                    Some(Ok(_)) => {}
                    Some(Err(e)) => {
                        eprintln!("WebSocket error: {e}");
                        break;
                    }
                    None => break,
                }
            }
        }
    }

    println!("{addr} disconnected");
}
