use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::methods::{StockList, HistoryList, Url};

pub type VariantIdTag = Vec<VariantId>;
type VariantId = String;

pub type VariantCategoryList = Vec<VariantCategory>;

#[derive(Deserialize, Serialize, Clone)]
pub struct VariantCategory {
    pub category: String,
    pub variants: Vec<Variant>
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Variant {
    pub name: String,
    pub stock: StockList,
    pub images: Vec<Url>,
    pub marginal_price: i32,
    pub variant_code: String,
    pub order_history: HistoryList
}

impl Display for Variant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,
            "\t{} ({}) ${}", 
            self.name, self.variant_code, self.marginal_price 
        )
    }
}