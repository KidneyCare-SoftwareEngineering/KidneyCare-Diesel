use axum::{
    routing::{delete, get, patch, post}, Extension, Router
};
use tower_http::cors::{Any, CorsLayer};
use dotenvy::dotenv;
use serde::Serialize;
use std::sync::Arc;
use tokio::net::TcpListener;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};

mod schema;
mod models;
mod routes; // Declare the routes module

use routes::ingredient::{get_ingredients, create_ingredient}; // Import create_ingredient
use routes::recipe::{update_recipe, delete_recipe};
use routes::mealplan::{create_meal_plan, get_meal_plan, user_already_eat, edit_meal_plan, ai_meal_plan, update_meal_plan}; // Import edit_meal_plan

use std::env;

// Define a struct to represent the ingredient table rows
#[derive(Serialize, Queryable)]
struct Ingredient {
    id: i32,
    name: String,
    quantity: i32,
}

#[tokio::main]
async fn main() {
    dotenv().expect("Failed to load .env file");

    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let server_address = format!("0.0.0.0:{}", port);

    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        eprintln!("DATABASE_URL not set. Exiting.");
        std::process::exit(1);
    });

    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let db_pool = r2d2::Pool::builder()
        .max_size(16)
        .build(manager)
        .unwrap_or_else(|err| {
            eprintln!("Failed to create database pool: {}", err);
            std::process::exit(1);
        });

    let listener = TcpListener::bind(&server_address).await.unwrap_or_else(|err| {
        eprintln!("Failed to bind to {}: {}", server_address, err);
        std::process::exit(1);
    });

    println!("Listening on {}", listener.local_addr().unwrap());

    let db_pool = Arc::new(db_pool);
    
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([
            http::Method::GET,
            http::Method::POST,
            http::Method::PATCH,
            http::Method::DELETE,
            http::Method::OPTIONS,
        ])
        .allow_headers(Any);

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/ingredients", get(get_ingredients))
        .route("/create_ingredient", post(create_ingredient)) // Add route for create_ingredient
        .route("/update_recipe/{r_id}", patch(update_recipe))
        .route("/delete_recipe/{r_id}", delete(delete_recipe))
        .route("/create_meal_plan", post(create_meal_plan))
        .route("/get_meal_plan", post(get_meal_plan))
        .route("/user_already_eat", patch(user_already_eat))
        .route("/edit_meal_plan", patch(edit_meal_plan))
        .route("/ai_meal_plan", post(ai_meal_plan))
        .route("/update_meal_plan", post(update_meal_plan))
        .fallback(fallback_handler) // Add a fallback route
        .layer(Extension(db_pool))
        .layer(cors);

    if let Err(err) = axum::serve(listener, app).await {
        eprintln!("Server error: {}", err);
    }
}

// Add a fallback handler to log unhandled requests
async fn fallback_handler(uri: axum::http::Uri) -> impl axum::response::IntoResponse {
    eprintln!("Unhandled request: {}", uri);
    (axum::http::StatusCode::NOT_FOUND, "Route not found")
}
