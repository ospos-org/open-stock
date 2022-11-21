use serde::{Serialize, Deserialize};
use crate::methods::{Url, TagList, DiscountValue, TransactionType};
use super::{VariantCategoryList, VariantIdTag};

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

    pub product_cost: i128,
    pub quantity: i128,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProductExchange {
    pub method_type: TransactionType,
    pub product_code: ProductCode,
    pub variant: VariantIdTag,
    pub quantity: i128,
}

pub type ProductCode = String;
pub type ProductPurchaseList = Vec<ProductPurchase>;