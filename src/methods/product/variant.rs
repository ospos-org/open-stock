use std::fmt::Display;

use chrono::{DateTime, Utc};
use sea_orm::{DbConn, DbErr, EntityTrait, QuerySelect, ColumnTrait, Set, ActiveModelTrait, InsertResult};
use serde::{Deserialize, Serialize};

use serde_json::json;
use uuid::Uuid;
use crate::entities::promotion;
use crate::methods::{StockList, HistoryList, Url, DiscountValue, Id, Session};
use crate::entities::prelude::Promotion as Promotions;

pub type VariantIdTag = Vec<VariantId>;
type VariantId = String;

pub type VariantCategoryList = Vec<VariantCategory>;

#[derive(Deserialize, Serialize, Clone)]
pub struct VariantCategory {
    pub category: String,
    pub variants: Vec<Variant>
}

#[derive(Deserialize, Serialize, Clone)]
pub struct VariantInformation {
    pub name: String,
    pub stock: StockList,
    pub images: Vec<Url>,
    pub retail_price: f32,
    pub marginal_price: f32,
    pub id: String,
    pub loyalty_discount: DiscountValue,
    /// The group codes for all sub-variants; i.e. is White, Short Sleeve and Small.
    pub variant_code: VariantIdTag,
    pub order_history: HistoryList,
    pub stock_information: StockInformation,
    pub barcode: String
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Promotion {
    id: Id,
    name: String,
    buy: PromotionBuy,
    get: PromotionGet,
    valid_till: DateTime<Utc>,
    timestamp: DateTime<Utc>
}

#[derive(Deserialize, Serialize, Clone)]
pub struct PromotionInput {
    name: String,
    buy: PromotionBuy,
    get: PromotionGet,
    valid_till: DateTime<Utc>,
    timestamp: DateTime<Utc>
}

#[derive(Deserialize, Serialize, Clone)]
enum PromotionBuy {
    // This(quantity), Specific((id, quantity)), Any(quantity)
    This(f32), Specific((String, f32)), Any(f32)
}

#[derive(Deserialize, Serialize, Clone)]
enum PromotionGet {
    // This((quantity, discount)), Specific((id, (quantity, discount))), Any((quantity, discount))
    This((f32, DiscountValue)), Specific((String, (f32, DiscountValue))), Any((f32, DiscountValue))
}

impl Promotion {
    pub async fn insert(prm: PromotionInput, db: &DbConn) -> Result<InsertResult<promotion::ActiveModel>, DbErr> {
        let id = Uuid::new_v4().to_string();

        let insert_crud = promotion::ActiveModel {
            id: Set(id.to_string()),
            name: Set(prm.name.to_string()),
            buy: Set(json!(prm.buy)),
            get: Set(json!(prm.get)),
            valid_till: Set(prm.valid_till.naive_utc()),
            timestamp: Set(prm.timestamp.naive_utc()),
        };

        match Promotions::insert(insert_crud).exec(db).await {
            Ok(res) => Ok(res),
            Err(err) => Err(err)
        }
    }

    pub async fn fetch_by_id(id: &str, db: &DbConn) -> Result<Promotion, DbErr> {
        let pdt = Promotions::find_by_id(id.to_string()).one(db).await?;
        let p = pdt.unwrap();

        Ok(Promotion { 
            id: p.id, 
            name: p.name, 
            buy: serde_json::from_value::<PromotionBuy>(p.buy).unwrap(), 
            get: serde_json::from_value::<PromotionGet>(p.get).unwrap(), 
            valid_till: DateTime::from_utc(p.valid_till, Utc), 
            timestamp: DateTime::from_utc(p.timestamp, Utc) 
        })
    }

    pub async fn fetch_by_query(query: &str, db: &DbConn) -> Result<Vec<Promotion>, DbErr> {
        let res = Promotions::find()
            .having(promotion::Column::Buy.contains(query))
            .having(promotion::Column::Get.contains(query))
            .all(db).await?;

        let mapped = res.iter().map(|p| {
            Promotion { 
                id: p.id.clone(), 
                name: p.name.clone(), 
                buy: serde_json::from_value::<PromotionBuy>(p.buy.clone()).unwrap(), 
                get: serde_json::from_value::<PromotionGet>(p.get.clone()).unwrap(), 
                valid_till: DateTime::from_utc(p.valid_till, Utc), 
                timestamp: DateTime::from_utc(p.timestamp, Utc) 
            }
        }).collect();

        Ok(mapped)
    }

    pub async fn update(prm: PromotionInput, id: &str, db: &DbConn) -> Result<Promotion, DbErr> {
        promotion::ActiveModel {
            id: Set(id.to_string()),
            name: Set(prm.name.to_string()),
            buy: Set(json!(prm.buy)),
            get: Set(json!(prm.get)),
            valid_till: Set(prm.valid_till.naive_utc()),
            timestamp: Set(prm.timestamp.naive_utc()),
        }.update(db).await?;

        Self::fetch_by_id(id, db).await
    }
}

/// Represents all sub-variant types; i.e. All 'White' variants, whether small, long-sleeve, ... it represents the sub-group of all which are 'White'.
#[derive(Deserialize, Serialize, Clone)]
pub struct Variant {
    pub name: String,
    pub images: Vec<Url>,
    pub marginal_price: i32,
    pub variant_code: String,
    pub order_history: HistoryList,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct StockInformation {
    pub stock_group: String,
    pub sales_group: String,
    pub value_stream: String,

    pub brand: String,
    pub unit: String,

    /// Non-required field which outlines the tax code of the product if necessary.
    pub tax_code: String,

    /// The variant's weight in kilograms.
    pub weight: String,

    /// The volume of the product in meters cubed, kept specific to each variant.
    pub volume: String,

    /// A quantity considered to be the *maximum*. If the quantity dips below such value, it is suggested a restock should take place.
    pub max_volume: String,

    /// If the product's supply cannot be fulfilled at the current time, due to a lack of availability. 
    /// 
    /// By setting `back_order` to `true`, it allows for the purchase of the product on the promise it will be delivered to the customer or collected from the store at a later date. **This must be made clear and known to the customer.**
    pub back_order: bool,
    /// A product which is no longer source-able. Once the product's inventory is consumed it is indicated to never be replenished.
    pub discontinued: bool,
    /// A `non_diminishing` product is often a service rather than a product, i.e. freight. It is **not removed** from inventory upon consumption, rather attached.
    pub non_diminishing: bool 
}

impl Display for Variant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,
            "\t{} ({}) ${}", 
            self.name, self.variant_code, self.marginal_price 
        )
    }
}