use crate::methods::{StockList, HistoryList};

pub type VariantIdTag = Vec<VariantId>;
type VariantId = String;

pub type VariantCategoryList = Vec<VariantCategory>;
pub type VariantList = Vec<Variant>;

pub struct VariantCategory {
    pub category: String,
    pub variants: Variant
}

pub struct Variant {
    pub name: String,
    pub stock: StockList,
    pub marginal_price: i128,
    pub variant_code: String,
    pub order_history: HistoryList
}