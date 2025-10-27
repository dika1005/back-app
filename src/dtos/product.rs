// src/dtos/product_dto.rs

use serde::{Deserialize, Serialize};
// Use f64 for price to avoid bigdecimal/sqlx trait issues for now

// Data yang diterima saat membuat Produk baru (POST)
#[derive(Debug, Deserialize)]
pub struct NewRodProductDto {
    pub name: String,
    pub description: String,
    pub category_id: i32, // Foreign Key
    pub rod_length: String,
    pub line_weight: String,
    pub cast_weight: String,
    pub action: String,
    pub material: String,
    pub power: String,
    pub reel_size: String,
    pub price: f64,
}

// Data yang dikirimkan saat response Produk
// Sesuaikan field ini agar sesuai dengan struktur tabel MySQL Anda
#[derive(Debug, Serialize, sqlx::FromRow)] // sqlx::FromRow digunakan untuk mapping langsung dari query DB
pub struct RodProduct {
    pub id: i64, // BIGINT di MySQL, jadi gunakan i64
    pub name: String,
    pub description: String,
    pub category_id: i32,
    pub rod_length: String,
    pub line_weight: String,
    pub cast_weight: String,
    pub action: String,
    pub material: String,
    pub power: String,
    pub reel_size: String,
    pub price: f64, // DECIMAL(10, 2) mapped to f64
}

// Struktur untuk detail produk dengan nama kategori (untuk GET All/Detail)
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct RodProductDetail {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub category_name: String, // Nama Kategori (hasil JOIN)
    pub rod_length: String,
    pub line_weight: String,
    pub cast_weight: String,
    pub action: String,
    pub material: String,
    pub power: String,
    pub reel_size: String,
    pub price: f64, // use f64 for price
}