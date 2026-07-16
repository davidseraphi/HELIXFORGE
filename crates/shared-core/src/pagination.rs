//! Cursor-style pagination types.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageRequest {
    pub limit: u32,
    #[serde(default)]
    pub cursor: Option<String>,
}

impl Default for PageRequest {
    fn default() -> Self {
        Self {
            limit: 50,
            cursor: None,
        }
    }
}

impl PageRequest {
    pub fn clamped(self) -> Self {
        Self {
            limit: self.limit.clamp(1, 500),
            cursor: self.cursor,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub next_cursor: Option<String>,
    pub total_hint: Option<u64>,
}

impl<T> Page<T> {
    pub fn empty() -> Self {
        Self {
            items: vec![],
            next_cursor: None,
            total_hint: Some(0),
        }
    }
}
