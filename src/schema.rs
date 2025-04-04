// @generated automatically by Diesel CLI.

diesel::table! {
    admins (admin_email) {
        admin_email -> Text,
        admin_password -> Uuid,
    }
}

diesel::table! {
    disease (disease_id) {
        disease_id -> Int4,
        disease_name -> Text,
    }
}

diesel::table! {
    food_condition_types (food_condition_type_id) {
        food_condition_type_id -> Int4,
        food_condition_type_name -> Text,
    }
}

diesel::table! {
    ingredient_allergies (ingredient_allergy_id) {
        ingredient_allergy_id -> Int4,
        ingredient_allergy_name -> Text,
    }
}

diesel::table! {
    ingredients (ingredient_id) {
        ingredient_id -> Int4,
        #[max_length = 150]
        ingredient_name -> Varchar,
        ingredient_name_eng -> Nullable<Text>,
    }
}

diesel::table! {
    meal_plan_recipes (meal_plan_recipe_id) {
        meal_plan_recipe_id -> Int4,
        meal_plan_id -> Int4,
        recipe_id -> Int4,
        ischecked -> Nullable<Bool>,
        meal_time -> Nullable<Int4>,
    }
}

diesel::table! {
    meal_plans (meal_plan_id) {
        meal_plan_id -> Int4,
        user_id -> Int4,
        #[max_length = 100]
        name -> Varchar,
        date -> Date,
    }
}

diesel::table! {
    nutrients (nutrient_id) {
        nutrient_id -> Int4,
        #[max_length = 100]
        name -> Varchar,
        #[max_length = 50]
        unit -> Varchar,
    }
}

diesel::table! {
    recipes (recipe_id) {
        recipe_id -> Int4,
        #[max_length = 150]
        recipe_name -> Varchar,
        recipe_method -> Nullable<Array<Nullable<Text>>>,
        calories -> Float8,
        #[max_length = 50]
        calories_unit -> Varchar,
        recipe_img_link -> Nullable<Array<Nullable<Text>>>,
        food_category -> Array<Nullable<Text>>,
        dish_type -> Nullable<Array<Nullable<Text>>>,
    }
}

diesel::table! {
    recipes_ingredient_allergies (recipe_id, ingredient_allergy_id) {
        recipe_id -> Int4,
        ingredient_allergy_id -> Int4,
    }
}

diesel::table! {
    recipes_ingredients (recipes_ingredients_id) {
        recipes_ingredients_id -> Int4,
        recipe_id -> Int4,
        ingredient_id -> Int4,
        amount -> Int4,
        #[max_length = 50]
        ingredient_unit -> Varchar,
    }
}

diesel::table! {
    recipes_nutrients (recipe_nutrient_id) {
        recipe_nutrient_id -> Int4,
        recipe_id -> Int4,
        nutrient_id -> Int4,
        quantity -> Float8,
    }
}

diesel::table! {
    user_calorie_tracking (tracking_id) {
        tracking_id -> Int4,
        user_id -> Int4,
        date -> Date,
        calories -> Float8,
    }
}

diesel::table! {
    user_medicines (user_medicine_id) {
        user_medicine_id -> Int4,
        user_id -> Int4,
        medicine_per_times -> Float8,
        user_medicine_img_link -> Nullable<Array<Nullable<Text>>>,
        #[max_length = 50]
        medicine_unit -> Nullable<Varchar>,
        medicine_name -> Nullable<Text>,
        medicine_note -> Nullable<Text>,
        medicine_schedule -> Nullable<Array<Nullable<Timestamp>>>,
        medicine_amount -> Nullable<Int4>,
    }
}

diesel::table! {
    user_nutrient_tracking (tracking_id) {
        tracking_id -> Int4,
        user_id -> Int4,
        nutrient_id -> Int4,
        date -> Date,
        quantity -> Float8,
    }
}

diesel::table! {
    user_take_medicines (user_take_medicines_id) {
        user_take_medicines_id -> Int4,
        user_id -> Nullable<Int4>,
        user_medicine_id -> Nullable<Int4>,
        user_take_medicine_time -> Nullable<Date>,
    }
}

diesel::table! {
    users (user_id) {
        user_id -> Int4,
        #[max_length = 100]
        name -> Varchar,
        birthdate -> Timestamp,
        weight -> Float8,
        height -> Float8,
        #[max_length = 255]
        profile_img_link -> Nullable<Varchar>,
        user_line_id -> Nullable<Text>,
        #[max_length = 50]
        gender -> Nullable<Varchar>,
        kidney_level -> Nullable<Int4>,
        kidney_dialysis -> Nullable<Bool>,
    }
}

diesel::table! {
    users_diseases (users_diseases_id) {
        users_diseases_id -> Int4,
        user_id -> Int4,
        disease_id -> Int4,
    }
}

diesel::table! {
    users_food_condition_types (users_food_condition_types_id) {
        users_food_condition_types_id -> Int4,
        user_id -> Int4,
        food_condition_type_id -> Int4,
    }
}

diesel::table! {
    users_ingredient_allergies (users_ingredient_allergies_id) {
        users_ingredient_allergies_id -> Int4,
        user_id -> Int4,
        ingredient_allergy_id -> Int4,
    }
}

diesel::table! {
    users_nutrients_limit_per_day (users_nutrients_limit_per_day_id) {
        users_nutrients_limit_per_day_id -> Int4,
        user_id -> Nullable<Int4>,
        nutrient_id -> Nullable<Int4>,
        nutrient_limit -> Nullable<Float8>,
    }
}

diesel::joinable!(meal_plan_recipes -> meal_plans (meal_plan_id));
diesel::joinable!(meal_plans -> users (user_id));
diesel::joinable!(recipes_ingredient_allergies -> ingredient_allergies (ingredient_allergy_id));
diesel::joinable!(recipes_ingredient_allergies -> recipes (recipe_id));
diesel::joinable!(recipes_ingredients -> ingredients (ingredient_id));
diesel::joinable!(recipes_nutrients -> nutrients (nutrient_id));
diesel::joinable!(user_calorie_tracking -> users (user_id));
diesel::joinable!(user_medicines -> users (user_id));
diesel::joinable!(user_nutrient_tracking -> nutrients (nutrient_id));
diesel::joinable!(user_nutrient_tracking -> users (user_id));
diesel::joinable!(user_take_medicines -> user_medicines (user_medicine_id));
diesel::joinable!(user_take_medicines -> users (user_id));
diesel::joinable!(users_diseases -> disease (disease_id));
diesel::joinable!(users_diseases -> users (user_id));
diesel::joinable!(users_food_condition_types -> food_condition_types (food_condition_type_id));
diesel::joinable!(users_food_condition_types -> users (user_id));
diesel::joinable!(users_ingredient_allergies -> ingredient_allergies (ingredient_allergy_id));
diesel::joinable!(users_ingredient_allergies -> users (user_id));
diesel::joinable!(users_nutrients_limit_per_day -> nutrients (nutrient_id));
diesel::joinable!(users_nutrients_limit_per_day -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    admins,
    disease,
    food_condition_types,
    ingredient_allergies,
    ingredients,
    meal_plan_recipes,
    meal_plans,
    nutrients,
    recipes,
    recipes_ingredient_allergies,
    recipes_ingredients,
    recipes_nutrients,
    user_calorie_tracking,
    user_medicines,
    user_nutrient_tracking,
    user_take_medicines,
    users,
    users_diseases,
    users_food_condition_types,
    users_ingredient_allergies,
    users_nutrients_limit_per_day,
);
