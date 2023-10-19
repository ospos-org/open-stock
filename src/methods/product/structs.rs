use std::fmt::Display;

use crate::{History, Session, TransactionType};
use chrono::{DateTime, Utc};
#[cfg(feature = "process")]
use sea_orm::{
    sea_query::{Expr, Func},
    ActiveModelTrait, ColumnTrait, Condition, DbConn, DbErr, EntityTrait, InsertResult,
    QueryFilter, QuerySelect, Statement,
};
use serde::{
    de::{MapAccess, Visitor},
    Deserialize, Deserializer, Serialize,
};
use uuid::Uuid;

use super::{
    Promotion, PromotionBuy, PromotionGet,
    VariantCategoryList, VariantIdTag, VariantInformation,
};
#[cfg(feature = "process")]
use crate::entities::prelude::Products;
#[cfg(feature = "process")]
use crate::entities::prelude::Promotion as Promotions;
#[cfg(feature = "process")]
use crate::entities::products;

use crate::{
    methods::{DiscountValue, TagList, Url},
    Note,
};
#[cfg(feature = "process")]
use futures::future::join_all;
use schemars::JsonSchema;
use crate::product::example::example_products;

#[cfg(feature = "types")]
#[derive(Deserialize, Serialize, Clone, JsonSchema)]
pub enum ProductVisibility {
    AlwaysShown,
    AlwaysHidden,
    ShowWhenInStock,
}

#[cfg(feature = "types")]
#[derive(Deserialize, Serialize, Clone, Default, JsonSchema)]
pub struct ProductIdentification {
    pub sku: String,
    pub ean: String,
    pub hs_code: String,
    pub article_code: String,
    pub isbn: String,
}

#[cfg(feature = "types")]
#[derive(Deserialize, Serialize, Clone, JsonSchema)]
/// A product, containing a list of `Vec<Variant>`, an identifiable `sku` along with identifying information such as `tags`, `description` and `specifications`.
/// > Stock-relevant information about a product is kept under each variant, thus allowing for modularity of different variants and a fine-grained control over your inventory.
pub struct Product {
    pub name: String,
    pub name_long: String,

    pub company: String,

    pub variant_groups: VariantCategoryList,
    /// Lists all the **possible** combinations of a product in terms of its variants.
    pub variants: Vec<VariantInformation>,

    pub sku: String,
    pub identification: ProductIdentification,

    pub images: Vec<Url>,
    pub tags: TagList,
    pub description: String,
    pub description_long: String,

    pub specifications: Vec<(String, String)>,
    pub visible: ProductVisibility,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>
}

#[cfg(feature = "types")]
#[derive(Deserialize, Serialize, Clone, JsonSchema)]
pub struct ProductWPromotion {
    pub product: Product,
    pub promotions: Vec<Promotion>,
}

impl Display for Product {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let variant_categories: String = self
            .variant_groups
            .iter()
            .map(|p| {
                let variants: String = p.variants.iter().map(|p| format!("{}\n", p)).collect();

                format!("{}(s):\n{}", p.category, variants)
            })
            .collect();

        write!(
            f,
            "{}: {} ({})\n{}",
            self.sku, self.name, self.company, variant_categories
        )
    }
}

#[cfg(feature = "methods")]
impl Product {
    pub async fn insert(
        pdt: Product,
        session: Session,
        db: &DbConn,
    ) -> Result<InsertResult<products::ActiveModel>, DbErr> {
        let insert_crud = pdt.into_active(session);

        match Products::insert(insert_crud).exec(db).await {
            Ok(res) => Ok(res),
            Err(err) => Err(err),
        }
    }

    pub async fn fetch_by_id(id: &str, session: Session, db: &DbConn) -> Result<Product, DbErr> {
        let pdt = Products::find_by_id(id.to_string())
            .filter(products::Column::TenantId.eq(session.tenant_id))
            .one(db)
            .await?;

        let p = pdt.unwrap();

        Ok(p.into())
    }

    pub async fn fetch_by_id_with_promotion(
        id: &str,
        session: Session,
        db: &DbConn,
    ) -> Result<ProductWPromotion, DbErr> {
        let pdt = Products::find_by_id(id.to_string())
            .filter(products::Column::TenantId.eq(session.tenant_id))
            .one(db)
            .await?;
        let p = pdt.unwrap();

        let product: Product = p.into();

        let promos = Promotions::find()
            .from_raw_sql(Statement::from_sql_and_values(
                sea_orm::DatabaseBackend::MySql,
                &format!(
                    "SELECT * FROM Promotion WHERE `buy` LIKE '%Any%'
                            OR `get` LIKE '%Any%'
                            OR `buy` LIKE '%{}%'
                            OR `get` LIKE '%{}%'
                            OR `buy` LIKE '%{}%'
                            OR `get` LIKE '%{}%'
                            AND `valid_till` >= NOW()
                            LIMIT 25",
                    product.sku,
                    product.sku,
                    product.tags.join("%' OR `buy` LIKE '%"),
                    product.tags.join("%' OR `get` LIKE '%")
                ),
                vec![],
            ))
            .all(db)
            .await
            .unwrap();

        let mapped: Vec<Promotion> = promos
            .iter()
            .map(|p| Promotion {
                name: p.name.clone(),
                buy: serde_json::from_value::<PromotionBuy>(p.buy.clone()).unwrap(),
                get: serde_json::from_value::<PromotionGet>(p.get.clone()).unwrap(),
                id: p.id.clone(),
                valid_till: DateTime::from_utc(p.valid_till, Utc),
                timestamp: DateTime::from_utc(p.timestamp, Utc),
            })
            .collect();

        Ok(ProductWPromotion {
            product,
            promotions: mapped,
        })
    }

    pub async fn search(query: &str, session: Session, db: &DbConn) -> Result<Vec<Product>, DbErr> {
        let res = products::Entity::find()
            .filter(products::Column::TenantId.eq(session.tenant_id))
            .filter(
                Condition::any()
                    .add(
                        Expr::expr(Func::lower(Expr::col(products::Column::Name)))
                            .like(format!("%{}%", query)),
                    )
                    .add(products::Column::Sku.contains(query))
                    .add(products::Column::Variants.contains(query)),
            )
            .limit(25)
            .all(db)
            .await?;

        let mapped = res
            .iter()
            .map(|p| p.clone().into())
            .collect();

        Ok(mapped)
    }

    pub async fn search_with_promotion(
        query: &str,
        session: Session,
        db: &DbConn,
    ) -> Result<Vec<ProductWPromotion>, DbErr> {
        let res = products::Entity::find()
            .from_raw_sql(
                Statement::from_sql_and_values(
                    sea_orm::DatabaseBackend::MySql,
                    &format!("SELECT * FROM `Products` WHERE (MATCH(`name`, `company`) AGAINST('{}' IN NATURAL LANGUAGE MODE) OR `Products`.`sku` LIKE '%{}%' OR `Products`.`variants` LIKE '%{}%') AND Products.tenant_id = '{}' LIMIT 25",
                    query, query, query, session.tenant_id),
                    vec![]
                )
            )
            .all(db)
            .await?;

        let mapped: Vec<ProductWPromotion> = res
            .iter()
            .map(|p| ProductWPromotion {
                product: p.clone().into(),
                promotions: vec![],
            })
            .collect();

        let with_promotions = join_all(mapped.iter().map(|p| async move {
            let b = db.clone();

            let tags_iterable = p.product.tags.clone().into_iter();
            let search_tags = tags_iterable
                .filter(|tag| !tag.is_empty())
                .collect::<Vec<String>>();

            let promos = Promotions::find()
                .from_raw_sql(Statement::from_sql_and_values(
                    sea_orm::DatabaseBackend::MySql,
                    &format!(
                        "SELECT * FROM Promotion WHERE `buy` LIKE '%Any%'
                            OR `get` LIKE '%Any%'
                            OR `buy` LIKE '%{}%'
                            OR `get` LIKE '%{}%'
                            OR `buy` LIKE '%{}%'
                            OR `get` LIKE '%{}%'
                            AND `valid_till` >= NOW()
                            LIMIT 25",
                        p.product.sku,
                        p.product.sku,
                        if search_tags.is_empty() {
                            Uuid::new_v4().to_string()
                        } else {
                            search_tags.join("%' OR `buy` LIKE '%")
                        },
                        if search_tags.is_empty() {
                            Uuid::new_v4().to_string()
                        } else {
                            search_tags.join("%' OR `get` LIKE '%")
                        }
                    ),
                    vec![],
                ))
                .all(&b)
                .await
                .unwrap();

            let mapped: Vec<Promotion> = promos
                .iter()
                .map(|p| Promotion {
                    name: p.name.clone(),
                    buy: serde_json::from_value::<PromotionBuy>(p.buy.clone()).unwrap(),
                    get: serde_json::from_value::<PromotionGet>(p.get.clone()).unwrap(),
                    id: p.id.clone(),
                    valid_till: DateTime::from_utc(p.valid_till, Utc),
                    timestamp: DateTime::from_utc(p.timestamp, Utc),
                })
                .collect();

            ProductWPromotion {
                product: p.product.clone(),
                promotions: mapped,
            }
        }))
        .await;

        Ok(with_promotions)
    }

    pub async fn fetch_by_name(
        name: &str,
        session: Session,
        db: &DbConn,
    ) -> Result<Vec<Product>, DbErr> {
        let res = products::Entity::find()
            .having(products::Column::Name.contains(name))
            .filter(products::Column::TenantId.eq(session.tenant_id))
            .limit(25)
            .all(db)
            .await?;

        let mapped = res
            .iter()
            .map(|p| p.clone().into())
            .collect();

        Ok(mapped)
    }

    pub async fn fetch_by_name_exact(
        name: &str,
        session: Session,
        db: &DbConn,
    ) -> Result<Vec<Product>, DbErr> {
        let res = products::Entity::find()
            .having(products::Column::Name.eq(name))
            .filter(products::Column::TenantId.eq(session.tenant_id))
            .limit(25)
            .all(db)
            .await?;

        let mapped = res
            .iter()
            .map(|p| p.clone().into())
            .collect();

        Ok(mapped)
    }

    pub async fn update(
        pdt: Product,
        session: Session,
        id: &str,
        db: &DbConn,
    ) -> Result<Product, DbErr> {
        pdt.into_active(session.clone())
            .update(db)
            .await?;

        Self::fetch_by_id(id, session, db).await
    }

    pub async fn fetch_all(session: Session, db: &DbConn) -> Result<Vec<Product>, DbErr> {
        let products = Products::find()
            .filter(products::Column::TenantId.eq(session.tenant_id))
            .all(db)
            .await?;

        let mapped = products
            .iter()
            .map(|p| p.clone().into())
            .collect();

        Ok(mapped)
    }

    pub async fn insert_many(
        products: Vec<Product>,
        session: Session,
        db: &DbConn,
    ) -> Result<InsertResult<products::ActiveModel>, DbErr> {
        let entities = products.into_iter().map(|pdt|
            pdt.into_active(session.clone())
        );

        match Products::insert_many(entities).exec(db).await {
            Ok(res) => Ok(res),
            Err(err) => Err(err),
        }
    }

    pub async fn generate(session: Session, db: &DbConn) -> Result<Vec<Product>, DbErr> {
        let products = example_products();

        match Product::insert_many(products, session.clone(), db).await {
            Ok(_) => match Product::fetch_all(session, db).await {
                Ok(res) => Ok(res),
                Err(e) => Err(e),
            },
            Err(e) => Err(e),
        }
    }
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct ProductPurchase {
    // Is the barcode of the product.
    pub id: String,

    pub product_code: ProductCode,
    pub product_sku: String,
    pub discount: DiscountValue,

    pub product_name: String,
    pub product_variant_name: String,

    // Cost before discount, discount will be applied on the product cost.
    pub product_cost: f32,
    pub quantity: f32,
    pub tags: TagList,

    pub transaction_type: TransactionType,
    pub instances: Vec<ProductInstance>,
}

impl<'de> Deserialize<'de> for ProductPurchase {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ProductVisitor;

        impl<'de> Visitor<'de> for ProductVisitor {
            type Value = ProductPurchase;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct Product")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut id = None;

                let mut product_code = None;
                let mut product_sku = None;
                let mut discount = None;

                let mut product_name = None;
                let mut product_variant_name = None;

                let mut tags = None;

                let mut product_cost = None;
                let mut transaction_type = None;
                let mut quantity = None;
                let mut instances: Option<Vec<ProductInstance>> = None;

                // pub transaction_type: TransactionType,
                while let Some(key_s) = map.next_key::<String>()? {
                    let key = key_s.as_str();

                    match key {
                        "id" => {
                            if id.is_some() {
                                return Err(serde::de::Error::duplicate_field("id"));
                            }
                            id = Some(map.next_value()?);
                        }
                        "product_code" => {
                            if product_code.is_some() {
                                return Err(serde::de::Error::duplicate_field("product_code"));
                            }
                            product_code = Some(map.next_value()?);
                        }
                        "product_sku" => {
                            if product_sku.is_some() {
                                return Err(serde::de::Error::duplicate_field("product_sku"));
                            }
                            product_sku = Some(map.next_value()?);
                        }
                        "discount" => {
                            if discount.is_some() {
                                return Err(serde::de::Error::duplicate_field("discount"));
                            }
                            discount = Some(map.next_value::<DiscountValue>()?);
                        }
                        "tags" => {
                            if tags.is_some() {
                                return Err(serde::de::Error::duplicate_field("tags"));
                            }
                            tags = Some(map.next_value::<TagList>()?);
                        }
                        "product_name" => {
                            if product_name.is_some() {
                                return Err(serde::de::Error::duplicate_field("product_name"));
                            }
                            product_name = Some(map.next_value()?);
                        }
                        "product_variant_name" => {
                            if product_variant_name.is_some() {
                                return Err(serde::de::Error::duplicate_field(
                                    "product_variant_name",
                                ));
                            }
                            product_variant_name = Some(map.next_value()?);
                        }
                        "product_cost" => {
                            if product_cost.is_some() {
                                return Err(serde::de::Error::duplicate_field("product_cost"));
                            }
                            product_cost = Some(map.next_value()?);
                        }
                        "transaction_type" => {
                            if transaction_type.is_some() {
                                return Err(serde::de::Error::duplicate_field("transaction_type"));
                            }
                            transaction_type = Some(map.next_value::<TransactionType>()?);
                        }
                        "quantity" => {
                            if quantity.is_some() {
                                return Err(serde::de::Error::duplicate_field("quantity"));
                            }
                            quantity = Some(map.next_value()?);
                        }
                        "instances" => {
                            if instances.is_some() {
                                return Err(serde::de::Error::duplicate_field("quantity"));
                            }
                            instances = Some(map.next_value()?);
                        }
                        _ => {
                            return Err(serde::de::Error::unknown_field(
                                key,
                                &["id", "quantity", "instances"],
                            ))
                        }
                    }
                }

                let id = id.ok_or_else(|| serde::de::Error::missing_field("id"))?;

                let product_code =
                    product_code.ok_or_else(|| serde::de::Error::missing_field("product_code"))?;
                let product_sku =
                    product_sku.ok_or_else(|| serde::de::Error::missing_field("product_sku"))?;
                let discount =
                    discount.ok_or_else(|| serde::de::Error::missing_field("discount"))?;

                let product_name =
                    product_name.ok_or_else(|| serde::de::Error::missing_field("product_name"))?;
                let product_variant_name = product_variant_name
                    .ok_or_else(|| serde::de::Error::missing_field("product_variant_name"))?;
                let product_cost =
                    product_cost.ok_or_else(|| serde::de::Error::missing_field("product_cost"))?;

                let transaction_type = transaction_type
                    .ok_or_else(|| serde::de::Error::missing_field("transaction_type"))?;
                let quantity =
                    quantity.ok_or_else(|| serde::de::Error::missing_field("quantity"))?;
                let tags = tags.ok_or_else(|| serde::de::Error::missing_field("tags"))?;
                let mut instances = instances.unwrap_or_default();

                while instances.len() < quantity as usize {
                    instances.push(ProductInstance {
                        id: format!("{}-{}-{}", id, instances.len() + 1, Uuid::new_v4()),
                        fulfillment_status: default_fulfillment(),
                    });
                }
                Ok(ProductPurchase {
                    id,
                    product_code,
                    product_sku,
                    discount,
                    product_name,
                    product_variant_name,
                    transaction_type,
                    product_cost,
                    tags,
                    quantity,
                    instances,
                })
            }
        }

        deserializer.deserialize_map(ProductVisitor)
    }
}

#[cfg(feature = "types")]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ProductInstance {
    pub id: String,
    #[serde(default = "default_fulfillment")]
    pub fulfillment_status: FulfillmentStatus,
}

fn default_fulfillment() -> FulfillmentStatus {
    FulfillmentStatus {
        pick_status: PickStatus::Pending,
        pick_history: vec![],
        last_updated: Utc::now(),
        notes: vec![],
    }
}

#[cfg(feature = "types")]
#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct FulfillmentStatus {
    pub pick_status: PickStatus,
    pub pick_history: Vec<History<PickStatus>>,
    pub last_updated: DateTime<Utc>,
    pub notes: Vec<Note>,
}

#[cfg(feature = "types")]
#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub enum PickStatus {
    Pending,
    Picked,
    Failed,
    Uncertain,
    Processing,
    Other(String),
}

#[cfg(feature = "types")]
#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct ProductExchange {
    pub method_type: TransactionType,
    pub product_code: ProductCode,
    pub variant: VariantIdTag,
    pub quantity: f32,
}

impl Display for ProductExchange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let method = match self.method_type {
            TransactionType::In => "IN",
            TransactionType::Out => "OUT",
            TransactionType::PendingIn => "PENDING-IN",
            TransactionType::PendingOut => "PENDING-OUT",
            TransactionType::Saved => "[SAVED]",
            TransactionType::Quote => "[QUOTE]",
        };

        write!(
            f,
            "{}: {}-{} x{}",
            method,
            self.product_code,
            self.variant.concat(),
            self.quantity
        )
    }
}

#[cfg(feature = "types")]
pub type ProductCode = String;

#[cfg(feature = "types")]
pub type ProductPurchaseList = Vec<ProductPurchase>;
