use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiscountValue {
    Percentage(i32), Absolute(i32)
}