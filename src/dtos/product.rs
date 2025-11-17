use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize)]
pub struct NewRodProductDto {
    pub name: String,
    pub description: String,
    pub category_id: i32,
    pub rod_length: Option<String>,
    pub line_weight: Option<String>,
    pub cast_weight: Option<String>,
    pub action: Option<String>,
    pub material: Option<String>,
    pub power: Option<String>,
    pub reel_size: Option<String>,
    pub price: f64,
    pub image_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct RodProductDetail {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub category_name: String,
    pub rod_length: Option<String>,
    pub line_weight: Option<String>,
    pub cast_weight: Option<String>,
    pub action: Option<String>,
    pub material: Option<String>,
    pub power: Option<String>,
    pub reel_size: Option<String>,
    pub price: f64,
    pub image_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct RodProduct {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub category_id: i32,
    pub rod_length: Option<String>,
    pub line_weight: Option<String>,
    pub cast_weight: Option<String>,
    pub action: Option<String>,
    pub material: Option<String>,
    pub power: Option<String>,
    pub reel_size: Option<String>,
    pub price: f64,
    pub image_url: Option<String>,
}
