use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx::types::BigDecimal;

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderItem {
    pub product_id: i64,
    pub quantity: i32,
}

#[derive(Debug, Serialize, Deserialize)]
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
