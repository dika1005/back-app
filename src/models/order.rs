use sqlx::{ MySql, Pool, Transaction };
use crate::dtos::order::{ NewOrderDto, Order, OrderItem }; // Order adalah struct utama
use crate::dtos::product::RodProduct;
use crate::middleware::auth::AuthUser;

// Catatan: Asumsi RodProduct::find_by_id sudah ada dan menerima &mut Transaction

impl Order {
    // --- 1. PROSES CHECKOUT (CREATE ORDER) ---
    pub async fn create_order(
        pool: &Pool<MySql>,
        user_id: i64,
        new_order: NewOrderDto
    ) -> Result<i64, sqlx::Error> {
        let mut tx = pool.begin().await?;
        let mut total_amount = 0.0;
        let mut items_to_insert: Vec<(i64, i32, f64)> = Vec::new();

        for item_dto in new_order.items.into_iter() {
            let product = RodProduct::find_by_id(&mut *tx, item_dto.product_id).await.map_err(
                |_| sqlx::Error::RowNotFound
            )?;

            let item_price = product.price * (item_dto.quantity as f64);
            total_amount += item_price;
            items_to_insert.push((item_dto.product_id, item_dto.quantity, product.price));
        }

        // 2. Insert ke tabel orders
        let order_result = sqlx
            ::query(
                r#"
            INSERT INTO orders (user_id, total_amount, shipping_address, payment_method, status)
            VALUES (?, ?, ?, ?, 'PENDING')
            "#
            )
            .bind(user_id)
            .bind(total_amount)
            .bind(new_order.shipping_address)
            .bind(new_order.payment_method)
            .execute(&mut *tx).await?;

        let order_id = order_result.last_insert_id() as i64; 

        // 3. Insert ke tabel order_items
        for (product_id, quantity, price_at_order) in items_to_insert {
            sqlx
                ::query(
                    r#"
                INSERT INTO order_items (order_id, product_id, quantity, price_at_order)
                VALUES (?, ?, ?, ?)
                "#
                )
                .bind(order_id)
                .bind(product_id)
                .bind(quantity)
                .bind(price_at_order)
                .execute(&mut *tx).await?;
        }
        tx.commit().await?;
        Ok(order_id)
    }

    // --- 2. LOGIKA PEMBAYARAN (UPDATE STATUS ORDER) ---
    pub async fn process_payment(
        pool: &Pool<MySql>,
        order_id: i64,
        is_success: bool
    ) -> Result<u64, sqlx::Error> {
        let new_status = if is_success { "PAID" } else { "FAILED" };

        let result = sqlx
            ::query(
                r#"
            UPDATE orders SET status = ?
            WHERE id = ? AND status = 'PENDING'
            "#
            )
            .bind(new_status)
            .bind(order_id)
            .execute(pool).await?;

        Ok(result.rows_affected())
    }

    // --- 3. FUNGSI BARU CEK STATUS LOKAL ---
   pub async fn find_status_by_id(
        pool: &Pool<MySql>,
        order_id: i64
    ) -> Result<String, sqlx::Error> {
        let order_record = sqlx::query_as!(
            Order, // Menggunakan struct Order untuk query_as
            r#"
            SELECT
                id,
                user_id,
                total_amount,
                shipping_address,
                payment_method,
                status,
                order_date
            FROM orders
            WHERE id = ?
            "#,
            order_id
        )
        .fetch_one(pool).await?;

        // Mengembalikan hanya status
        Ok(order_record.status)
    }
}
