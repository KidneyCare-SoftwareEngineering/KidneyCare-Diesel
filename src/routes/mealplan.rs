use crate::schema::{
    meal_plan_recipes, meal_plans, recipes, recipes_nutrients, recipes_ingredient_allergies, users,
    users_ingredient_allergies, users_nutrients_limit_per_day,
};
use axum::http::StatusCode;
use axum::{Extension, Json};
use chrono::NaiveDate;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Nutrition {
    pub calories: f32,
    pub carbs: f32,
    pub fat: f32,
    pub phosphorus: f32,
    pub potassium: f32,
    pub protein: f32,
    pub sodium: f32,
}
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Recipe {
    pub recipe_id: Option<i32>, // Change recipe_id to Option<i32>
}

#[derive(Deserialize, Debug)]
pub struct CreateMealPlanPayload {
    pub user_line_id: String,        // The user_line_id field
    pub mealplans: Vec<Vec<Recipe>>, // A 2D vector representing the meal plans
}

#[derive(Deserialize, Debug)]
pub struct GetMealPlanRequest {
    pub user_line_id: String,
    pub date: Option<String>,
}

#[derive(Serialize, Debug)]
pub struct RecipeInfo {
    pub recipe_id: i32,
    pub recipe_name: String,
    pub recipe_img_link: Vec<String>,
    pub ischecked: Option<bool>,
    pub meal_plan_recipe_id: i32,
    pub meal_time: Option<i32>, // Add meal_time
    pub calories: f64,         // Add calories
}

#[derive(Serialize, Debug)]
pub struct MealPlanEntry {
    pub meal_plan_id: i32,
    pub user_id: i32,
    pub name: String,
    pub date: NaiveDate,
    pub recipes: Vec<RecipeInfo>,
}

#[derive(Serialize, Debug)]
pub struct GetMealPlanResponse {
    pub meal_plans: Vec<MealPlanEntry>,
}

#[derive(Serialize, Debug)]
pub struct ErrorResponse {
    pub error: String,
}

#[derive(Deserialize, Debug)]
pub struct UserAlreadyEatPayload {
    pub meal_plan_recipe_id: i32,
    pub ischecked: bool,
}

#[derive(Deserialize, Debug)]
pub struct EditMealPlanPayload {
    pub user_line_id: String,
    pub date: String,
    pub recipes: Vec<Recipe>,
}

#[derive(Deserialize, Debug)]
pub struct MealPlanRequest {
    pub data: MealPlanRequestData,
}

#[derive(Deserialize, Debug)]
pub struct MealPlanRequestData {
    pub u_id: String,
    pub days: i32,
}

#[derive(Serialize, Debug, Clone)]
pub struct FoodMenu {
    pub name: String,
    pub nutrition: Nutrition,
    pub recipe_id: i32,
    pub recipe_img_link: Vec<String>,
}

#[derive(Serialize, Debug)]
pub struct ResponseData {
    pub user_line_id: String,
    pub days: i32,
    pub food_menus: Vec<FoodMenu>,
    pub nutrition_limit_per_day: Nutrition,
}

#[derive(Deserialize, Debug)]
pub struct UpdateMealPlanRequest {
    pub user_id: String,
    pub days: i32,
    pub mealplans: Vec<Vec<Recipe>>, // Use Recipe for mealplans
}

#[derive(Serialize, Debug)]
pub struct UpdateMealPlanResponse {
    pub user_line_id: String,
    pub days: i32,
    pub nutrition_limit_per_day: Nutrition,
    pub food_menus: Vec<FoodMenu>,
    pub mealplan: UpdateMealPlanRequestWithoutDays,
}

#[derive(Serialize, Debug)]
pub struct UpdateMealPlanRequestWithoutDays {
    pub user_id: String,
    pub mealplans: Vec<Vec<FoodMenu>>,
}

pub type DbPool = Pool<ConnectionManager<PgConnection>>;

#[axum::debug_handler]
pub async fn create_meal_plan(
    Extension(db_pool): Extension<Arc<DbPool>>,
    Json(payload): Json<CreateMealPlanPayload>,
) -> Result<Json<serde_json::Value>, Json<serde_json::Value>> {
    println!("Received create_meal_plan payload: {:?}", payload);

    let mut conn = db_pool.get().map_err(|err| {
        println!("Failed to connect to the database: {}", err);
        Json(json!({ "status": "error", "message": "Failed to connect to the database" }))
    })?;

    // 1. Fetch user_id from user_line_id
    let user_id: i32 = users::table
        .filter(users::user_line_id.eq(&payload.user_line_id))
        .select(users::user_id)
        .first(&mut conn)
        .map_err(|err| {
            println!("Failed to fetch user: {}", err);
            Json(json!({ "status": "error", "message": "User not found" }))
        })?;

    println!("Fetched user_id: {}", user_id);

    // 2. Find the latest meal plan date for the user
    let latest_date: Option<NaiveDate> = meal_plans::table
        .filter(meal_plans::user_id.eq(user_id))
        .select(meal_plans::date)
        .order(meal_plans::date.desc())
        .first::<NaiveDate>(&mut conn)
        .optional()
        .map_err(|err| {
            println!("Failed to fetch latest meal plan date: {}", err);
            Json(json!({ "status": "error", "message": "Failed to fetch latest meal plan date" }))
        })?;

    let today = chrono::Local::now().date_naive();
    let start_date = match latest_date {
        Some(date) if date >= today => date + chrono::Duration::days(1), // Start from the next day if the latest date is in the future or today
        _ => today, // Start from today if no meal plans exist or the latest date is in the past
    };

    println!("Starting meal plan creation from date: {}", start_date);

    // 3. Create new meal plans
    let transaction_result = {
        let conn = &mut conn;
        conn.transaction::<_, diesel::result::Error, _>(|conn| {
            for (day_index, day_mealplans) in payload.mealplans.iter().enumerate() {
                let meal_plan_date = start_date + chrono::Duration::days(day_index as i64);
                println!("Creating meal plan for date: {}", meal_plan_date);

                let meal_plan_name = format!("Meal Plan {}", meal_plan_date.format("%d/%m/%Y"));

                let meal_plan_id: i32 = diesel::insert_into(meal_plans::table)
                    .values((
                        meal_plans::user_id.eq(user_id),
                        meal_plans::name.eq(meal_plan_name),
                        meal_plans::date.eq(meal_plan_date),
                    ))
                    .returning(meal_plans::meal_plan_id)
                    .get_result(conn)?;

                println!("Created meal_plan_id: {}", meal_plan_id);

                for (recipe_index, recipe) in day_mealplans.iter().enumerate() {
                    if let Some(recipe_id) = recipe.recipe_id {
                        println!("Processing recipe_id: {}", recipe_id);

                        // Determine meal_time
                        let meal_time = if recipe_index < 4 {
                            (recipe_index + 1) as i32 // 1, 2, 3, 4 for the first four recipes
                        } else {
                            4 // 4 for all subsequent recipes
                        };

                        diesel::insert_into(meal_plan_recipes::table)
                            .values((
                                meal_plan_recipes::meal_plan_id.eq(meal_plan_id),
                                meal_plan_recipes::recipe_id.eq(recipe_id),
                                meal_plan_recipes::ischecked.eq(false),
                                meal_plan_recipes::meal_time.eq(Some(meal_time)), // Assign meal_time
                            ))
                            .execute(conn)?;
                    }
                }
            }
            Ok(())
        })
    };

    transaction_result.map_err(|err| {
        println!("Failed to create meal plan: {}", err);
        Json(json!({ "status": "error", "message": "Failed to create meal plan" }))
    })?;

    println!("Meal plan created successfully");
    Ok(Json(
        json!({ "status": "success", "message": "Meal plan created successfully" }),
    ))
}

#[axum::debug_handler]
pub async fn get_meal_plan(
    Extension(db_pool): Extension<Arc<DbPool>>,
    Json(payload): Json<GetMealPlanRequest>,
) -> Result<Json<GetMealPlanResponse>, (StatusCode, Json<ErrorResponse>)> {
    let mut conn = db_pool.get().map_err(|err| {
        eprintln!("Failed to connect to the database: {}", err);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to connect to the database".to_string(),
            }),
        )
    })?;

    // 1. Fetch user_id from user_line_id
    let user_id: i32 = users::table
        .filter(users::user_line_id.eq(&payload.user_line_id))
        .select(users::user_id)
        .first(&mut conn)
        .map_err(|_| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: "User not found".to_string(),
                }),
            )
        })?;

    // 2. Build the query
    let mut query = meal_plans::table
        .inner_join(
            meal_plan_recipes::table
                .on(meal_plans::meal_plan_id.eq(meal_plan_recipes::meal_plan_id)),
        )
        .inner_join(recipes::table.on(meal_plan_recipes::recipe_id.eq(recipes::recipe_id)))
        .filter(meal_plans::user_id.eq(user_id))
        .into_boxed();

    if let Some(date_str) = &payload.date {
        let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d").map_err(|_| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: "Invalid date format. Use YYYY-MM-DD".to_string(),
                }),
            )
        })?;
        query = query.filter(meal_plans::date.eq(date));
    }

    // 3. Fetch meal plans
    let results = query
        .select((
            meal_plans::meal_plan_id,
            meal_plans::user_id,
            meal_plans::name,
            meal_plans::date,
            meal_plan_recipes::meal_plan_recipe_id,
            meal_plan_recipes::recipe_id,
            meal_plan_recipes::meal_time, // Include meal_time
            recipes::recipe_name,
            recipes::recipe_img_link,
            recipes::calories, // Include calories
            meal_plan_recipes::ischecked,
        ))
        .load::<(
            i32,
            i32,
            String,
            NaiveDate,
            i32,
            i32,
            Option<i32>,
            String,
            Option<Vec<Option<String>>>,
            f64,
            Option<bool>,
        )>(&mut conn)
        .map_err(|err| {
            eprintln!("Database error fetching meal plans: {}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Error fetching meal plans".to_string(),
                }),
            )
        })?;

    // 4. Organize the data into the desired structure
    let mut meal_plans_map: HashMap<i32, MealPlanEntry> = HashMap::new();
    for (
        meal_plan_id,
        user_id,
        name,
        date,
        meal_plan_recipe_id,
        recipe_id,
        meal_time,
        recipe_name,
        recipe_img_link,
        calories,
        ischecked,
    ) in results
    {
        let meal_plan_entry = meal_plans_map
            .entry(meal_plan_id)
            .or_insert_with(|| MealPlanEntry {
                meal_plan_id,
                user_id,
                name,
                date,
                recipes: Vec::new(),
            });

        meal_plan_entry.recipes.push(RecipeInfo {
            recipe_id,
            recipe_name,
            recipe_img_link: recipe_img_link
                .unwrap_or_default()
                .into_iter()
                .filter_map(|x| x)
                .collect(),
            ischecked,
            meal_plan_recipe_id,
            meal_time,
            calories,
        });
    }

    let meal_plans: Vec<MealPlanEntry> = meal_plans_map.into_values().collect();

    Ok(Json(GetMealPlanResponse { meal_plans }))
}

#[axum::debug_handler]
pub async fn user_already_eat(
    Extension(db_pool): Extension<Arc<DbPool>>,
    Json(payload): Json<UserAlreadyEatPayload>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    let mut conn = db_pool.get().map_err(|err| {
        eprintln!("Failed to connect to the database: {}", err);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to connect to the database".to_string(),
            }),
        )
    })?;

    // Update the ischecked field for the given meal_plan_recipe_id
    diesel::update(meal_plan_recipes::table.filter(meal_plan_recipes::meal_plan_recipe_id.eq(payload.meal_plan_recipe_id)))
        .set(meal_plan_recipes::ischecked.eq(payload.ischecked))
        .execute(&mut conn)
        .map_err(|err| {
            eprintln!("Failed to update ischecked: {}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to update meal plan recipe".to_string(),
                }),
            )
        })?;

    println!(
        "Updated meal_plan_recipe_id {} with ischecked = {}",
        payload.meal_plan_recipe_id, payload.ischecked
    );

    Ok(Json(json!({
        "status": "success",
        "message": "Meal plan recipe updated successfully"
    })))
}

#[axum::debug_handler]
pub async fn edit_meal_plan(
    Extension(db_pool): Extension<Arc<DbPool>>,
    Json(payload): Json<EditMealPlanPayload>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    let mut conn = db_pool.get().map_err(|err| {
        eprintln!("Failed to connect to the database: {}", err);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to connect to the database".to_string(),
            }),
        )
    })?;

    // 1. Fetch user_id from user_line_id
    let user_id: i32 = users::table
        .filter(users::user_line_id.eq(&payload.user_line_id))
        .select(users::user_id)
        .first(&mut conn)
        .map_err(|_| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: "User not found".to_string(),
                }),
            )
        })?;

    // 2. Parse the date and find the meal_plan_id
    let date = NaiveDate::parse_from_str(&payload.date, "%Y-%m-%d").map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Invalid date format. Use YYYY-MM-DD".to_string(),
            }),
        )
    })?;

    let meal_plan_id: i32 = meal_plans::table
        .filter(meal_plans::user_id.eq(user_id))
        .filter(meal_plans::date.eq(date))
        .select(meal_plans::meal_plan_id)
        .first(&mut conn)
        .map_err(|_| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: "Meal plan not found for the given date".to_string(),
                }),
            )
        })?;

    println!("Found meal_plan_id: {}", meal_plan_id);

    // 3. Delete existing recipes for the meal_plan_id
    diesel::delete(meal_plan_recipes::table.filter(meal_plan_recipes::meal_plan_id.eq(meal_plan_id)))
        .execute(&mut conn)
        .map_err(|err| {
            eprintln!("Failed to delete old recipes: {}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to delete old recipes".to_string(),
                }),
            )
        })?;

    println!("Deleted old recipes for meal_plan_id: {}", meal_plan_id);

    // 4. Insert new recipes
    let transaction_result = {
        let conn = &mut conn;
        conn.transaction::<_, diesel::result::Error, _>(|conn| {
            for (recipe_index, recipe) in payload.recipes.iter().enumerate() {
                let meal_time = if recipe_index < 4 {
                    (recipe_index + 1) as i32 // 1, 2, 3, 4 for the first four recipes
                } else {
                    4 // 4 for all subsequent recipes
                };

                diesel::insert_into(meal_plan_recipes::table)
                    .values((
                        meal_plan_recipes::meal_plan_id.eq(meal_plan_id),
                        meal_plan_recipes::recipe_id.eq(recipe.recipe_id
                            .ok_or_else(|| diesel::result::Error::RollbackTransaction)?),
                        meal_plan_recipes::ischecked.eq(false),
                        meal_plan_recipes::meal_time.eq(Some(meal_time)),
                    ))
                    .execute(conn)?;
            }
            Ok(())
        })
    };

    transaction_result.map_err(|err| {
        eprintln!("Failed to insert new recipes: {}", err);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to insert new recipes".to_string(),
            }),
        )
    })?;

    println!("Updated meal plan successfully for meal_plan_id: {}", meal_plan_id);

    Ok(Json(json!({
        "status": "success",
        "message": "Meal plan updated successfully"
    })))
}

#[axum::debug_handler]
pub async fn ai_meal_plan(
    Extension(db_pool): Extension<Arc<DbPool>>,
    Json(payload): Json<MealPlanRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut conn = db_pool.get().map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Database connection error".to_string(),
        )
    })?;

    // 1. Fetch user information
    let user = users::table
        .filter(users::user_line_id.eq(&payload.data.u_id))
        .select((users::user_id, users::user_line_id))
        .first::<(i32, Option<String>)>(&mut conn)
        .map_err(|_| (StatusCode::NOT_FOUND, "User not found".to_string()))?;

    let user_id = user.0;

    // 2. Fetch food menus that the user is not allergic to
    let filtered_recipes = recipes::table
        .left_join(recipes_nutrients::table.on(recipes::recipe_id.eq(recipes_nutrients::recipe_id)))
        .filter(diesel::dsl::not(diesel::dsl::exists(
            recipes_ingredient_allergies::table
                .inner_join(users_ingredient_allergies::table.on(
                    recipes_ingredient_allergies::ingredient_allergy_id
                        .eq(users_ingredient_allergies::ingredient_allergy_id),
                ))
                .filter(users_ingredient_allergies::user_id.eq(user_id))
                .filter(recipes_ingredient_allergies::recipe_id.eq(recipes::recipe_id)),
        )))
        .group_by((recipes::recipe_id, recipes::recipe_name, recipes::recipe_img_link))
        .select((
            recipes::recipe_id,
            recipes::recipe_name,
            recipes::recipe_img_link,
            diesel::dsl::sum(recipes_nutrients::quantity).nullable(), // protein
            diesel::dsl::sum(recipes_nutrients::quantity).nullable(), // carbs
            diesel::dsl::sum(recipes_nutrients::quantity).nullable(), // fat
            diesel::dsl::sum(recipes_nutrients::quantity).nullable(), // sodium
            diesel::dsl::sum(recipes_nutrients::quantity).nullable(), // phosphorus
            diesel::dsl::sum(recipes_nutrients::quantity).nullable(), // potassium
            diesel::dsl::sum(recipes_nutrients::quantity).nullable(), // calories
        ))
        .load::<(
            i32,
            String,
            Option<Vec<Option<String>>>,
            Option<f64>,
            Option<f64>,
            Option<f64>,
            Option<f64>,
            Option<f64>,
            Option<f64>,
            Option<f64>,
        )>(&mut conn)
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error fetching filtered recipes".to_string(),
            )
        })?;

    // 3. Fetch the user's daily nutrition limits
    let nutrition_limits = users_nutrients_limit_per_day::table
        .filter(users_nutrients_limit_per_day::user_id.eq(user_id))
        .select((
            users_nutrients_limit_per_day::nutrient_id,
            users_nutrients_limit_per_day::nutrient_limit,
        ))
        .load::<(Option<i32>, Option<f64>)>(&mut conn)
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error fetching nutrition limits".to_string(),
            )
        })?;

    let mut nutrition_map = Nutrition {
        calories: 0.0,
        carbs: 0.0,
        fat: 0.0,
        phosphorus: 0.0,
        potassium: 0.0,
        protein: 0.0,
        sodium: 0.0,
    };

    for (nutrient_id, nutrient_limit) in nutrition_limits {
        match nutrient_id {
            Some(1) => nutrition_map.calories = nutrient_limit.unwrap_or(0.0) as f32,
            Some(2) => nutrition_map.carbs = nutrient_limit.unwrap_or(0.0) as f32,
            Some(3) => nutrition_map.fat = nutrient_limit.unwrap_or(0.0) as f32,
            Some(4) => nutrition_map.phosphorus = nutrient_limit.unwrap_or(0.0) as f32,
            Some(5) => nutrition_map.potassium = nutrient_limit.unwrap_or(0.0) as f32,
            Some(6) => nutrition_map.protein = nutrient_limit.unwrap_or(0.0) as f32,
            Some(7) => nutrition_map.sodium = nutrient_limit.unwrap_or(0.0) as f32,
            _ => (),
        }
    }

    // 4. Convert the filtered recipes into the required `FoodMenu` format
    let food_menus: Vec<FoodMenu> = filtered_recipes
        .into_iter()
        .map(|recipe| FoodMenu {
            name: recipe.1,
            nutrition: Nutrition {
                calories: recipe.9.unwrap_or(0.0) as f32,
                carbs: recipe.4.unwrap_or(0.0) as f32,
                fat: recipe.5.unwrap_or(0.0) as f32,
                phosphorus: recipe.7.unwrap_or(0.0) as f32,
                potassium: recipe.8.unwrap_or(0.0) as f32,
                protein: recipe.3.unwrap_or(0.0) as f32,
                sodium: recipe.6.unwrap_or(0.0) as f32,
            },
            recipe_id: recipe.0,
            recipe_img_link: recipe
                .2
                .unwrap_or_default()
                .into_iter()
                .filter_map(|x| x)
                .collect(),
        })
        .collect();

    // 5. Construct the request payload
    let response_data = ResponseData {
        user_line_id: user_id.to_string(), // Send user_id but label it as user_line_id
        days: payload.data.days,
        food_menus,
        nutrition_limit_per_day: nutrition_map,
    };

    // Print request before sending
    println!("Sending request to AI service: {:#?}", response_data);

    // 6. Send a POST request to the external AI service
    let client = Client::new();
    let api_url = "https://ai-rec-1025044834972.asia-southeast1.run.app/ai";

    let response = client
        .post(api_url)
        .json(&response_data)
        .send()
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to send request".to_string(),
            )
        })?
        .json::<serde_json::Value>()
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to parse response".to_string(),
            )
        })?;

    // 7. Return the response from the external API
    Ok(Json(response))
}

#[axum::debug_handler]
pub async fn update_meal_plan(
    Extension(db_pool): Extension<Arc<DbPool>>,
    Json(payload): Json<UpdateMealPlanRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut conn = db_pool.get().map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Database connection error".to_string()))?;

    // 1. Fetch user information
    let user = users::table
        .filter(users::user_line_id.eq(&payload.user_id))
        .select((users::user_id, users::user_line_id))
        .first::<(i32, Option<String>)>(&mut conn)
        .map_err(|_| (StatusCode::NOT_FOUND, "User not found".to_string()))?;

    let user_id = user.0;
    let user_line_id = user.1.ok_or((StatusCode::NOT_FOUND, "User LINE ID not found".to_string()))?;

    // 2. Validate and filter mealplans
    let valid_mealplans: Vec<Vec<Recipe>> = payload
        .mealplans
        .iter()
        .map(|day| {
            day.iter()
                .filter(|recipe| recipe.recipe_id.is_some()) // Keep only recipes with valid recipe_id
                .cloned()
                .collect()
        })
        .collect();

    // 3. Fetch food menus that the user is not allergic to
    let filtered_recipes = recipes::table
        .left_join(recipes_nutrients::table.on(recipes::recipe_id.eq(recipes_nutrients::recipe_id)))
        .filter(diesel::dsl::not(diesel::dsl::exists(
            recipes_ingredient_allergies::table
                .inner_join(users_ingredient_allergies::table.on(
                    recipes_ingredient_allergies::ingredient_allergy_id.eq(users_ingredient_allergies::ingredient_allergy_id),
                ))
                .filter(users_ingredient_allergies::user_id.eq(user_id))
                .filter(recipes_ingredient_allergies::recipe_id.eq(recipes::recipe_id)),
        )))
        .group_by((recipes::recipe_id, recipes::recipe_name, recipes::recipe_img_link))
        .select((
            recipes::recipe_id,
            recipes::recipe_name,
            recipes::recipe_img_link,
            diesel::dsl::sum(recipes_nutrients::quantity).nullable(), // protein
            diesel::dsl::sum(recipes_nutrients::quantity).nullable(), // carbs
            diesel::dsl::sum(recipes_nutrients::quantity).nullable(), // fat
            diesel::dsl::sum(recipes_nutrients::quantity).nullable(), // sodium
            diesel::dsl::sum(recipes_nutrients::quantity).nullable(), // phosphorus
            diesel::dsl::sum(recipes_nutrients::quantity).nullable(), // potassium
            diesel::dsl::sum(recipes_nutrients::quantity).nullable(), // calories
        ))
        .load::<(
            i32,
            String,
            Option<Vec<Option<String>>>,
            Option<f64>,
            Option<f64>,
            Option<f64>,
            Option<f64>,
            Option<f64>,
            Option<f64>,
            Option<f64>,
        )>(&mut conn)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Error fetching filtered recipes".to_string()))?;

    // 4. Fetch the user's daily nutrition limits
    let nutrition_limits = users_nutrients_limit_per_day::table
        .filter(users_nutrients_limit_per_day::user_id.eq(user_id))
        .select((users_nutrients_limit_per_day::nutrient_id, users_nutrients_limit_per_day::nutrient_limit))
        .load::<(Option<i32>, Option<f64>)>(&mut conn)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Error fetching nutrition limits".to_string()))?;

    let mut nutrition_map = Nutrition {
        calories: 0.0,
        carbs: 0.0,
        fat: 0.0,
        phosphorus: 0.0,
        potassium: 0.0,
        protein: 0.0,
        sodium: 0.0,
    };

    for (nutrient_id, nutrient_limit) in nutrition_limits {
        match nutrient_id {
            Some(1) => nutrition_map.calories = nutrient_limit.unwrap_or(0.0) as f32,
            Some(2) => nutrition_map.carbs = nutrient_limit.unwrap_or(0.0) as f32,
            Some(3) => nutrition_map.fat = nutrient_limit.unwrap_or(0.0) as f32,
            Some(4) => nutrition_map.phosphorus = nutrient_limit.unwrap_or(0.0) as f32,
            Some(5) => nutrition_map.potassium = nutrient_limit.unwrap_or(0.0) as f32,
            Some(6) => nutrition_map.protein = nutrient_limit.unwrap_or(0.0) as f32,
            Some(7) => nutrition_map.sodium = nutrient_limit.unwrap_or(0.0) as f32,
            _ => (),
        }
    }

    // 5. Convert the filtered recipes into the required `FoodMenu` format
    let food_menus: Vec<FoodMenu> = filtered_recipes
        .into_iter()
        .map(|recipe| FoodMenu {
            name: recipe.1,
            nutrition: Nutrition {
                calories: recipe.9.unwrap_or(0.0) as f32,
                carbs: recipe.4.unwrap_or(0.0) as f32,
                fat: recipe.5.unwrap_or(0.0) as f32,
                phosphorus: recipe.7.unwrap_or(0.0) as f32,
                potassium: recipe.8.unwrap_or(0.0) as f32,
                protein: recipe.3.unwrap_or(0.0) as f32,
                sodium: recipe.6.unwrap_or(0.0) as f32,
            },
            recipe_id: recipe.0,
            recipe_img_link: recipe.2.unwrap_or_default().into_iter().filter_map(|x| x).collect(),
        })
        .collect();

    // 6. Construct the detailed mealplans
    let detailed_mealplans: Vec<Vec<FoodMenu>> = valid_mealplans
        .iter()
        .map(|day| {
            day.iter()
                .filter_map(|recipe| {
                    food_menus.iter().find(|menu| menu.recipe_id == recipe.recipe_id.unwrap_or_default()).cloned()
                })
                .collect()
        })
        .collect();

    // 7. Construct the response to send to the external API
    let response_data = UpdateMealPlanResponse {
        user_line_id: user_line_id.clone(),
        days: payload.days,
        nutrition_limit_per_day: nutrition_map,
        food_menus,
        mealplan: UpdateMealPlanRequestWithoutDays {
            user_id: user_line_id.clone(),
            mealplans: detailed_mealplans, // Use detailed mealplans
        },
    };

    // Print the request JSON
    let request_json = serde_json::to_string(&response_data).map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to serialize request JSON".to_string()))?;
    println!("Request JSON to ai_update: {}", request_json);

    // 8. Send a POST request to the external AI service
    let client = Client::new();
    let api_url = "https://ai-rec-1025044834972.asia-southeast1.run.app/ai_update";

    let mut ai_response = client
        .post(api_url)
        .json(&response_data)
        .send()
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to send request".to_string()))?
        .json::<serde_json::Value>()
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to parse response".to_string()))?;

    // 9. Rename `user_id` to `user_line_id` in the AI response
    if let Some(user_id) = ai_response.get("user_id").cloned() {
        ai_response.as_object_mut().unwrap().remove("user_id");
        ai_response.as_object_mut().unwrap().insert("user_line_id".to_string(), user_id);
    }

    // 10. Return the modified AI response
    Ok(Json(ai_response))
}
