use sqlx::{MySql, Pool, Transaction};
use crate::dtos::order::{NewOrderDto, Order, OrderItem};
use crate::dtos::product::RodProduct; // Harus diimport untuk mengambil harga produk
use crate::middleware::auth::AuthUser;

// Catatan: Asumsi RodProduct::find_by_id sudah ada dan menerima &mut Transaction

impl Order {
    // --- 1. PROSES CHECKOUT (CREATE ORDER) ---
    // Fungsi ini harus dijalankan dalam transaksi database!
    pub async fn create_order(
        pool: &Pool<MySql>,
        user_id: i64,
        new_order: NewOrderDto,
    ) -> Result<i64, sqlx::Error> {
        // Mulai Transaksi
        let mut tx = pool.begin().await?;

        // 1. Hitung Total Harga, Validasi, dan Persiapan Insert Item
        let mut total_amount = 0.0;
        
        // Kita butuh vektor item yang sudah divalidasi dan memiliki harga saat ini
        let mut items_to_insert: Vec<(i64, i32, f64)> = Vec::new(); // (product_id, quantity, price_at_order)

        for item_dto in new_order.items.into_iter() {
            // Dapatkan harga produk saat ini. RodProduct::find_by_id harus mengembalikan RodProduct
            // RodProduct::find_by_id harus menerima &mut Transaction untuk konsistensi.
            let product = RodProduct::find_by_id(&mut *tx, item_dto.product_id).await
                .map_err(|_| sqlx::Error::RowNotFound)?; // Menangkap error jika produk tidak ditemukan

            // Hitung sub-total dan total
            let item_price = product.price * item_dto.quantity as f64;
            total_amount += item_price;

            items_to_insert.push((
                item_dto.product_id, 
                item_dto.quantity, 
                product.price
            ));
        }
        
        // 2. Insert ke tabel orders
        let order_result = sqlx::query(
            r#"
            INSERT INTO orders (user_id, total_amount, shipping_address, payment_method, status)
            VALUES (?, ?, ?, ?, 'PENDING')
            "#,
        )
    // bind user_id dari parameter
    .bind(user_id)
        .bind(total_amount)
        .bind(new_order.shipping_address)
        .bind(new_order.payment_method)
        .execute(&mut *tx)
        .await?;

        let order_id = order_result.last_insert_id() as i64; // Order ID adalah i64

        // 3. Insert ke tabel order_items
        for (product_id, quantity, price_at_order) in items_to_insert {
            sqlx::query(
                r#"
                INSERT INTO order_items (order_id, product_id, quantity, price_at_order)
                VALUES (?, ?, ?, ?)
                "#,
            )
            .bind(order_id)
            .bind(product_id)
            .bind(quantity)
            .bind(price_at_order)
            .execute(&mut *tx)
            .await?;
        }

        // 4. Commit Transaksi (jika semua langkah di atas berhasil)
        tx.commit().await?;

        Ok(order_id)
    }

    // --- 2. LOGIKA PEMBAYARAN (UPDATE STATUS ORDER) ---
    pub async fn process_payment(
        pool: &Pool<MySql>,
        order_id: i64,
        is_success: bool,
    ) -> Result<u64, sqlx::Error> {
        let new_status = if is_success { "PAID" } else { "FAILED" };
        
        let result = sqlx::query(
            r#"
            UPDATE orders SET status = ?
            WHERE id = ? AND status = 'PENDING'
            "#,
        )
        .bind(new_status)
        .bind(order_id)
        .execute(pool)
        .await?;

        Ok(result.rows_affected())
    }
}