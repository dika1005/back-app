// src/models/product.rs

use sqlx::{MySql, Pool};
use crate::dtos::product::{NewRodProductDto, RodProductDetail, RodProduct};

// Catatan: RodProduct di sini adalah nama struct yang digunakan untuk mengelompokkan method.

impl RodProduct {
    // --- 1. INSERT (CREATE) ---
    pub async fn insert(
        pool: &Pool<MySql>,
        new_product: NewRodProductDto
    ) -> Result<u64, sqlx::Error> {
        let result = sqlx
            ::query(
                r#"
                INSERT INTO products (name, description, category_id, rod_length, line_weight, cast_weight, 
                                      action, material, power, reel_size, price)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#
            )
            .bind(new_product.name)
            .bind(new_product.description)
            .bind(new_product.category_id)
            .bind(new_product.rod_length)
            .bind(new_product.line_weight)
            .bind(new_product.cast_weight)
            .bind(new_product.action)
            .bind(new_product.material)
            .bind(new_product.power)
            .bind(new_product.reel_size)
            .bind(new_product.price)
            .execute(pool).await?;

        Ok(result.last_insert_id())
    }

    // --- 2. FIND ALL DETAILS (READ ALL) ---
    pub async fn find_all_details(
        pool: &Pool<MySql>
    ) -> Result<Vec<RodProductDetail>, sqlx::Error> {
        sqlx
            ::query_as::<_, RodProductDetail>(
                r#"
                SELECT 
                     p.id, p.name, p.description, 
                     c.name as category_name, 
                     p.rod_length, p.line_weight, p.cast_weight, p.action, p.material, p.power, p.reel_size, 
                     p.price
                FROM products p
                JOIN kategori c ON p.category_id = c.id
                ORDER BY p.id DESC
                "#
            )
            .fetch_all(pool).await
    }

    // --- 3. FIND DETAIL BY ID (READ DETAIL) ---
    pub async fn find_detail_by_id(
        pool: &Pool<MySql>,
        id: i64
    ) -> Result<Option<RodProductDetail>, sqlx::Error> {
        sqlx
            ::query_as::<_, RodProductDetail>(
                r#"
                SELECT
                     p.id, p.name, p.description, 
                     c.name as category_name, 
                     p.rod_length, p.line_weight, p.cast_weight, p.action, p.material, p.power, p.reel_size, 
                     p.price
                FROM products p
                JOIN kategori c ON p.category_id = c.id
                WHERE p.id = ?
                "#
            )
            .bind(id)
            .fetch_optional(pool).await
    }

    // --- 4. UPDATE ---
    pub async fn update(
        pool: &Pool<MySql>,
        id: i64,
        updated_product: NewRodProductDto
    ) -> Result<(), sqlx::Error> {
        sqlx
            ::query(
                r#"
                UPDATE products SET 
                    name = ?, description = ?, category_id = ?, rod_length = ?, line_weight = ?, cast_weight = ?, 
                    action = ?, material = ?, power = ?, reel_size = ?, price = ?
                WHERE id = ?
                "#
            )
            .bind(updated_product.name)
            .bind(updated_product.description)
            .bind(updated_product.category_id)
            .bind(updated_product.rod_length)
            .bind(updated_product.line_weight)
            .bind(updated_product.cast_weight)
            .bind(updated_product.action)
            .bind(updated_product.material)
            .bind(updated_product.power)
            .bind(updated_product.reel_size)
            .bind(updated_product.price)
            .bind(id)
            .execute(pool).await?;
        Ok(())
    }

    // --- 5. DELETE ---
    pub async fn delete(pool: &Pool<MySql>, id: i64) -> Result<u64, sqlx::Error> {
        let result = sqlx
            ::query("DELETE FROM products WHERE id = ?")
            .bind(id)
            .execute(pool).await?;
        Ok(result.rows_affected())
    }
}