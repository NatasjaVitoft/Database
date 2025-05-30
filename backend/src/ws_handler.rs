use std::error::Error;

use crate::*;
use axum::extract::ws::{Message, WebSocket};
use axum::extract::{Query, WebSocketUpgrade};
use axum::response::IntoResponse;
use axum::{extract::State, http::StatusCode};
use futures_util::{SinkExt, StreamExt};
use mongodb::bson::doc;
use serde::Deserialize;

// Redis
use redis::AsyncCommands;
use redis::aio::ConnectionManager;

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Query(params): Query<WsParams>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let role_opt = user_has_access(&params.user_email, &params.document_id, &state).await;

    if let Some(role) = role_opt {
        return ws.on_upgrade(move |socket| handle_socket(socket, params, state, role));
    }

    println!(
        "Refused access to user: {} on doc: {}",
        &params.user_email, &params.document_id
    );
    (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()).into_response()
}

async fn handle_socket(
    mut socket: WebSocket,
    params: WsParams,
    state: AppState,
    role: String,
) {
    println!(
        "WebSocket opened for user {} on doc {} with role: {}",
        params.user_email, params.document_id, role
    );

    let doc_key = format!("doc:{}", params.document_id);
    let channel = format!("channel:{}", params.document_id);

    let mut conn = state
        .redis_client
        .get_tokio_connection_manager()
        .await
        .expect("Failed to connect to Redis");

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

    let content: String = conn.get(&doc_key).await.unwrap_or_default();

    match socket.send(Message::Text(content.clone())).await {
        Ok(_) => {
            let mut map = state.ws_connections.lock().await;
            *map.entry(params.document_id.clone()).or_insert(0) += 1;
        }
        Err(e) => {
            eprintln!("Error while sending content to client: {e:?}");
            return;
        }
    }

    let mut pubsub_conn = state
        .redis_client
        .get_async_connection()
        .await
        .unwrap()
        .into_pubsub();
    pubsub_conn.subscribe(&channel).await.unwrap();

    let (mut sender, mut receiver) = socket.split();

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

    let doc_key_close = doc_key.clone();
    let doc_id = &params.document_id.clone();
    let state_close = state.clone();

    if role == "owner" || role == "editor" {
        let ws_to_redis = tokio::spawn(async move {
            while let Some(Ok(msg)) = receiver.next().await {
                match msg {
                    Message::Text(text) => {
                        let _: () = conn.set(&doc_key, &text).await.unwrap();
                        let _: () = conn.publish(&channel, &text).await.unwrap();
                    }
                    Message::Close(frame) => {
                        println!("Close connection received: {:?}", frame);
                        return;
                    }
                    _ => {}
                }
            }
        });

        let _ = ws_to_redis.await;
    } else {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Close(frame) = msg {
                println!("Close connection received from reader: {:?}", frame);
                break;
            }
        }
    }

    redis_to_ws.abort();

    let mut map = state_close.ws_connections.lock().await;
    println!("Got map");

    match map.get_mut(doc_id) {
        Some(count) => {
            *count -= 1;
            println!("count after decrement: {}", count);
            if *count <= 0 {
                println!("No more clients connected to: {}", &doc_key_close);

                if let Ok(_) = flush_mongo(&state_close, doc_id, &doc_key_close, &mut conn_close)
                    .await
                    .map_err(|e| {
                        eprintln!(
                            "Error on close flush doc with id: {} Error: {}",
                            doc_id, e
                        );
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

async fn user_has_access(email: &String, doc_id: &String, state: &AppState) -> Option<String> {
    let user = sqlx::query!(
        r#"
        SELECT user_role
        FROM document_relation
        WHERE user_email = $1
        AND document_id = $2
        UNION
        SELECT group_role FROM groups AS user_role
        NATURAL JOIN document_relation_group
        NATURAL JOIN group_members
        WHERE document_id = $2
        AND member_email = $1
        ORDER BY user_role
        LIMIT 1;
        "#,
        email,
        doc_id,
    )
    .fetch_optional(&state.pg_pool)
    .await
    .ok()??;

    Some(user.user_role.unwrap())
}

#[derive(Debug, Deserialize)]
pub struct WsParams {
    pub user_email: String,
    pub document_id: String,
}
