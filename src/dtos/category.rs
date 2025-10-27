// src/dtos/kategori_dto.rs

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// Data yang diterima saat membuat Kategori baru (POST)
#[derive(Debug, Deserialize)]
pub struct NewKategoriDto {
    pub name: String, // Nama kategori (misal: "Sungai")
}

// Data yang dikirimkan saat response Kategori
#[derive(Debug, Serialize, FromRow)]
pub struct KategoriDto {
    pub id: i32,
    pub name: String,
}   