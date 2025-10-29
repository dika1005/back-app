// src/dtos/product_dto.rs

use serde::{Deserialize, Serialize};

// Data yang diterima saat membuat Produk baru (POST)
#[derive(Debug, Deserialize)]
pub struct NewRodProductDto {
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
    pub price: f64,
    //  <--- (Tag Gambar disarankan di DTO)
    
    // AKTIFKAN FIELD IMAGE_URL DI DTO INPUT
    pub image_url: Option<String>, 
}

// Data yang dikirimkan saat response Produk
#[derive(Debug, Serialize, sqlx::FromRow)] 
pub struct RodProduct {
    pub id: i64,
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
    pub price: f64,
    // AKTIFKAN FIELD IMAGE_URL DI STRUCT MODEL
    pub image_url: Option<String>, 
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
    pub price: f64, 
    // AKTIFKAN FIELD IMAGE_URL DI STRUCT DETAIL
    pub image_url: Option<String>,
}