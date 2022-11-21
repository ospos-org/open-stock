use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiscountValue {
    Percentage(i128), Absolute(i128)
}