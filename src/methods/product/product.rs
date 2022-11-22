use serde::{Serialize, Deserialize};
use crate::{methods::{Url, TagList, DiscountValue}, entities::sea_orm_active_enums::TransactionType};
use super::{VariantCategoryList, VariantIdTag};

#[derive(Deserialize, Serialize)]
pub struct Product {
    pub name: String,
    pub variants: VariantCategoryList,
    pub sku: String,
    
    pub loyalty_discount: DiscountValue,

    pub images: Vec<Url>,
    pub tags: TagList,
    pub description: String,
    pub specifications: Vec<(String, String)>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductPurchase {
    // Includes variant
    pub product_code: ProductCode,
    pub variant: VariantIdTag,
    pub discount: DiscountValue,

    pub product_cost: i32,
    pub quantity: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProductExchange {
    pub method_type: TransactionType,
    pub product_code: ProductCode,
    pub variant: VariantIdTag,
    pub quantity: i32,
}

pub type ProductCode = String;
pub type ProductPurchaseList = Vec<ProductPurchase>;