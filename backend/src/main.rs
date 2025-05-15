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

#[tokio::main]
async fn main() {

    // Load environment variables from .env file
    dotenvy::dotenv().expect("Enviroment file doesn not exist");

    // Get the server address and database URL from environment variables
    let server_address = std::env::var("SERVER_ADDRESS").unwrap_or("127.0.0.1:3000".to_owned());
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not found in env file");

    // Create a connection pool to the PostgreSQL database
    let db_pool = PgPoolOptions::new()
        .max_connections(64)
        .acquire_timeout(Duration::from_secs(5))
        .connect(&database_url)
        .await
        .expect("Cannot connect to database");

    let listener = TcpListener::bind(server_address)
        .await
        .expect("Could not create tcp listener");

    // Print the address the server is listening on
    println!("listening on {}", listener.local_addr().unwrap());

    // Create the Axum router and add the needed routes
    let app = Router::new()
        .route("/login", post(login_user))
        .with_state(db_pool);

    // Serve the application using the listener
    axum::serve(listener, app)
        .await
        .expect("Error serving application");
}


// ***************************************************************************************************************************************


// This function handles the login request
async fn login_user(
    // Extract the database connection pool from the state
    State(db_pool): State<PgPool>,
    Json(payload): Json<LoginRequest>,
) -> Result<(StatusCode, String), (StatusCode, String)> {
    // Checks if the user exists in the database
    let user = sqlx::query_as!(
        UserRow,
        "SELECT email, first_name, last_name FROM users WHERE email = $1 AND password = $2",
        payload.email,
        payload.password
    )
    .fetch_optional(&db_pool)
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

// Struct for the login request
#[derive(Deserialize)]
struct LoginRequest {
    email: String,
    password: String,
}

// Struct for the user row returned from the database
#[derive(Serialize)]
struct UserRow {
    email: String,
    first_name: String,
    last_name: String,
}


