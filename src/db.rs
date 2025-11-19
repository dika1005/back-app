use sqlx::{MySql, Pool, mysql::MySqlPoolOptions};
use std::time::Duration;

pub async fn init_db() -> Pool<MySql> {
    // Ambil DATABASE_URL dari .env
    let db_url = std::env::var("DATABASE_URL").expect("⚠️ DATABASE_URL belum diset di file .env");

    println!("🔍 Menghubungkan ke MySQL...");

    // Buat koneksi pool
    match MySqlPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(30))
        .connect(&db_url)
        .await
    {
        Ok(pool) => {
            println!("✅ Koneksi ke MySQL berhasil!");
            pool
        }
        Err(e) => {
            eprintln!("❌ Gagal konek ke MySQL: {:?}", e);
            eprintln!("💡 Tips:");
            eprintln!("  - Pastikan MySQL sedang jalan (service aktif)");
            eprintln!("  - Cek DATABASE_URL di .env, contoh:");
            eprintln!("    mysql://user:password@localhost:3306/nama_database");
            eprintln!("  - Coba matikan SSL dengan ?ssl-mode=DISABLED jika error SSL");
            std::process::exit(1);
        }
    }
}
