// src/models/category.rs

use sqlx::{MySql, Pool};
use crate::dtos::category::{KategoriDto, NewKategoriDto};
// use sqlx::FromRow; // tidak dipakai di file ini (jika diperlukan, import di DTO yang sesuai)

// Asumsi: Kita buat struct KategoriModel untuk mengelompokkan method
pub struct KategoriModel; 

// Tambahkan FromRow di struct DTO jika belum ada
// #[derive(Debug, Serialize, FromRow)] 
// pub struct KategoriDto { ... }


impl KategoriModel {
    // --- 1. CREATE (Insert) ---
    pub async fn insert(pool: &Pool<MySql>, new_kategori: NewKategoriDto) -> Result<u64, sqlx::Error> {
        let result = sqlx::query!("INSERT INTO kategori (name) VALUES (?)", new_kategori.name)
            .execute(pool).await?;
        Ok(result.last_insert_id())
    }

    // --- 2. FIND ALL (READ ALL) ---
    pub async fn find_all(pool: &Pool<MySql>) -> Result<Vec<KategoriDto>, sqlx::Error> {
        sqlx::query_as::<_, KategoriDto>("SELECT id, name FROM kategori ORDER BY id ASC")
            .fetch_all(pool).await
    }

    // --- 3. FIND BY ID (READ DETAIL) ---
    pub async fn find_by_id(pool: &Pool<MySql>, id: i32) -> Result<Option<KategoriDto>, sqlx::Error> {
        sqlx::query_as::<_, KategoriDto>("SELECT id, name FROM kategori WHERE id = ?")
            .bind(id)
            .fetch_optional(pool).await
    }

    // --- 4. UPDATE ---
    pub async fn update(pool: &Pool<MySql>, id: i32, updated_name: &str) -> Result<u64, sqlx::Error> {
        let result = sqlx::query!("UPDATE kategori SET name = ? WHERE id = ?", updated_name, id)
            .execute(pool).await?;
        Ok(result.rows_affected())
    }

    // --- 5. DELETE ---
    pub async fn delete(pool: &Pool<MySql>, id: i32) -> Result<u64, sqlx::Error> {
        let result = sqlx::query!("DELETE FROM kategori WHERE id = ?", id)
            .execute(pool).await?;
        Ok(result.rows_affected())
    }
}

// Convenience module-level wrappers used by handlers (names expected by handlers)
// Note: convenience wrappers removed because handlers call KategoriModel::* directly.
// If other modules need wrappers with these names, re-add them or expose via pub API.