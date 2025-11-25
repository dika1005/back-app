use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct KategoriDto {
    #[schema(example = 1)]
    pub id: i32,
    #[schema(example = "Joran Casting")]
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "name": "Joran Spinning"
}))]
pub struct NewKategoriDto {
    pub name: String,
}
