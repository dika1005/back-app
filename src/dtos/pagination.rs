use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    #[serde(default = "default_page")]
    pub page: Option<u32>,
    #[serde(default = "default_limit")]
    pub per_page: Option<u32>,
}

fn default_page() -> Option<u32> {
    1
}
fn default_limit() -> Option<u32> {
    10
}

impl PaginationParams {
    pub fn offset(&self) -> u32 {
        (self.page.saturating_sub(1)) * self.limit
    }
}

#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T>{
    pub data: Vec<T>,
    pub pagination: PaginationMeta,
}

#[derive(Debug, Serialize)]
pub struct PaginationMeta {
    pub current_page: u32,
    pub per_page: u32,
    pub total: u32,
    pub total_pages: u32,
}

impl PaginationMeta {
    pub fn new(current_page: u32, per_page: u32, total: u32) -> Self {
        let total_pages = (total as f32 / per_page as f32).ceil() as u32;
        self {
            current_page,
            per_page,
            total,
            total_pages,
        
        }
    }
}