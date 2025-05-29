use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use std::time::Duration;

use axum::routing::get;
use axum::{Json, Router, extract::State, http::StatusCode, routing::post};

use mongodb::bson::doc;
use redis::AsyncCommands;
use redis::Client;
use redis::aio::ConnectionManager;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{PgPool, postgres::PgPoolOptions};
use sqlx::{Postgres, Row, Transaction};
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio::time;
use tower_http::cors::{Any, CorsLayer};

// MongoDB
use mongodb::bson::oid::ObjectId;
use mongodb::{Client as MongoClient, Database as MongoDatabase};

// For counting connections across threads
pub type WSConnections = Arc<Mutex<HashMap<String, usize>>>;

mod ws_handler;

#[tokio::main]
async fn main() {
    // Load environment variables from .env file
    dotenvy::dotenv().expect("Enviroment file doesn not exist");

    // Get the server address and database URL from environment variables
    let server_address = std::env::var("SERVER_ADDRESS").unwrap_or("127.0.0.1:3000".to_owned());
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not found in env file");

    // MongoDB connection string
    let mongo_connection_string = std::env::var("MONGO_CONNECTION_STRING")
        .expect("MONGO_CONNECTION_STRING not found in env file");

    // Allow any cors origin policy
    let cors = CorsLayer::new().allow_origin(Any).allow_headers(Any);

    // Create a connection pool to the PostgreSQL database
    let db_pool = PgPoolOptions::new()
        .max_connections(64)
        .acquire_timeout(Duration::from_secs(5))
        .connect(&database_url)
        .await
        .expect("Cannot connect to database");

    // Create a MongoDB client
    let mongo_client = MongoClient::with_uri_str(&mongo_connection_string)
        .await
        .expect("Failed to connect to MongoDB");

    // MongoDB database name
    let mongo_db_name =
        std::env::var("MONGO_DB_NAME").expect("MONGO_DB_NAME not found in env file");

    // Redis client
    let redis_conn_str =
        std::env::var("REDIS_CONNECTION_STRING").unwrap_or(String::from("redis://localhost:6379"));
    let redis_client = redis::Client::open(redis_conn_str).expect("Failed to create Redis client");

    let state = AppState {
        pg_pool: db_pool,
        mongo_db: mongo_client.database(&mongo_db_name),
        redis_client,
        ws_connections: Arc::new(Mutex::new(HashMap::new())),
    };

    let listener = TcpListener::bind(server_address)
        .await
        .expect("Could not create tcp listener");

    // Print the address the server is listening on
    println!("listening on {}", listener.local_addr().unwrap());

    let flush_timer_state = state.clone();

    // Creating the Axum router and add the needed routes
    let app = Router::new()
        .route("/login", post(login_user))
        .route("/save_document", post(save_document))
        .route("/ws", get(ws_handler::ws_handler))
        .route(
            "/save_document_and_relations",
            post(save_document_and_relations),
        )
        .route("/get_all_documents_owner", post(get_all_documents_owner))
        .route("/get_all_documents_shared", post(get_all_documents_shared))
        .route("/create_group", post(create_groups))
        .route("/get_groups_by_owner", post(get_groups_by_owner))
        .layer(cors)
        .with_state(state);

    // Periodic persistence and redis housekeeping
    start_periodic_flush(flush_timer_state).await;

    // Serving the application using the listener
    axum::serve(listener, app)
        .await
        .expect("Error serving application");
}

// ***************************************************************************************************************************************

pub async fn start_periodic_flush(state: AppState) {
    if let Ok(mut conn) = state.redis_client.get_tokio_connection_manager().await {
        tokio::spawn(async move {
            loop {
                time::sleep(time::Duration::from_secs(10)).await;

                match flush_mongo_all(&state, &mut conn).await {
                    Ok(_) => println!("Periodic flush successful"),
                    Err(e) => eprintln!("Periodic flush failed: {}", e),
                }
            }
        });
    }
}

// TODO: TEST
async fn flush_mongo_all(
    state: &AppState,
    conn: &mut ConnectionManager,
) -> Result<(), Box<dyn Error>> {
    let doc_keys: Vec<String> = conn.keys(String::from('*')).await?;
    println!("Flush timer: Keys read: {:?}", doc_keys);

    for key in doc_keys {
        let content: String = conn.get(&key).await?; // ERROR

        let id_str = key.replace("doc:", "");
        let obj_id = ObjectId::parse_str(&id_str)?; // ERROR
        let filter = doc! { "_id": obj_id };
        let cont = doc! { "$set": {"content": content}};

        state
            .mongo_db
            .collection::<Document>("documents")
            .find_one_and_update(filter, cont, None)
            .await?;

        let map = state.ws_connections.lock().await;

        match map.get(&id_str) {
            Some(count) => {
                if *count <= 0 {
                    let _: () = conn.del(key).await?;
                }
            }
            None => {
                eprintln!("Failed to find doc id in ws_connections: {}\nCleaning up", id_str);
                let _: () = conn.del(key).await?;
            }
        }
    }
    Ok(())
}

// This function handles the Login endpoint/request

async fn login_user(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<(StatusCode, String), (StatusCode, String)> {
    let user = sqlx::query_as!(
        UserRow,
        "SELECT email, first_name, last_name FROM users WHERE email = $1 AND password = $2",
        payload.email,
        payload.password
    )
    .fetch_optional(&state.pg_pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            json!({"success": false, "message": e.to_string()}).to_string(),
        )
    })?;

    if let Some(user) = user {
        Ok((
            StatusCode::OK,
            json!({"success": true, "user": user}).to_string(),
        ))
    } else {
        Ok((
            StatusCode::UNAUTHORIZED,
            json!({"success": false}).to_string(),
        ))
    }
}

// This function handles the saving of a document and its relations in both MongoDB and PostgreSQL

async fn save_document_and_relations(
    State(state): State<AppState>,
    Json(payload): Json<DocumentCreateRequest>,
) -> Result<(StatusCode, String), (StatusCode, String)> {

    if payload.title.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            json!({ "success": false, "message": "Title must not be empty" }).to_string(),
        ));
    }    
    if payload.format.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            json!({ "success": false, "message": "Format must not be empty" }).to_string(),
        ));
    }


    let collection = state.mongo_db.collection::<Document>("documents");

    let document = Document {
        id: None,
        title: payload.title,
        content: String::new(),
        format: payload.format,
    };

    let insert_result = collection.insert_one(document, None).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            json!({"success": false, "message": e.to_string()}).to_string(),
        )
    })?;

    let document_id = insert_result.inserted_id.as_object_id().ok_or((
        StatusCode::INTERNAL_SERVER_ERROR,
        json!({"success": false, "message": "Failed to extract ObjectId"}).to_string(),
    ))?;

    sqlx::query!(
        "INSERT INTO document_relation (user_email, document_id, user_role) VALUES ($1, $2, $3)",
        payload.owner,
        document_id.to_string(),
        "owner" as &str, // user_role = "owner"
    )
    .execute(&state.pg_pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            json!({"success": false, "message": e.to_string()}).to_string(),
        )
    })?;

    for collaborator in payload.collaborators.iter() {
        sqlx::query!(
            "INSERT INTO document_relation (user_email, document_id, user_role) VALUES ($1, $2, $3)",
            collaborator,
            document_id.to_string(),
            "editor" as &str, // user_role = "editor"
        )
        .execute(&state.pg_pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({"success": false, "message": e.to_string()}).to_string(),
            )
        })?;
    }

    for reader in payload.readers.iter() {
        sqlx::query!(
            "INSERT INTO document_relation (user_email, document_id, user_role) VALUES ($1, $2, $3)",
            reader,
            document_id.to_string(),
            "read" as &str // user_role = "reader"
        )
        .execute(&state.pg_pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({ "success": false, "message": e.to_string() }).to_string(),
            )
        })?;
    }

    // Insert document related user groups
    for group_id in payload.groups.iter() {
        sqlx::query!(
            "INSERT INTO document_relation_group (group_id, document_id) VALUES ($1, $2)",
            group_id,
            document_id.to_string(),
        )
        .execute(&state.pg_pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({ "success": false, "message": e.to_string() }).to_string(),
            )
        })?;
    }

    Ok((StatusCode::OK, json!({ "success": true, "message": "Created" }).to_string()))
}

// This function handles the request for getting all documents belonging to the owner

async fn get_all_documents_owner(
    State(state): State<AppState>,
    Json(payload): Json<GetDocumentRequest>,
) -> Result<(StatusCode, String), (StatusCode, String)> {
    // Get document IDs and owner email from PostgreSQL
    let rows = sqlx::query!(
        "SELECT document_id, user_email FROM document_relation WHERE user_email = $1 AND user_role = $2",
        payload.email,
        "owner"
    )
    .fetch_all(&state.pg_pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            json!({ "success": false, "message": e.to_string() }).to_string(),
        )
    })?;

    let document_ids: Vec<(String, String)> = rows
        .iter()
        .map(|row| (row.document_id.clone(), row.user_email.clone()))
        .collect();

    let collection = state.mongo_db.collection::<Document>("documents");
    let mut documents = Vec::new();

    for (document_id, owner_email) in document_ids {
        let obj_id_result = mongodb::bson::oid::ObjectId::parse_str(&document_id);
        if obj_id_result.is_err() {
            continue;
        }
        let obj_id = obj_id_result.unwrap();

        let filter = mongodb::bson::Document::from_iter(vec![(String::from("_id"), obj_id.into())]);
        let document_result = collection.find_one(filter, None).await;

        if let Ok(Some(doc)) = document_result {
            documents.push(serde_json::json!({
                "id": document_id,
                "title": doc.title,
                "format": doc.format,
                "owner_email": owner_email,
            }));
        }
    }
    Ok((
        StatusCode::OK,
        serde_json::json!({ "success": true, "documents": documents }).to_string(),
    ))
}

// This function handles the request for getting all documents shared with the user signed in

async fn get_all_documents_shared(
    State(state): State<AppState>,
    Json(payload): Json<GetDocumentRequest>,
) -> Result<(StatusCode, String), (StatusCode, String)> {
    let relation_rows = sqlx::query!(
        "SELECT document_id FROM document_relation WHERE user_email = $1 AND user_role IN ($2, $3)",
        payload.email,
        "editor",
        "read"
    )
    .fetch_all(&state.pg_pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            json!({ "success": false, "message": e.to_string() }).to_string(),
        )
    })?;

    // get document id's from share groups as well
    let group_rows = sqlx::query!(
        "SELECT document_id FROM group_members NATURAL JOIN document_relation_group WHERE member_email = $1",
        payload.email
    )
    .fetch_all(&state.pg_pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            json!({ "success": false, "message": e.to_string() }).to_string(),
        )
    })?;

    let relation_ids: Vec<String> = relation_rows
        .iter()
        .map(|row| row.document_id.clone())
        .collect();
    let group_document_ids: Vec<String> = group_rows
        .iter()
        .map(|row| row.document_id.clone())
        .collect();

    let document_ids = [relation_ids, group_document_ids].concat();

    if document_ids.is_empty() {
        return Ok((
            StatusCode::OK,
            json!({ "success": true, "documents": [] }).to_string(),
        ));
    }

    let mut documents = Vec::new();
    let collection = state.mongo_db.collection::<Document>("documents");

    for doc_id in document_ids {
        let owner_row = sqlx::query!(
            "SELECT user_email FROM document_relation WHERE document_id = $1 AND user_role = $2",
            doc_id,
            "owner"
        )
        .fetch_optional(&state.pg_pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({ "success": false, "message": e.to_string() }).to_string(),
            )
        })?;

        let owner_email = match owner_row {
            Some(row) => row.user_email,
            None => continue,
        };

        let obj_id = match mongodb::bson::oid::ObjectId::parse_str(&doc_id) {
            Ok(oid) => oid,
            Err(_) => continue,
        };

        let filter = mongodb::bson::doc! { "_id": obj_id };

        let mongo_doc = collection.find_one(filter, None).await.map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({ "success": false, "message": e.to_string() }).to_string(),
            )
        })?;

        if let Some(doc) = mongo_doc {
            documents.push(json!({
                "id": doc_id,
                "title": doc.title,
                "format": doc.format,
                "owner_email": owner_email,
            }));
        }
    }

    Ok((
        StatusCode::OK,
        json!({ "success": true, "documents": documents }).to_string(),
    ))
}

// ADD GROUPS INTO POSTGRES TABLE

pub async fn create_groups(
    State(state): State<AppState>,
    Json(payload): Json<Vec<GroupsRequest>>,
) -> Result<(StatusCode, String), (StatusCode, String)> {
    for group in payload {

        // Check for malformed requests
        if group.name.is_empty() {
            return Err((
                StatusCode::BAD_REQUEST,
                json!({ "success": false, "message": "Group name must not be empty" }).to_string(),
            ));
        }    

        if group.members.is_empty() {
            return Err((
                StatusCode::BAD_REQUEST,
                json!({ "success": false, "message": "Members must not be empty" }).to_string(),
            ));
        }    


        // Start transaction
        let mut tx: Transaction<'_, Postgres> = state.pg_pool.begin().await.map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({ "success": false, "message": format!("Failed to start transaction: {}", e) }).to_string(),
            )
        })?;

        // Insert group
        let query_groups = sqlx::query(
            "INSERT INTO groups (group_name, owner_email, group_role) VALUES ($1, $2, $3) RETURNING group_id"
        )
        .bind(&group.name)       // changed from group.group_name to group.name
        .bind(&group.owner)      // unchanged, matches struct
        .bind(&group.role)       // changed from group.group_role to group.role
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({ "success": false, "message": format!("Error inserting group: {}", e) }).to_string(),
            )
        })?;

        let group_id: i32 = query_groups.try_get("group_id").map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({ "success": false, "message": format!("Failed to get group_id: {}", e) })
                    .to_string(),
            )
        })?;

        // Insert each member
        for member_email in group.members.iter() {
            sqlx::query(
                "INSERT INTO group_members (group_id, member_email) VALUES ($1, $2)"
            )
            .bind(group_id)
            .bind(member_email)
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({ "success": false, "message": format!("Error inserting group member: {}", e) }).to_string(),
                )
            })?;
        }

        tx.commit().await.map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({ "success": false, "message": format!("Failed to commit transaction: {}", e) }).to_string(),
            )
        })?;
    }

    Ok((
        StatusCode::CREATED,
        json!({ "success": true, "message": "Group created successfully" }).to_string(),
    ))
}

async fn get_groups_by_owner(
    State(state): State<AppState>,
    Json(payload): Json<GetDocumentRequest>,
) -> Result<(StatusCode, String), (StatusCode, String)> {
    let rows = sqlx::query!(
        "SELECT group_id, group_name, owner_email, group_role FROM groups WHERE owner_email = $1",
        payload.email
    )
    .fetch_all(&state.pg_pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            json!({ "success": false, "message": e.to_string() }).to_string(),
        )
    })?;

    let groups: Vec<_> = rows
        .into_iter()
        .map(|row| {
            json!({
                "group_name": row.group_name,
                "owner_email": row.owner_email,
                "group_role": row.group_role,
                "group_id": row.group_id,
            })
        })
        .collect();

    Ok((
        StatusCode::OK,
        json!({ "success": true, "groups": groups }).to_string(),
    ))
}

// ***************************************************************************************************************************************
// This function handles the MongoDB document creation

async fn save_document(
    State(state): State<AppState>,
    Json(mut payload): Json<Document>,
) -> Result<(StatusCode, String), (StatusCode, String)> {
    let collection = state.mongo_db.collection::<Document>("documents");

    payload.id = None;

    match collection.insert_one(payload, None).await {
        Ok(insert_result) => Ok((
            StatusCode::CREATED,
            json!({"inserted_id": insert_result.inserted_id}).to_string(),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            json!({"success": false, "message": e.to_string()}).to_string(),
        )),
    }
}

// ***************************************************************************************************************************************

// Struct for the login request
#[derive(Deserialize)]
struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Deserialize)]
pub struct GetDocumentRequest {
    pub email: String,
}

#[derive(Serialize)]
struct RelationRow {
    user_email: String,
    document_id: String,
    user_role: String,
}

// Struct for the user row returned from the database
#[derive(Serialize)]
struct UserRow {
    email: String,
    first_name: String,
    last_name: String,
}

#[derive(Clone)]
pub struct AppState {
    pg_pool: PgPool,
    mongo_db: MongoDatabase,
    redis_client: Client,
    ws_connections: Arc<Mutex<HashMap<String, usize>>>,
}

#[derive(Deserialize, Serialize, Debug)]
struct Document {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    id: Option<ObjectId>,
    title: String,
    content: String,
    format: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct DocumentCreateRequest {
    title: String,
    format: String,
    collaborators: Vec<String>,
    readers: Vec<String>,
    owner: String,
    groups: Vec<i32>,
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
