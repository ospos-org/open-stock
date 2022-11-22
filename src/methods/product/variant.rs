use serde::{Deserialize, Serialize};

use crate::methods::{StockList, HistoryList, Url};

pub type VariantIdTag = Vec<VariantId>;
type VariantId = String;

pub type VariantCategoryList = Vec<VariantCategory>;

#[derive(Deserialize, Serialize)]
pub struct VariantCategory {
    pub category: String,
    pub variants: Variant
}

#[derive(Deserialize, Serialize)]
pub struct Variant {
    pub name: String,
    pub stock: StockList,
    pub images: Vec<Url>,
    pub marginal_price: i32,
    pub variant_code: String,
    pub order_history: HistoryList
}