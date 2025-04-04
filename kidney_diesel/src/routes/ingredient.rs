use axum::{Extension, Json};
use axum::http::StatusCode;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use crate::schema::ingredients::dsl::*;
use serde_json::json;

#[derive(Serialize, Queryable)]
pub struct Ingredient {
    pub ingredient_id: i32,
    pub ingredient_name: String,
    pub ingredient_name_eng: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct CreateIngredientPayload {
    pub ingredient_name: String,
    pub ingredient_name_eng: Option<String>, // Optional field
}

pub type DbPool = Pool<ConnectionManager<PgConnection>>;

pub async fn get_ingredients(
    Extension(db_pool): Extension<Arc<DbPool>>,
) -> Result<Json<Vec<Ingredient>>, axum::http::StatusCode> {
    let mut conn = db_pool.get().map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
    let results = ingredients
        .select((ingredient_id, ingredient_name, ingredient_name_eng))
        .load::<Ingredient>(&mut conn)
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(results))
}

pub async fn create_ingredient(
    Extension(db_pool): Extension<Arc<DbPool>>,
    Json(payload): Json<CreateIngredientPayload>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut conn = db_pool.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    diesel::insert_into(ingredients)
        .values((
            ingredient_name.eq(payload.ingredient_name.clone()),
            ingredient_name_eng.eq(payload.ingredient_name_eng.clone()),
        ))
        .execute(&mut conn)
        .map_err(|err| {
            eprintln!("Failed to insert ingredient: {}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    println!(
        "Created ingredient with name: {} and name_eng: {:?}",
        &payload.ingredient_name, &payload.ingredient_name_eng
    );

    Ok(Json(json!({
        "status": "success",
        "message": "Ingredient created successfully"
    })))
}