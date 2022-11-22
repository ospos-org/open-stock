use std::{str::FromStr, fmt::Display};

use sea_orm::{DbConn, DbErr, EntityTrait, Set};
use serde::{Serialize, Deserialize};
use serde_json::json;
use crate::{methods::{Url, TagList, DiscountValue}, entities::{sea_orm_active_enums::TransactionType, products}};
use super::{VariantCategoryList, VariantIdTag};
use crate::entities::prelude::Products;

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

impl Product {
    pub async fn insert(pdt: Product, db: &DbConn) -> Result<(), DbErr> {
        let insert_crud = products::ActiveModel {
            sku: Set(pdt.sku),
            name: Set(pdt.name),
            variants: Set(json!(pdt.variants)),
            loyalty_discount: Set(DiscountValue::to_string(&pdt.loyalty_discount)),
            images: Set(json!(pdt.images)),
            tags: Set(json!(pdt.tags)),
            description: Set(pdt.description),
            specifications: Set(json!(pdt.specifications)),
        };

        match Products::insert(insert_crud).exec(db).await {
            Ok(_) => Ok(()),
            Err(err) => Err(err)
        }
    }

    pub async fn fetch_by_id(id: &str, db: &DbConn) -> Result<Product, DbErr> {
        let pdt = Products::find_by_id(id.to_string()).one(db).await?;
        let p = pdt.unwrap();

        Ok(Product { 
            name: p.name, 
            variants: serde_json::from_value::<VariantCategoryList>(p.variants).unwrap(), 
            sku: p.sku, 
            loyalty_discount: DiscountValue::from_str(&p.loyalty_discount).unwrap(), 
            images: serde_json::from_value::<Vec<Url>>(p.images).unwrap(), 
            tags: serde_json::from_value::<TagList>(p.tags).unwrap(), 
            description: p.description, 
            specifications: serde_json::from_value::<Vec<(String, String)>>(p.specifications).unwrap() 
        })
    }
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

impl Display for ProductExchange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}-{} x{}", self.method_type, self.product_code, self.variant.concat(), self.quantity)
    }
}

pub type ProductCode = String;
pub type ProductPurchaseList = Vec<ProductPurchase>;