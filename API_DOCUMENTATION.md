# API Documentation

Dokumentasi API telah berhasil ditambahkan menggunakan **utoipa** (OpenAPI 3.0).

## Cara Melihat Dokumentasi

### Opsi 1: Swagger UI Lokal (Recommended)

1. Jalankan server:
   ```bash
   cargo run
   ```

2. Buka file `swagger-ui.html` di browser:
   ```bash
   start swagger-ui.html  # Windows
   # atau double-click file swagger-ui.html
   ```

3. Swagger UI akan menampilkan semua endpoint dengan:
   - Request/Response schemas
   - Try it out feature (test API langsung)
   - Authentication support
   - Example values

### Opsi 2: OpenAPI JSON

Akses OpenAPI specification dalam format JSON:
```
http://127.0.0.1:3001/api-docs/openapi.json
```

Copy URL tersebut dan paste ke:
- **Swagger Editor**: https://editor.swagger.io
- **Postman**: Import > Link > Paste URL
- **Insomnia**: Create > Import from URL

### Opsi 3: VS Code Extension

Install extension **OpenAPI (Swagger) Editor** di VS Code:
1. Install extension: `42Crunch.vscode-openapi`
2. Buka file OpenAPI JSON dari endpoint
3. Preview dengan Swagger UI built-in

## Endpoints yang Terdokumentasi

### Authentication (`/auth`)
- ✅ `POST /auth/register` - Register user baru
- ✅ `POST /auth/login` - Login dengan email/password

### Products (`/products`)
- ✅ `GET /products` - Get all products (paginated)
- ✅ `GET /products/{id}` - Get product by ID
- ✅ `POST /products/create` - Create product (admin only)

### Categories (`/categories`)
- ✅ `GET /categories` - Get all categories
- ✅ `POST /categories/create` - Create category (admin only)

### Orders (`/orders`)
- ✅ `POST /orders/checkout` - Create order & get payment URL

### Chatbot (`/chatbot`)
- ✅ `POST /chatbot/recommend` - Get AI product recommendations

## Fitur Dokumentasi

### 1. Request/Response Schemas
Semua DTO (Data Transfer Objects) sudah dianotasi dengan `#[derive(ToSchema)]` dan contoh values:

```rust
#[derive(Deserialize, ToSchema)]
pub struct LoginRequest {
    #[schema(example = "john@example.com")]
    pub email: String,
    #[schema(example = "Password123!")]
    pub password: String,
}
```

### 2. Path Documentation
Setiap handler function memiliki dokumentasi lengkap:

```rust
#[utoipa::path(
    post,
    path = "/auth/login",
    tag = "auth",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful"),
        (status = 401, description = "Invalid credentials")
    )
)]
pub async fn login_handler(...) { }
```

### 3. Security Schemes
JWT authentication sudah terdefinisi:
- Bearer token di header `Authorization: Bearer <token>`
- Cookie authentication via `jwt` cookie

### 4. Tags & Organization
Endpoints dikelompokkan berdasarkan fungsi:
- 🔐 `auth` - Authentication
- 🎣 `products` - Product management
- 📂 `categories` - Category management  
- 🛒 `orders` - Order & payment
- 🤖 `chatbot` - AI recommendations

## Menambah Dokumentasi untuk Endpoint Baru

### 1. Tambahkan ToSchema ke DTO

```rust
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct NewDto {
    #[schema(example = "Example value")]
    pub field: String,
}
```

### 2. Tambahkan utoipa::path ke Handler

```rust
#[utoipa::path(
    get,
    path = "/your-endpoint",
    tag = "your-tag",
    responses(
        (status = 200, description = "Success"),
    )
)]
pub async fn your_handler() { }
```

### 3. Register di main.rs

Tambahkan ke `#[openapi(...)]` macro:

```rust
#[openapi(
    paths(
        // ... existing paths
        handlers::your_module::your_handler,
    ),
    components(
        schemas(
            // ... existing schemas
            dtos::your_module::NewDto,
        )
    )
)]
```

### 4. Rebuild

```bash
cargo build
```

## Troubleshooting

### Server tidak mau jalan
Pastikan MySQL sudah running dan `.env` sudah dikonfigurasi:
```env
DATABASE_URL=mysql://user:pass@localhost/dbname
JWT_SECRET=your-secret-key
MIDTRANS_SERVER_KEY=...
GROQ_API_KEY=...
```

### OpenAPI JSON kosong/error
1. Check compilation errors: `cargo build`
2. Pastikan semua DTOs memiliki `#[derive(ToSchema)]`
3. Pastikan semua handlers terdaftar di `#[openapi(paths(...))]`

### Swagger UI tidak load
1. Pastikan server running di `http://127.0.0.1:3001`
2. Check browser console untuk CORS errors
3. Test endpoint langsung: `curl http://127.0.0.1:3001/api-docs/openapi.json`

## Dependencies

```toml
utoipa = { version = "5", features = ["axum_extras", "chrono"] }
utoipa-swagger-ui = { version = "8", features = ["axum"] }
```

## Resources

- **utoipa docs**: https://docs.rs/utoipa/
- **OpenAPI Spec**: https://swagger.io/specification/
- **Swagger Editor**: https://editor.swagger.io/
