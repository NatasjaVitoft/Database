use std::time::Duration;

use axum::{
    extract::{State},
    http::StatusCode,
    routing::post,
    Json, Router,
};

use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{postgres::PgPoolOptions, PgPool};
use tokio::net::TcpListener;
use tower_http::cors::{CorsLayer, Any};

// MongoDB
use mongodb::{Client as MongoClient, Database as MongoDatabase};
use mongodb::bson::oid::ObjectId;

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
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_headers(Any);

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
    let mongo_db_name = std::env::var("MONGO_DB_NAME").expect("MONGO_DB_NAME not found in env file");

    let state = AppState {
        pg_pool: db_pool,          
        mongo_db: mongo_client.database(&mongo_db_name),
    };

    let listener = TcpListener::bind(server_address)
        .await
        .expect("Could not create tcp listener");

    // Print the address the server is listening on
    println!("listening on {}", listener.local_addr().unwrap());

    // Creating the Axum router and add the needed routes
    let app = Router::new()
        .route("/login", post(login_user))
        .route("/save_document", post(save_document))
        .route("/access_doc", post(access_doc))
        .layer(cors)
        .with_state(state);

    // Serving the application using the listener
    axum::serve(listener, app)
        .await
        .expect("Error serving application");
}

// ***************************************************************************************************************************************

// This function handles the login request
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


// ***************************************************************************************************************************************
 // This function handles the MongoDB document creation 

async fn save_document(
    // Extract the state from the request
    State(state): State<AppState>,
    Json(mut payload): Json<Document>,
) -> Result<(StatusCode, String), (StatusCode, String)> {
    let collection = state.mongo_db.collection::<Document>("documents");

    payload.id = None; // We let MongoDB generate the ID

    // Insert the document into the MongoDB collection
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
// This function handles document content

async fn access_doc(State(state): State<AppState>, Json(mut payload): Json<DocumentRequest>) -> Result<(StatusCode, String), (StatusCode, String)> {
    // validate user

    let access = user_has_access(payload.user_email, payload.document_id, &state).await;

    if !access {
        return Err((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()));
    }
    
    //  TODO: finish logic
    Ok((StatusCode::OK, "Access granted".to_string()))
}

async fn user_has_access(email: String, doc_id: String, state: &AppState) -> bool {
    let user = sqlx::query_as!(
        RelationRow,
        "SELECT user_email, document_id, user_role FROM document_relation WHERE user_email = $1 AND document_id = $2 AND user_role = 'owner'",
        email,
        doc_id,
    ).fetch_optional(&state.pg_pool)
    .await;

    matches!(user, Ok(Some(_)))
}


// Struct for the login request
#[derive(Deserialize)]
struct LoginRequest {
    email: String,
    password: String,
}
#[derive(Deserialize)]
struct DocumentRequest {
    user_email: String,
    document_id: String,
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
struct AppState {
    // Add the database connection pool and MongoDB client here
    pg_pool: PgPool,
    mongo_db: MongoDatabase,
}

#[derive(Deserialize, Serialize, Debug)]
struct Document {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    id: Option<ObjectId>,
    title: String,
    content: String,
    format: String,
}