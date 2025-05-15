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











































/*
use postgres::{Client, NoTls};
use postgres::Error as postgresError;
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::env;

#[macro_use]
extern crate serde_derive


//Model: User struct with id, first_name, last_name, email and password
#[derive(Serialize, Deserialize)]
struct User {
    id: i32,
    first_name: String,
    last_name: String,
    email: String,
    password: String,
};

// Database URL
const DB_URL: &str = !env("DATABASE_URL");

//Constant for http response
const OK_RESPONSE : &str = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n";
const NOT_FOUND_RESPONSE : &str = "HTTP/1.1 404 Not Found\r\nContent-Type: application/json\r\n\r\n";
const INTERNAL_SERVER_ERROR_RESPONSE : &str = "HTTP/1.1 500 Internal Server Error\r\nContent-Type: application/json\r\n\r\n";

// main function
fn main() {
    //set database 
    if let Err(e) = set_database() {
        eprintln!("Error setting up database: {}", e);
        return;
    }

    // start server with listener

    let listener = TcpListener::bind("forward:8080").unwrap();

    println!("Server running on http://localhost:8080");

    // Handle the incoming connections
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                handle_connection(stream);
            }
            Err(e) => {
                eprintln!("Error accepting connection: {}", e);
            }
        }
    };
}

// set database function
fn set_database() --> Result<(), postgresError> {
    // Connect to the database
    let mut client = Client::connect(DB_URL, NoTls)?;

    // Create the users table if it doesn't exist
    client.execute("
        CREATE TABLE IF NOT EXISTS users (
            id SERIAL PRIMARY KEY,
            first_name VARCHAR NOT NULL,
            last_name VARCHAR NOT NULL,
            email VARCHAR NOT NULL UNIQUE,
            password VARCHAR NOT NULL
        )
    ", &[]
    )?;
}

*/
