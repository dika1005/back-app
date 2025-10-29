use serde::{Deserialize, Serialize};

// Data Item yang Diterima saat Checkout (Bagian dari NewOrderDto)
#[derive(Debug, Deserialize)]
pub struct NewOrderItemDto {
    // product_id BIGINT di SQL, jadi i64 di Rust
    pub product_id: i64, 
    // quantity INT di SQL, jadi i32 di Rust
    pub quantity: i32, 
}

// Data yang Diterima saat Pengguna Melakukan Checkout (POST /checkout)
#[derive(Debug, Deserialize)]
pub struct NewOrderDto {
    // user_id akan diambil dari token (AuthUser), bukan dari body.
    
    // Alamat pengiriman
    pub shipping_address: String,
    
    // Metode pembayaran yang dipilih
    pub payment_method: String,
    
    // Daftar item yang dipesan
    pub items: Vec<NewOrderItemDto>,
}

// Data Order dari Database (Digunakan di Model dan Response)
// sqlx::FromRow digunakan untuk mapping langsung dari tabel 'orders'
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Order {
    // id BIGINT di SQL, jadi i64 di Rust
    pub id: i64,
    // user_id INT di SQL, jadi i32 di Rust
    pub user_id: i32, 
    pub total_amount: f64, // DECIMAL(10, 2) mapped to f64
    pub shipping_address: String,
    pub payment_method: String,
    pub status: String, 
    // Menggunakan NaiveDateTime untuk kolom DATETIME
    pub order_date: chrono::NaiveDateTime, 
}

// Data Order Item dari Database (Digunakan di Model dan Response)
// sqlx::FromRow digunakan untuk mapping langsung dari tabel 'order_items'
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct OrderItem {
    // id BIGINT di SQL, jadi i64 di Rust
    pub id: i64,
    // order_id BIGINT di SQL, jadi i64 di Rust
    pub order_id: i64,
    // product_id BIGINT di SQL, jadi i64 di Rust
    pub product_id: i64,
    pub quantity: i32,
    pub price_at_order: f64, // DECIMAL(10, 2) mapped to f64
}

// Opsional: Struktur Gabungan untuk Response Detail Order
#[derive(Debug, Serialize)]
pub struct OrderDetail {
    pub order: Order,
    pub items: Vec<OrderItem>,
}