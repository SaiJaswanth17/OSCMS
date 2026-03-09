use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginationParams {
    #[serde(default = "default_page")]
    pub page: u64,
    #[serde(default = "default_limit")]
    pub limit: u64,
}

fn default_page() -> u64 { 1 }
fn default_limit() -> u64 { 20 }

impl PaginationParams {
    pub fn offset(&self) -> u64 {
        (self.page.saturating_sub(1)) * self.limit
    }

    pub fn limit_clamped(&self) -> u64 {
        self.limit.min(100)
    }
}

#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T: Serialize> {
    pub data: Vec<T>,
    pub meta: PaginationMeta,
}

#[derive(Debug, Serialize)]
pub struct PaginationMeta {
    pub total: u64,
    pub page: u64,
    pub limit: u64,
    pub total_pages: u64,
    pub has_next: bool,
    pub has_prev: bool,
}

impl<T: Serialize> PaginatedResponse<T> {
    pub fn new(data: Vec<T>, total: u64, params: &PaginationParams) -> Self {
        let limit = params.limit_clamped();
        let total_pages = (total as f64 / limit as f64).ceil() as u64;
        Self {
            data,
            meta: PaginationMeta {
                total,
                page: params.page,
                limit,
                total_pages,
                has_next: params.page < total_pages,
                has_prev: params.page > 1,
            },
        }
    }
}
