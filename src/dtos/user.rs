use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct UpdateProfile {
    pub name: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
}
