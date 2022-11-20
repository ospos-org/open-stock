use crate::methods::{Url, TagList, DiscountValue};

use super::{VariantCategoryList, VariantIdTag};

struct Product {
    pub name: String,
    pub variants: VariantCategoryList,
    pub image: Url,
    pub sku: String,
    pub loyalty_discount: DiscountValue,
    pub tags: TagList
}

#[derive(Debug)]
pub struct ProductPurchase {
    // Includes variant
    pub product_code: ProductCode,
    pub variant: VariantIdTag,
    pub discount: DiscountValue,

    pub product_cost: i128,
    pub quantity: i128,
}

#[derive(Debug)]
pub struct ProductExchange {
    pub product_code: ProductCode,
    pub variant: VariantIdTag,
    pub quantity: i128,
}

pub type ProductCode = String;
pub type ProductPurchaseList = Vec<ProductPurchase>;