use sqlx::{ MySql, Pool };
use serde::{ Serialize, Deserialize };

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub password: String,
    pub address: Option<String>,
    pub role: String,
}

impl User {
    // Cek apakah email sudah terdaftar
    pub async fn exists_by_email(pool: &Pool<MySql>, email: &str) -> Result<bool, sqlx::Error> {
        let count: Option<i64> = sqlx
            ::query_scalar("SELECT COUNT(*) FROM users WHERE email = ?")
            .bind(email)
            .fetch_optional(pool).await?
            .flatten();
        Ok(count.unwrap_or(0) > 0)
    }

    // Masukkan user baru
    pub async fn insert(
        pool: &Pool<MySql>,
        name: &str,
        email: &str,
        password: &str,
        address: Option<&String>
    ) -> Result<(), sqlx::Error> {
        sqlx
            ::query(
                "INSERT INTO users (name, email, password, address, role) VALUES (?, ?, ?, ?, ?)"
            )
            .bind(name)
            .bind(email)
            .bind(password)
            .bind(address)
            .bind("user")
            .execute(pool).await?;
        Ok(())
    }

    // Cari user berdasarkan email
    pub async fn find_by_email(
        pool: &Pool<MySql>,
        email: &str
    ) -> Result<Option<User>, sqlx::Error> {
        sqlx
            ::query_as::<_, User>("SELECT * FROM users WHERE email = ?")
            .bind(email)
            .fetch_optional(pool).await
    }

    // Update role pengguna
    pub async fn update_role(
        pool: &Pool<MySql>,
        email: &str,
        new_role: &str
    ) -> Result<(), sqlx::Error> {
        sqlx
            ::query("UPDATE users SET role = ? WHERE email = ?")
            .bind(new_role)
            .bind(email)
            .execute(pool).await?;
        Ok(())
    }

    // Logika untuk Google Auth: Insert jika user belum ada
    pub async fn upsert_google_user(
        pool: &Pool<MySql>,
        email: &str,
        name: &str
    ) -> Result<(), sqlx::Error> {
        let is_existing = Self::exists_by_email(pool, email).await?;

        if !is_existing {
            // Gunakan insert, dengan password kosong dan role default 'user'
            // Gunakan None::<String> untuk address jika field address Anda di DB bisa NULL
            sqlx
                ::query(
                    "INSERT INTO users (name, email, password, address, role) VALUES (?, ?, ?, ?, ?)"
                )
                .bind(name)
                .bind(email)
                .bind("") // Password kosong karena ini login via Google
                .bind(None::<String>) // Alamat kosong
                .bind("user")
                .execute(pool).await?;
        }

        Ok(())
    }
}
