use serde::Deserialize;

#[derive(Deserialize)]
pub struct UpdateRoleRequest {
    pub role: String,
}
