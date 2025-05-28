use std::error::Error;

use crate::*;
use axum::extract::ws::{Message, WebSocket};
use axum::extract::{Query, WebSocketUpgrade};
use axum::response::IntoResponse;
use axum::{extract::State, http::StatusCode};
use futures_util::{SinkExt, StreamExt};
use mongodb::bson::doc;
use serde::Deserialize;
use tokio::time;

// Redis
use redis::AsyncCommands;
use redis::aio::ConnectionManager;

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Query(params): Query<WsParams>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    // validate user

    let access = user_has_access(&params.user_email, &params.document_id, &state).await;

    if !access {
        println!(
            "Refused access to user: {} on doc: {}",
            &params.user_email, &params.document_id
        );
        return (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()).into_response();
    }

    ws.on_upgrade(move |socket| handle_socket(socket, params, state))
}

async fn handle_socket(mut socket: WebSocket, params: WsParams, state: AppState) {
    println!(
        "WebSocket opened for user {} on doc {}",
        params.user_email, params.document_id
    );

    // Connect to Redis
    let doc_key = format!("doc:{}", params.document_id);
    let channel = format!("channel:{}", params.document_id);

    let redis_conn_str =
        std::env::var("REDIS_CONNECTION_STRING").unwrap_or(String::from("redis://localhost:6379"));

    let redis_client = redis::Client::open(redis_conn_str).expect("Failed to create Redis client");
    let mut conn = redis_client
        .get_tokio_connection_manager()
        .await
        .expect("Failed to connect to Redis");

    let mut conn_flush = conn.clone();
    let mut conn_close = conn.clone();

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
                eprintln!("No content was found with objectId: {}", object_id);
            }
        } else {
            eprintln!("Invalid ObjectId format: {}", &params.document_id);
        }
    }

    // Load latest content from Redis
    let content: String = conn.get(&doc_key).await.unwrap_or_default();

    // Send the full content to the client first
    match socket.send(Message::Text(content.clone())).await {
        Ok(_) => {
            // Count active client on success
            let mut map = state.ws_connections.lock().await;
            *map.entry(params.document_id.clone()).or_insert(0) += 1;
        }
        Err(e) => {
            eprintln!("Error while sending content to client: {e:?}");
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
                if let Err(e) = sender.send(Message::Text(payload)).await {
                    eprintln!("Failed Websocket send: {:?}", e);
                    break;
                };
            }
        }
    });

    let doc_key_flush = doc_key.clone();
    let doc_key_close = doc_key.clone();

    // listen to messages from the client and forward them to Redis
    let ws_to_redis = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => {
                    // Update Redis doc value
                    let _: () = conn.set(&doc_key, &text).await.unwrap();

                    // Publish to other clients
                    let _: () = conn.publish(&channel, &text).await.unwrap();
                }
                Message::Close(frame) => {
                    println!("Close connection received: {:?}", frame);
                    return;
                }
                // Other message types are note handled for now
                _ => {}
            }
        }
    });

    let doc_id = &params.document_id.clone();
    let state_close = &state.clone();

    let mut flush_timer: Option<tokio::task::JoinHandle<()>> = None;

    // Scope for releasing map before awaiting receiver
    { 
        let map = state_close.ws_connections.lock().await;

        // BUG: Only client that started it can close it again
        if let Some(count) = map.get(&params.document_id) {
            if *count == 1 {
                flush_timer = Some(tokio::spawn(async move {
                    println!("Started flush timer");
                    loop {
                        time::sleep(time::Duration::from_secs(10)).await;
                        let _ = flush_mongo(
                            &state,
                            &params.document_id,
                            &doc_key_flush,
                            &mut conn_flush,
                        )
                        .await
                        .map_err(|e| {
                            eprintln!(
                                "Error on timed flush doc with id: {} Error: {}",
                                &params.document_id, e
                            );
                        });
                    }
                }));
            }
        }
    }

    // Await socket receiver. Flush when client breaks connection and abort listener/sender processes
    let _ = ws_to_redis.await;

    redis_to_ws.abort();

    // Flush MongoDB and clear Redis on last client disconnect
    let mut map = state_close.ws_connections.lock().await;
    println!("Got map");

    match map.get_mut(doc_id) {
        Some(count) => {
            *count -= 1;
            println!("count after decrement: {}", count);
            if *count <= 0 {
                println!("No more clients connected to: {}", &doc_key_close);
                if let Some(flush_timer) = flush_timer {
                    flush_timer.abort();
                    println!("Aborted flush timer")
                }
                else {
                    println!("No flush timer found?")
                }

                if let Ok(_) = flush_mongo(&state_close, &doc_id, &doc_key_close, &mut conn_close)
                    .await
                    .map_err(|e| {
                        eprintln!("Error on close flush doc with id: {} Error: {}", &doc_id, e);
                    })
                {
                    println!("Close Mongo flush for {}", &doc_key_close);
                    if let Ok(()) = conn_close.del(&doc_key_close).await.map_err(|e| {
                        eprintln!("Failed to delete Redis key: {}", e);
                    }) {
                        map.remove(doc_id);
                        println!("Removed Redis key: {} ", doc_key_close);
                    }
                }
            }
        }
        None => {
            eprintln!("No ws_connections count found for id: {}", doc_id);
        }
    }

    println!(
        "WebSocket closed for user {} on doc: {}",
        params.user_email, doc_id
    );
}

async fn flush_mongo(
    state: &AppState,
    document_id: &str,
    doc_key_flush: &str,
    conn: &mut ConnectionManager,
) -> Result<(), Box<dyn Error>> {
    let content: String = conn.get(doc_key_flush).await?;
    let obj_id = ObjectId::parse_str(document_id)?;
    let filter = doc! { "_id": obj_id };
    let cont = doc! { "$set": {"content": content}};
    state
        .mongo_db
        .collection::<Document>("documents")
        .find_one_and_update(filter, cont, None)
        .await?;
    Ok(())
}

async fn user_has_access(email: &String, doc_id: &String, state: &AppState) -> bool {
    let user = sqlx::query_as!(
        RelationRow,
        "SELECT user_email, document_id, user_role FROM document_relation WHERE user_email = $1 AND document_id = $2 AND user_role = 'owner' OR user_role = 'editor'",
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
