use chrono::{DateTime, Days, Utc};
#[cfg(feature = "process")]
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbConn, DbErr, EntityTrait, InsertResult, QueryFilter,
    QuerySelect, Set,
};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use rocket_okapi::JsonSchema;

#[cfg(feature = "process")]
use crate::entities::prelude::Promotion as Promotions;
#[cfg(feature = "process")]
use crate::entities::promotion;
use crate::methods::{DiscountValue, HistoryList, Id, StockList, Url};
#[cfg(feature = "process")]
use crate::products;
use crate::{ProductIdentification, Session};
use serde_json::json;
use uuid::Uuid;
use validator::Validate;

#[cfg(feature = "types")]
pub type VariantIdTag = Vec<VariantId>;

#[cfg(feature = "types")]
type VariantId = String;

#[cfg(feature = "types")]
pub type VariantCategoryList = Vec<VariantCategory>;

#[cfg(feature = "types")]
#[derive(Deserialize, Serialize, Clone, JsonSchema, Validate)]
pub struct VariantCategory {
    pub category: String,
    pub variants: Vec<Variant>,
}

#[cfg(feature = "types")]
#[derive(Deserialize, Serialize, Clone, JsonSchema, Validate)]
/// #### Information for a Variant.
/// This includes its name, identification, stock information and quantities, prices, etc.
pub struct VariantInformation {
    pub id: String,

    /// The variant name
    pub name: String,

    /// The variants stock locations and quantities
    pub stock: StockList,

    /// The variants stock information, such as group, volume, sales stream, etc.
    pub stock_information: StockInformation,

    /// Images specific to the variant, should take priority over product images.
    pub images: Vec<Url>,

    /// Price for the good to be sold at
    pub retail_price: f32,

    /// Imported/Cost price of the good to compare with
    pub marginal_price: f32,

    /// Minimum quantity purchasable
    pub buy_min: f64,

    /// Maximum quantity purchasable
    pub buy_max: f64,

    /// The discount given if in a loyalty program
    pub loyalty_discount: DiscountValue,

    /// The group codes for all sub-variants; i.e. is White, Short Sleeve and Small.
    pub variant_code: VariantIdTag,

    /// <deprecated> Variant-associated order history
    pub order_history: HistoryList,

    /// Barcode for product / primary identification method
    pub barcode: String,

    /// Further identification methods, such as isbn, sku, ...
    pub identification: ProductIdentification,

    /// If `stock_tracking` is false, the product will never be considered 'out of stock'.
    pub stock_tracking: bool,
}

impl Display for VariantInformation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "\t{} ({:?}) ${}[R-:-M]${}",
            self.name, self.variant_code, self.retail_price, self.marginal_price
        )
    }
}

#[cfg(feature = "types")]
#[derive(Deserialize, Serialize, Clone, JsonSchema, Validate)]
pub struct Promotion {
    pub id: Id,
    pub name: String,
    pub buy: PromotionBuy,
    pub get: PromotionGet,
    pub valid_till: DateTime<Utc>,
    pub timestamp: DateTime<Utc>,
}

#[cfg(feature = "types")]
#[derive(Deserialize, Serialize, Clone, JsonSchema, Validate)]
pub struct PromotionInput {
    name: String,
    buy: PromotionBuy,
    get: PromotionGet,
    valid_till: DateTime<Utc>,
    timestamp: DateTime<Utc>,
}

#[cfg(feature = "types")]
#[derive(Deserialize, Serialize, Clone, JsonSchema)]
pub enum PromotionBuy {
    // This(quantity), Specific((id, quantity)), Any(quantity)
    Specific((String, f32)),
    Any(f32),
    Category((String, f32)),
}

#[cfg(feature = "types")]
#[derive(Deserialize, Serialize, Clone, JsonSchema)]
pub enum PromotionGet {
    /// `SoloThis(discount)` <br />
    /// *Represents the individual product.* <br /> <br />
    /// Is used in cases where the product is the recipient of the promotion in inclusive quantity, i.e. 50% off t-shirts (applies to self)
    SoloThis(DiscountValue),
    /// `This((quantity, discount))` <br />
    /// *Represents the continual product.* <br /> <br />
    /// Applies when the following product is the recipient of the promotion, i.e. Buy 1 get 1 half price (product receives 50% discount, but is not directly matching the GET criterion (quantity >= 2...))
    This((f32, DiscountValue)),
    /// `Specific((sku, (quantity, discount)))` <br />
    /// *Represents a specific product* <br /> <br />
    ///  Is used to reference a specific product by its SKU, i.e. Buy any 1 product, get a lib balm $5 off.
    Specific((String, (f32, DiscountValue))),
    /// `Any((quantity, discount))` <br />
    /// *Represents all products* <br /> <br />
    /// A general match-any clause to refer to any product, i.e. Buy 1 get any other product $5 off.
    Any((f32, DiscountValue)),
    /// `AnyOther((quantity, discount))` <br />
    /// *Represents all products other than the original* <br /> <br />
    /// A general match-any clause to refer to any **other** product, i.e. Buy 1 get any other product half price
    AnyOther((f32, DiscountValue)),
    /// `Category(category, (quantity, discount))` <br />
    /// *Represents all products within a category* <br /> <br />
    /// Matches any product within a category, the category is referenced in a products `TagList`. I.e. Buy any 1 product, get any t-shirt 20% off.
    Category((String, (f32, DiscountValue))),
}

#[cfg(feature = "methods")]
impl Promotion {
    pub async fn insert(
        prm: PromotionInput,
        session: Session,
        db: &DbConn,
    ) -> Result<InsertResult<promotion::ActiveModel>, DbErr> {
        let id = Uuid::new_v4().to_string();

        let insert_crud = promotion::ActiveModel {
            id: Set(id.to_string()),
            name: Set(prm.name.to_string()),
            buy: Set(json!(prm.buy)),
            get: Set(json!(prm.get)),
            valid_till: Set(prm.valid_till.naive_utc()),
            timestamp: Set(prm.timestamp.naive_utc()),
            tenant_id: Set(session.tenant_id),
        };

        match Promotions::insert(insert_crud).exec(db).await {
            Ok(res) => Ok(res),
            Err(err) => Err(err),
        }
    }

    pub async fn fetch_by_id(id: &str, session: Session, db: &DbConn) -> Result<Promotion, DbErr> {
        let pdt = Promotions::find_by_id(id.to_string())
            .filter(products::Column::TenantId.eq(session.tenant_id))
            .one(db)
            .await?;
        let p = pdt.unwrap();

        Ok(Promotion {
            id: p.id,
            name: p.name,
            buy: serde_json::from_value::<PromotionBuy>(p.buy).unwrap(),
            get: serde_json::from_value::<PromotionGet>(p.get).unwrap(),
            valid_till: DateTime::from_utc(p.valid_till, Utc),
            timestamp: DateTime::from_utc(p.timestamp, Utc),
        })
    }

    pub async fn fetch_by_query(
        query: &str,
        session: Session,
        db: &DbConn,
    ) -> Result<Vec<Promotion>, DbErr> {
        let res = Promotions::find()
            .filter(products::Column::TenantId.eq(session.tenant_id))
            // Is the bought product
            .having(promotion::Column::Buy.contains(query))
            // Is the promoted product
            .having(promotion::Column::Get.contains(query))
            // Meets the Any criterion
            .having(promotion::Column::Buy.contains("Any"))
            // Meets the Any criterion
            .having(promotion::Column::Get.contains("Any"))
            .all(db)
            .await?;

        let mapped = res
            .iter()
            .map(|p| Promotion {
                id: p.id.clone(),
                name: p.name.clone(),
                buy: serde_json::from_value::<PromotionBuy>(p.buy.clone()).unwrap(),
                get: serde_json::from_value::<PromotionGet>(p.get.clone()).unwrap(),
                valid_till: DateTime::from_utc(p.valid_till, Utc),
                timestamp: DateTime::from_utc(p.timestamp, Utc),
            })
            .collect();

        Ok(mapped)
    }

    pub async fn update(
        prm: PromotionInput,
        session: Session,
        id: &str,
        db: &DbConn,
    ) -> Result<Promotion, DbErr> {
        promotion::ActiveModel {
            id: Set(id.to_string()),
            name: Set(prm.name.to_string()),
            buy: Set(json!(prm.buy)),
            get: Set(json!(prm.get)),
            valid_till: Set(prm.valid_till.naive_utc()),
            timestamp: Set(prm.timestamp.naive_utc()),
            ..Default::default()
        }
        .update(db)
        .await?;

        Self::fetch_by_id(id, session, db).await
    }

    pub async fn fetch_all(session: Session, db: &DbConn) -> Result<Vec<Promotion>, DbErr> {
        let stores = Promotions::find()
            .filter(promotion::Column::TenantId.eq(session.tenant_id))
            .all(db)
            .await?;

        let mapped = stores
            .iter()
            .map(|e| Promotion {
                id: e.id.clone(),
                name: e.name.clone(),
                buy: serde_json::from_value::<PromotionBuy>(e.buy.clone()).unwrap(),
                get: serde_json::from_value::<PromotionGet>(e.get.clone()).unwrap(),
                timestamp: DateTime::from_utc(e.timestamp, Utc),
                valid_till: DateTime::from_utc(e.valid_till, Utc),
            })
            .collect();

        Ok(mapped)
    }

    pub async fn insert_many(
        stores: Vec<PromotionInput>,
        session: Session,
        db: &DbConn,
    ) -> Result<InsertResult<promotion::ActiveModel>, DbErr> {
        let entities = stores.into_iter().map(|prm| {
            let id = Uuid::new_v4().to_string();

            promotion::ActiveModel {
                id: Set(id),
                name: Set(prm.name.to_string()),
                buy: Set(json!(prm.buy)),
                get: Set(json!(prm.get)),
                valid_till: Set(prm.valid_till.naive_utc()),
                timestamp: Set(prm.timestamp.naive_utc()),
                tenant_id: Set(session.clone().tenant_id),
            }
        });

        match Promotions::insert_many(entities).exec(db).await {
            Ok(res) => Ok(res),
            Err(err) => Err(err),
        }
    }

    pub async fn generate(session: Session, db: &DbConn) -> Result<Vec<Promotion>, DbErr> {
        let promotions = example_promotions();

        match Promotion::insert_many(promotions, session.clone(), db).await {
            Ok(_) => match Promotion::fetch_all(session, db).await {
                Ok(res) => Ok(res),
                Err(e) => Err(e),
            },
            Err(e) => Err(e),
        }
    }
}

/// Represents all sub-variant types; i.e. All 'White' variants, whether small, long-sleeve, ... it represents the sub-group of all which are 'White'.
#[cfg(feature = "types")]
#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema, Validate)]
pub struct Variant {
    pub name: String,
    pub images: Vec<Url>,
    pub marginal_price: f32,
    pub variant_code: String,
    pub order_history: HistoryList,
}

#[cfg(feature = "types")]
#[derive(Deserialize, Serialize, Clone, JsonSchema, Validate)]
pub struct StockInformation {
    pub stock_group: String,
    pub sales_group: String,
    pub value_stream: String,

    /// At this stock level (or below), it should be indicated that the stock level is 'low'.
    pub min_stock_before_alert: f64,

    /// Will treat product when at this quantity as 'out of stock'
    pub min_stock_level: f64,

    /// The publisher/author/creator/manufacturer of the good or service
    pub brand: String,

    /// Individual shipment packing unit - used to show identification of multi-part shipments
    pub colli: String,

    pub size_x: f64,
    pub size_y: f64,
    pub size_z: f64,

    pub size_x_unit: String,
    pub size_y_unit: String,
    pub size_z_unit: String,
    pub size_override_unit: String,

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
    pub non_diminishing: bool,

    /// A non-shippable good is one which cannot be dispatched between stores or sent to a customers home, this might be a fragile product, service, oversized good or edge case.
    pub shippable: bool,
}

impl Display for Variant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "\t{} ({}) (MP: ${})",
            self.name, self.variant_code, self.marginal_price
        )
    }
}

fn example_promotions() -> Vec<PromotionInput> {
    vec![
        PromotionInput {
            name: "Buy 1 Get 1 10% off".to_string(),
            buy: PromotionBuy::Any(1.0),
            get: PromotionGet::Any((1.0, DiscountValue::Percentage(10))),
            valid_till: Utc::now().checked_add_days(Days::new(7)).unwrap(),
            timestamp: Utc::now(),
        },
        PromotionInput {
            name: "50% off T-shirts".to_string(),
            buy: PromotionBuy::Category(("Tee".into(), 1.0)),
            get: PromotionGet::SoloThis(DiscountValue::Percentage(50)),
            valid_till: Utc::now().checked_add_days(Days::new(7)).unwrap(),
            timestamp: Utc::now(),
        },
        PromotionInput {
            name: "Buy a Kayak, get a Life Jacket 50% off".to_string(),
            buy: PromotionBuy::Specific(("654321".into(), 1.0)),
            get: PromotionGet::Specific(("162534".into(), (1.0, DiscountValue::Percentage(50)))),
            valid_till: Utc::now().checked_add_days(Days::new(7)).unwrap(),
            timestamp: Utc::now(),
        },
    ]
}
