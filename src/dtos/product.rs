use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "name": "Abu Garcia Pro Max Casting Rod",
    "description": "7ft medium power casting rod for bass fishing",
    "category_id": 1,
    "rod_length": "7ft",
    "line_weight": "10-20lb",
    "cast_weight": "1/4-3/4oz",
    "action": "Fast",
    "material": "Carbon Fiber",
    "power": "Medium",
    "reel_size": null,
    "price": 1500000.0,
    "image_url": "https://example.com/rod.jpg"
}))]
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

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct RodProductDetail {
    #[schema(example = 1)]
    pub id: i64,
    #[schema(example = "Abu Garcia Pro Max")]
    pub name: String,
    pub description: String,
    #[schema(example = "Joran Casting")]
    pub category_name: String,
    pub rod_length: Option<String>,
    pub line_weight: Option<String>,
    pub cast_weight: Option<String>,
    pub action: Option<String>,
    pub material: Option<String>,
    pub power: Option<String>,
    pub reel_size: Option<String>,
    #[schema(example = 1500000.0)]
    pub price: f64,
    pub image_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct RodProduct {
    #[schema(example = 1)]
    pub id: i64,
    pub name: String,
    pub description: String,
    #[schema(example = 1)]
    pub category_id: i32,
    pub rod_length: Option<String>,
    pub line_weight: Option<String>,
    pub cast_weight: Option<String>,
    pub action: Option<String>,
    pub material: Option<String>,
    pub power: Option<String>,
    pub reel_size: Option<String>,
    #[schema(example = 1500000.0)]
    pub price: f64,
    pub image_url: Option<String>,
}
