use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct PaginationParams {
    #[serde(default = "default_page")]
    #[schema(example = 1)]
    pub page: u32,
    #[serde(default = "default_limit")]
    #[schema(example = 10)]
    pub per_page: u32,
}

fn default_page() -> u32 {
    1
}

fn default_limit() -> u32 {
    10
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            page: 1,
            per_page: 10,
        }
    }
}

impl PaginationParams {
    pub fn offset(&self) -> u32 {
        (self.page.saturating_sub(1)) * self.per_page
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub pagination: PaginationMeta,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PaginationMeta {
    #[schema(example = 1)]
    pub current_page: u32,
    #[schema(example = 10)]
    pub per_page: u32,
    #[schema(example = 100)]
    pub total: u32,
    #[schema(example = 10)]
    pub total_pages: u32,
}

impl PaginationMeta {
    pub fn new(current_page: u32, per_page: u32, total: u32) -> Self {
        let total_pages = if per_page == 0 {
            0
        } else {
            (total as f32 / per_page as f32).ceil() as u32
        };
        Self {
            current_page,
            per_page,
            total,
            total_pages,
        }
    }
}