use sqlx::{mysql::MySqlPoolOptions, MySql, Pool};
use std::time::Duration;

pub async fn init_db() -> Pool<MySql> {
    // Ambil DATABASE_URL dari .env
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not set");

    println!("🔍 Connecting to MySQL at");

    // Coba buat pool dengan timeout lebih lama
    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(30)) // timeout dinaikkan
        .connect(&db_url)
        .await;

    match pool {
        Ok(p) => {
            println!("✅ Connected to MySQL successfully!");
            p
        }
        Err(e) => {
            // Tampilkan error detail supaya gampang debug
            eprintln!("❌ Failed to connect to MySQL: {:?}", e);
            eprintln!("💡 Tips:");
            eprintln!("  - Cek apakah server MySQL bisa diakses dari host ini");
            eprintln!("  - Cek SSL mode (MySQL8 default mungkin pakai caching_sha2_password)");
            eprintln!("  - Cek firewall atau network timeout");
            std::process::exit(1);
        }
    }
}
