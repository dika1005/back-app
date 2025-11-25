use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx::types::BigDecimal;
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct OrderItem {
    #[schema(example = 1)]
    pub product_id: i64,
    #[schema(example = 2)]
    pub quantity: i32,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "items": [
        {"product_id": 1, "quantity": 2},
        {"product_id": 3, "quantity": 1}
    ],
    "shipping_address": "Jl. Sudirman No. 123, Jakarta",
    "payment_method": "midtrans"
}))]
pub struct NewOrderDto {
    pub items: Vec<OrderItem>,
    pub shipping_address: String,
    pub payment_method: String,
}

#[allow(dead_code)]
#[derive(Debug, FromRow)]
pub struct Order {
    pub id: i64,
    pub user_id: i64,
    pub total_amount: BigDecimal,
    pub shipping_address: String,
    pub payment_method: String,
    pub status: String,
    pub order_date: NaiveDateTime,
}
