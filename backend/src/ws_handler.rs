use crate::*;
use axum::extract::ws::{Message, WebSocket};
use axum::extract::{Query, WebSocketUpgrade};
use axum::response::IntoResponse;
use axum::{extract::State, http::StatusCode};
use futures_util::future::OrElse;
use futures_util::{SinkExt, StreamExt};
use mongodb::bson::doc;
use mongodb::{Client as MongoClient, Database as MongoDatabase};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

// Redis
use redis::AsyncCommands;

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Query(params): Query<WsParams>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    // validate user

    let access = user_has_access(&params.user_email, &params.document_id, &state).await;

    if !access {
        return (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()).into_response();
    }

    ws.on_upgrade(move |socket| handle_socket(socket, params, state))
}

async fn handle_socket(mut socket: WebSocket, params: WsParams, state: AppState) {
    println!(
        "WebSocket opened for user {} on doc {}",
        params.user_email, params.document_id
    );

    // TODO: Check if doc is already in redis store

    // Connect to Redis
    let doc_key = format!("doc:{}", params.document_id);
    let channel = format!("channel:{}", params.document_id);

    let redis_conn_str =
        std::env::var("REDIS_CONNECTION_STRING").unwrap_or(String::from("redis://localhost:6379"));

    let redis_client = redis::Client::open(redis_conn_str).expect("Failed to create Redis client");
    let mut conn = redis_client
        .get_tokio_connection()
        .await
        .expect("Failed to connect to Redis");

    let doc_exists: bool = conn.exists(&doc_key).await.unwrap_or(false);

    if !doc_exists {
        if let Ok(object_id) = ObjectId::parse_str(&params.document_id) {
            let filter = doc! { "_id": object_id };
    
            if let Ok(Some(doc)) = state
                .mongo_db
                .collection::<Document>("documents")
                .find_one(filter, None)
                .await
            {
                let content = doc.content.clone();
                println!("Some document content: {}", content);
    
                let _: () = conn.set(&doc_key, content.clone()).await.unwrap();
            } else {
                println!("No content was found with that ObjectId");
            }
        } else {
            println!("Invalid ObjectId format: {}", &params.document_id);
        }
    }

    // Load latest content from Redis
    let content: String = conn.get(&doc_key).await.unwrap_or_default();

    // Send the full content to the client first
    match socket.send(Message::Text(content.clone())).await {
        Ok(x) => x,
        Err(e) => {
            println!("Error while sending content to client: {e:?}");
            return;
        }
    }

    // Set up Pub/Sub subscription
    let mut pubsub_conn = redis_client
        .get_async_connection()
        .await
        .unwrap()
        .into_pubsub();
    pubsub_conn.subscribe(&channel).await.unwrap();

    // Start listening to both incoming WebSocket messages and Redis PubSub
    let (mut sender, mut receiver) = socket.split();

    // spawn task to receive from Redis pubsub
    let redis_to_ws = tokio::spawn(async move {
        let mut pubsub_stream = pubsub_conn.on_message();

        while let Some(msg) = pubsub_stream.next().await {
            if let Ok(payload) = msg.get_payload::<String>() {
                let _ = sender.send(Message::Text(payload)).await;
            }
        }
    });

    // You could also listen to messages from the client and forward them to Redis
    let ws_to_redis = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Text(text) = msg {
                // Update Redis doc value
                let _: () = conn.set(&doc_key, &text).await.unwrap();

                // Publish to other clients
                let _: () = conn.publish(&channel, &text).await.unwrap();
            }
        }
    });

    // Await both tasks
    let _ = tokio::join!(redis_to_ws, ws_to_redis);

    /*     while let Some(Ok(Message::Text(msg))) = socket.next().await {
        println!("Received: {}", msg);
        if socket
            .send(Message::Text(format!("Echo: {}", msg)))
            .await
            .is_err()
        {
            break;
        }
    } */
    println!("WebSocket closed");
}

async fn user_has_access(email: &String, doc_id: &String, state: &AppState) -> bool {
    let user = sqlx::query_as!(
        RelationRow,
        "SELECT user_email, document_id, user_role FROM document_relation WHERE user_email = $1 AND document_id = $2 AND user_role = 'owner'",
        email,
        doc_id,
    ).fetch_optional(&state.pg_pool)
    .await;

    matches!(user, Ok(Some(_)))
}

#[derive(Debug, Deserialize)]
pub struct WsParams {
    user_email: String,
    document_id: String,
}
