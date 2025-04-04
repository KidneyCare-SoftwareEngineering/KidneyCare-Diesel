use diesel::prelude::*;
use serde::{Serialize, Deserialize};

// Admins Table
#[derive(Queryable, Selectable, Insertable, Debug, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::admins)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Admin {
    pub admin_email: String,
    pub admin_password: uuid::Uuid,
}

// Disease Table
#[derive(Queryable, Selectable, Insertable, Debug, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::disease)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Disease {
    pub disease_id: i32,
    pub disease_name: String,
}

// Food Condition Types Table
#[derive(Queryable, Selectable, Insertable, Debug, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::food_condition_types)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct FoodConditionType {
    pub food_condition_type_id: i32,
    pub food_condition_type_name: String,
}

// Ingredient Allergies Table
#[derive(Queryable, Selectable, Insertable, Debug, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::ingredient_allergies)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct IngredientAllergy {
    pub ingredient_allergy_id: i32,
    pub ingredient_allergy_name: String,
}

// Ingredients Table
#[derive(Queryable, Selectable, Insertable, Debug, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::ingredients)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Ingredient {
    pub ingredient_id: i32,
    pub ingredient_name: String,
    pub ingredient_name_eng: Option<String>,
}

// Meal Plan Recipes Table
#[derive(Queryable, Selectable, Insertable, Debug, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::meal_plan_recipes)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct MealPlanRecipe {
    pub meal_plan_recipe_id: i32,
    pub meal_plan_id: i32,
    pub recipe_id: i32,
    pub ischecked: Option<bool>,
}

// Meal Plans Table
#[derive(Queryable, Selectable, Insertable, Debug, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::meal_plans)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct MealPlan {
    pub meal_plan_id: i32,
    pub user_id: i32,
    pub name: String,
    pub date: chrono::NaiveDate,
}

// Nutrients Table
#[derive(Queryable, Selectable, Insertable, Debug, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::nutrients)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Nutrient {
    pub nutrient_id: i32,
    pub name: String,
    pub unit: String,
}

// Recipes Table
#[derive(Queryable, Selectable, Insertable, Debug, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::recipes)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Recipe {
    pub recipe_id: i32,
    pub recipe_name: String,
    pub recipe_method: Option<Vec<Option<String>>>,
    pub calories: f64,
    pub calories_unit: String,
    pub recipe_img_link: Option<Vec<Option<String>>>,
    pub food_category: Vec<Option<String>>,
    pub dish_type: Option<Vec<Option<String>>>,
}

// Recipes Ingredient Allergies Table
#[derive(Queryable, Selectable, Insertable, Debug, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::recipes_ingredient_allergies)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct RecipeIngredientAllergy {
    pub recipe_id: i32,
    pub ingredient_allergy_id: i32,
}

// Recipes Ingredients Table
#[derive(Queryable, Selectable, Insertable, Debug, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::recipes_ingredients)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct RecipeIngredient {
    pub recipes_ingredients_id: i32,
    pub recipe_id: i32,
    pub ingredient_id: i32,
    pub amount: i32,
    pub ingredient_unit: String,
}

// Recipes Nutrients Table
#[derive(Queryable, Selectable, Insertable, Debug, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::recipes_nutrients)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct RecipeNutrient {
    pub recipe_nutrient_id: i32,
    pub recipe_id: i32,
    pub nutrient_id: i32,
    pub quantity: f64,
}

// User Calorie Tracking Table
#[derive(Queryable, Selectable, Insertable, Debug, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::user_calorie_tracking)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UserCalorieTracking {
    pub tracking_id: i32,
    pub user_id: i32,
    pub date: chrono::NaiveDate,
    pub calories: f64,
}

// User Medicines Table
#[derive(Queryable, Selectable, Insertable, Debug, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::user_medicines)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UserMedicine {
    pub user_medicine_id: i32,
    pub user_id: i32,
}
