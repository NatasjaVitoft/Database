use mongodb::{bson::oid::ObjectId};
use redis::Client;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tokio::sync::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use crate::MongoDatabase;

// Struct for the login request
#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct GetDocumentRequest {
    pub email: String,
}

// Struct for the user row returned from the database
#[derive(Serialize)]
pub struct UserRow {
    pub email: String,
    pub first_name: String,
    pub last_name: String,
}

#[derive(Clone)]
pub struct AppState {
    pub pg_pool: PgPool,
    pub mongo_db: MongoDatabase,
    pub redis_client: Client,
    pub ws_connections: Arc<Mutex<HashMap<String, usize>>>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Document {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub title: String,
    pub content: String,
    pub format: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct DocumentCreateRequest {
    pub title: String,
    pub format: String,
    pub collaborators: Vec<String>,
    pub readers: Vec<String>,
    pub owner: String,
    pub groups: Vec<i32>,
}

// STRUCT FOR GROUPS REQUEST
#[derive(Deserialize, Serialize, Debug)]
pub struct GroupsRequest {
    pub owner: String,
    pub name: String,
    pub role: String,
    pub members: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct GetGroupsRequest {
    pub email: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct GetUserRole {
    pub email: String,
    pub document_id: String,
}

#[derive(Debug, Deserialize)]
pub struct WsParams {
    pub user_email: String,
    pub document_id: String,
}