use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
#[schema(example = json!({
    "role": "admin"
}))]
pub struct UpdateRoleRequest {
    pub role: String,
}
