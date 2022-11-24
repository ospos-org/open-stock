use std::{str::FromStr, fmt::Display};

use sea_orm::{DbConn, DbErr, EntityTrait, Set, QuerySelect, ColumnTrait, InsertResult};
use serde::{Serialize, Deserialize};
use serde_json::json;

use crate::{methods::{Url, TagList, DiscountValue}, entities::{sea_orm_active_enums::TransactionType, products}};
use super::{VariantCategoryList, VariantIdTag};
use crate::entities::prelude::Products;

#[derive(Deserialize, Serialize, Clone)]
/// A product, containing a list of `Vec<Variant>`, an identifiable `sku` along with identifying information such as `tags`, `description` and `specifications`.
/// > Stock-relevant information about a product is kept under each variant, thus allowing for modularity of different variants and a fine-grained control over your inventory. 
pub struct Product {
    pub name: String,
    pub company: String,
    pub variants: VariantCategoryList,
    pub sku: String,
    
    pub loyalty_discount: DiscountValue,

    pub images: Vec<Url>,
    pub tags: TagList,
    pub description: String,
    pub specifications: Vec<(String, String)>,
}

impl Display for Product {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let variant_categories: String = self.variants
            .iter()
            .map(|p| {
                let variants: String = p.variants
                    .iter()
                    .map(|p| {
                        format!("{}\n", p)
                    }).collect();

                format!(
                    "{}(s):\n{}", 
                    p.category, variants
                )
            }).collect();

        write!(f, "{}: {} ({})\n{}", self.sku, self.name, self.company, variant_categories)
    }
}

impl Product {
    pub async fn insert(pdt: Product, db: &DbConn) -> Result<InsertResult<products::ActiveModel>, DbErr> {
        let insert_crud = products::ActiveModel {
            sku: Set(pdt.sku),
            name: Set(pdt.name),
            company: Set(pdt.company),
            variants: Set(json!(pdt.variants)),
            loyalty_discount: Set(DiscountValue::to_string(&pdt.loyalty_discount)),
            images: Set(json!(pdt.images)),
            tags: Set(json!(pdt.tags)),
            description: Set(pdt.description),
            specifications: Set(json!(pdt.specifications)),
        };

        match Products::insert(insert_crud).exec(db).await {
            Ok(res) => Ok(res),
            Err(err) => Err(err)
        }
    }

    pub async fn fetch_by_id(id: &str, db: &DbConn) -> Result<Product, DbErr> {
        let pdt = Products::find_by_id(id.to_string()).one(db).await?;
        let p = pdt.unwrap();

        Ok(Product { 
            name: p.name, 
            company: p.company,
            variants: serde_json::from_value::<VariantCategoryList>(p.variants).unwrap(), 
            sku: p.sku, 
            loyalty_discount: DiscountValue::from_str(&p.loyalty_discount).unwrap(), 
            images: serde_json::from_value::<Vec<Url>>(p.images).unwrap(), 
            tags: serde_json::from_value::<TagList>(p.tags).unwrap(), 
            description: p.description, 
            specifications: serde_json::from_value::<Vec<(String, String)>>(p.specifications).unwrap() 
        })
    }

    pub async fn fetch_by_name(name: &str, db: &DbConn) -> Result<Vec<Product>, DbErr> {
        let res = products::Entity::find()
            .having(products::Column::Name.contains(name))
            .all(db).await?;
            
        let mapped = res.iter().map(|p| 
            Product { 
                name: p.name.clone(), 
                company: p.company.clone(),
                variants: serde_json::from_value::<VariantCategoryList>(p.variants.clone()).unwrap(), 
                sku: p.sku.clone(), 
                loyalty_discount: DiscountValue::from_str(&p.loyalty_discount).unwrap(), 
                images: serde_json::from_value::<Vec<Url>>(p.images.clone()).unwrap(), 
                tags: serde_json::from_value::<TagList>(p.tags.clone()).unwrap(), 
                description: p.description.clone(), 
                specifications: serde_json::from_value::<Vec<(String, String)>>(p.specifications.clone()).unwrap() 
            }
        ).collect();

        Ok(mapped)
    }

    pub async fn fetch_by_name_exact(name: &str, db: &DbConn) -> Result<Vec<Product>, DbErr> {
        let res = products::Entity::find()
            .having(products::Column::Name.eq(name))
            .all(db).await?;
            
        let mapped = res.iter().map(|p| 
            Product { 
                name: p.name.clone(), 
                company: p.company.clone(),
                variants: serde_json::from_value::<VariantCategoryList>(p.variants.clone()).unwrap(), 
                sku: p.sku.clone(), 
                loyalty_discount: DiscountValue::from_str(&p.loyalty_discount).unwrap(), 
                images: serde_json::from_value::<Vec<Url>>(p.images.clone()).unwrap(), 
                tags: serde_json::from_value::<TagList>(p.tags.clone()).unwrap(), 
                description: p.description.clone(), 
                specifications: serde_json::from_value::<Vec<(String, String)>>(p.specifications.clone()).unwrap() 
            }
        ).collect();

        Ok(mapped)
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProductExchange {
    pub method_type: TransactionType,
    pub product_code: ProductCode,
    pub variant: VariantIdTag,
    pub quantity: i32,
}

impl Display for ProductExchange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let method = match self.method_type {
            TransactionType::In => "IN",
            TransactionType::Out => "OUT",
        };

        write!(f, "{}: {}-{} x{}", method, self.product_code, self.variant.concat(), self.quantity)
    }
}

pub type ProductCode = String;
pub type ProductPurchaseList = Vec<ProductPurchase>;