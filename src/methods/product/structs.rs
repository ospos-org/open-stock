use std::fmt::Display;

use crate::History;
use chrono::{DateTime, Utc};
use sea_orm::{
    sea_query::{Expr, Func},
    ActiveModelTrait, ColumnTrait, Condition, DbConn, DbErr, EntityTrait, InsertResult,
    QueryFilter, QuerySelect, Set, Statement,
};
use serde::{
    de::{MapAccess, Visitor},
    Deserialize, Deserializer, Serialize,
};
use serde_json::json;
use uuid::Uuid;

use super::{
    Promotion, PromotionBuy, PromotionGet, StockInformation, Variant, VariantCategory,
    VariantCategoryList, VariantIdTag, VariantInformation,
};
use crate::entities::prelude::Products;
use crate::entities::prelude::Promotion as Promotions;
use crate::{
    entities::{products, sea_orm_active_enums::TransactionType},
    methods::{
        Address, ContactInformation, DiscountValue, Email, Location, MobileNumber, Quantity, Stock,
        TagList, Url,
    },
    Note,
};
use futures::future::join_all;

#[derive(Deserialize, Serialize, Clone)]
pub enum ProductVisibility {
    AlwaysShown,
    AlwaysHidden,
    ShowWhenInStock,
}

#[derive(Deserialize, Serialize, Clone, Default)]
pub struct ProductIdentification {
    pub sku: String,
    pub ean: String,
    pub hs_code: String,
    pub article_code: String,
    pub isbn: String,
}

#[derive(Deserialize, Serialize, Clone)]
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
}

#[derive(Deserialize, Serialize, Clone)]
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

impl Product {
    pub async fn insert(
        pdt: Product,
        db: &DbConn,
    ) -> Result<InsertResult<products::ActiveModel>, DbErr> {
        let insert_crud = products::ActiveModel {
            sku: Set(pdt.sku),
            name: Set(pdt.name),
            company: Set(pdt.company),
            variants: Set(json!(pdt.variants)),
            variant_groups: Set(json!(pdt.variant_groups)),
            images: Set(json!(pdt.images)),
            tags: Set(json!(pdt.tags)),
            description: Set(pdt.description),
            specifications: Set(json!(pdt.specifications)),
            identification: Set(json!(pdt.identification)),
            visible: Set(json!(pdt.visible)),
            name_long: Set(pdt.name_long),
            description_long: Set(pdt.description_long),
        };

        match Products::insert(insert_crud).exec(db).await {
            Ok(res) => Ok(res),
            Err(err) => Err(err),
        }
    }

    pub async fn fetch_by_id(id: &str, db: &DbConn) -> Result<Product, DbErr> {
        let pdt = Products::find_by_id(id.to_string()).one(db).await?;

        let p = pdt.unwrap();

        Ok(Product {
            name: p.name,
            company: p.company,
            variant_groups: serde_json::from_value::<VariantCategoryList>(p.variant_groups)
                .unwrap(),
            variants: serde_json::from_value::<Vec<VariantInformation>>(p.variants).unwrap(),
            sku: p.sku,
            images: serde_json::from_value::<Vec<Url>>(p.images).unwrap(),
            tags: serde_json::from_value::<TagList>(p.tags).unwrap(),
            description: p.description,
            specifications: serde_json::from_value::<Vec<(String, String)>>(p.specifications)
                .unwrap(),
            identification: serde_json::from_value::<ProductIdentification>(p.identification)
                .unwrap(),
            visible: serde_json::from_value::<ProductVisibility>(p.visible).unwrap(),
            name_long: p.name_long,
            description_long: p.description_long,
        })
    }

    pub async fn fetch_by_id_with_promotion(
        id: &str,
        db: &DbConn,
    ) -> Result<ProductWPromotion, DbErr> {
        let pdt = Products::find_by_id(id.to_string()).one(db).await?;
        let p = pdt.unwrap();

        let product = Product {
            name: p.name,
            company: p.company,
            variant_groups: serde_json::from_value::<VariantCategoryList>(p.variant_groups)
                .unwrap(),
            variants: serde_json::from_value::<Vec<VariantInformation>>(p.variants).unwrap(),
            sku: p.sku,
            images: serde_json::from_value::<Vec<Url>>(p.images).unwrap(),
            tags: serde_json::from_value::<TagList>(p.tags).unwrap(),
            description: p.description,
            specifications: serde_json::from_value::<Vec<(String, String)>>(p.specifications)
                .unwrap(),
            identification: serde_json::from_value::<ProductIdentification>(p.identification)
                .unwrap(),
            visible: serde_json::from_value::<ProductVisibility>(p.visible).unwrap(),
            name_long: p.name_long,
            description_long: p.description_long,
        };

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

    pub async fn search(query: &str, db: &DbConn) -> Result<Vec<Product>, DbErr> {
        let res = products::Entity::find()
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
            .map(|p| Product {
                name: p.name.clone(),
                company: p.company.clone(),
                variant_groups: serde_json::from_value::<VariantCategoryList>(
                    p.variant_groups.clone(),
                )
                .unwrap(),
                variants: serde_json::from_value::<Vec<VariantInformation>>(p.variants.clone())
                    .unwrap(),
                sku: p.sku.clone(),
                images: serde_json::from_value::<Vec<Url>>(p.images.clone()).unwrap(),
                tags: serde_json::from_value::<TagList>(p.tags.clone()).unwrap(),
                description: p.description.clone(),
                specifications: serde_json::from_value::<Vec<(String, String)>>(
                    p.specifications.clone(),
                )
                .unwrap(),
                identification: serde_json::from_value::<ProductIdentification>(
                    p.identification.clone(),
                )
                .unwrap(),
                visible: serde_json::from_value::<ProductVisibility>(p.visible.clone()).unwrap(),
                name_long: p.name_long.clone(),
                description_long: p.description_long.clone(),
            })
            .collect();

        Ok(mapped)
    }

    pub async fn search_with_promotion(
        query: &str,
        db: &DbConn,
    ) -> Result<Vec<ProductWPromotion>, DbErr> {
        let res = products::Entity::find()
            .from_raw_sql(
                Statement::from_sql_and_values(
                    sea_orm::DatabaseBackend::MySql,
                    &format!("SELECT * FROM `Products` WHERE MATCH(`name`, `company`) AGAINST('{}' IN NATURAL LANGUAGE MODE) OR `Products`.`sku` LIKE '%{}%' OR `Products`.`variants` LIKE '%{}%' LIMIT 25",
                    query, query, query),
                    vec![]
                )
            )
            .all(db)
            .await?;

        let mapped: Vec<ProductWPromotion> = res
            .iter()
            .map(|p| ProductWPromotion {
                product: Product {
                    name: p.name.clone(),
                    company: p.company.clone(),
                    variant_groups: serde_json::from_value::<VariantCategoryList>(
                        p.variant_groups.clone(),
                    )
                    .unwrap(),
                    variants: serde_json::from_value::<Vec<VariantInformation>>(p.variants.clone())
                        .unwrap(),
                    sku: p.sku.clone(),
                    images: serde_json::from_value::<Vec<Url>>(p.images.clone()).unwrap(),
                    tags: serde_json::from_value::<TagList>(p.tags.clone()).unwrap(),
                    description: p.description.clone(),
                    specifications: serde_json::from_value::<Vec<(String, String)>>(
                        p.specifications.clone(),
                    )
                    .unwrap(),
                    identification: serde_json::from_value::<ProductIdentification>(
                        p.identification.clone(),
                    )
                    .unwrap(),
                    visible: serde_json::from_value::<ProductVisibility>(p.visible.clone())
                        .unwrap(),
                    name_long: p.name_long.clone(),
                    description_long: p.description_long.clone(),
                },
                promotions: vec![],
            })
            .collect();

        let with_promotions = join_all(mapped.iter().map(|p| async move {
            let b = db.clone();

            let promos = Promotions::find()
                // .filter(
                //     Condition::any()
                //         .add(promotion::Column::Buy.contains(&p.product.sku))
                //         .add(promotion::Column::Buy.contains(&p.product.tags[0]))
                //         .add(promotion::Column::Get.contains(&p.product.sku))
                //         .add(promotion::Column::Buy.contains("Any"))
                //         .add(promotion::Column::Get.contains("Any"))
                // )
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
                        p.product.tags.join("%' OR `buy` LIKE '%"),
                        p.product.tags.join("%' OR `get` LIKE '%")
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

    pub async fn fetch_by_name(name: &str, db: &DbConn) -> Result<Vec<Product>, DbErr> {
        let res = products::Entity::find()
            .having(products::Column::Name.contains(name))
            .limit(25)
            .all(db)
            .await?;

        let mapped = res
            .iter()
            .map(|p| Product {
                name: p.name.clone(),
                company: p.company.clone(),
                variant_groups: serde_json::from_value::<VariantCategoryList>(
                    p.variant_groups.clone(),
                )
                .unwrap(),
                variants: serde_json::from_value::<Vec<VariantInformation>>(p.variants.clone())
                    .unwrap(),
                sku: p.sku.clone(),
                images: serde_json::from_value::<Vec<Url>>(p.images.clone()).unwrap(),
                tags: serde_json::from_value::<TagList>(p.tags.clone()).unwrap(),
                description: p.description.clone(),
                specifications: serde_json::from_value::<Vec<(String, String)>>(
                    p.specifications.clone(),
                )
                .unwrap(),
                identification: serde_json::from_value::<ProductIdentification>(
                    p.identification.clone(),
                )
                .unwrap(),
                visible: serde_json::from_value::<ProductVisibility>(p.visible.clone()).unwrap(),
                name_long: p.name_long.clone(),
                description_long: p.description_long.clone(),
            })
            .collect();

        Ok(mapped)
    }

    pub async fn fetch_by_name_exact(name: &str, db: &DbConn) -> Result<Vec<Product>, DbErr> {
        let res = products::Entity::find()
            .having(products::Column::Name.eq(name))
            .limit(25)
            .all(db)
            .await?;

        let mapped = res
            .iter()
            .map(|p| Product {
                name: p.name.clone(),
                company: p.company.clone(),
                variant_groups: serde_json::from_value::<VariantCategoryList>(
                    p.variant_groups.clone(),
                )
                .unwrap(),
                variants: serde_json::from_value::<Vec<VariantInformation>>(p.variants.clone())
                    .unwrap(),
                sku: p.sku.clone(),
                images: serde_json::from_value::<Vec<Url>>(p.images.clone()).unwrap(),
                tags: serde_json::from_value::<TagList>(p.tags.clone()).unwrap(),
                description: p.description.clone(),
                specifications: serde_json::from_value::<Vec<(String, String)>>(
                    p.specifications.clone(),
                )
                .unwrap(),
                identification: serde_json::from_value::<ProductIdentification>(
                    p.identification.clone(),
                )
                .unwrap(),
                visible: serde_json::from_value::<ProductVisibility>(p.visible.clone()).unwrap(),
                name_long: p.name_long.clone(),
                description_long: p.description_long.clone(),
            })
            .collect();

        Ok(mapped)
    }

    pub async fn update(pdt: Product, id: &str, db: &DbConn) -> Result<Product, DbErr> {
        products::ActiveModel {
            sku: Set(pdt.sku),
            name: Set(pdt.name),
            company: Set(pdt.company),
            variants: Set(json!(pdt.variants)),
            variant_groups: Set(json!(pdt.variant_groups)),
            images: Set(json!(pdt.images)),
            tags: Set(json!(pdt.tags)),
            description: Set(pdt.description),
            specifications: Set(json!(pdt.specifications)),
            identification: Set(json!(pdt.identification)),
            visible: Set(json!(pdt.visible)),
            name_long: Set(pdt.name_long),
            description_long: Set(pdt.description_long),
        }
        .update(db)
        .await?;

        Self::fetch_by_id(id, db).await
    }

    pub async fn fetch_all(db: &DbConn) -> Result<Vec<Product>, DbErr> {
        let products = Products::find().all(db).await?;

        let mapped = products
            .iter()
            .map(|p| Product {
                name: p.name.clone(),
                company: p.company.clone(),
                variant_groups: serde_json::from_value::<VariantCategoryList>(
                    p.variant_groups.clone(),
                )
                .unwrap(),
                variants: serde_json::from_value::<Vec<VariantInformation>>(p.variants.clone())
                    .unwrap(),
                sku: p.sku.clone(),
                images: serde_json::from_value::<Vec<Url>>(p.images.clone()).unwrap(),
                tags: serde_json::from_value::<TagList>(p.tags.clone()).unwrap(),
                description: p.description.clone(),
                specifications: serde_json::from_value::<Vec<(String, String)>>(
                    p.specifications.clone(),
                )
                .unwrap(),
                identification: serde_json::from_value::<ProductIdentification>(
                    p.identification.clone(),
                )
                .unwrap(),
                visible: serde_json::from_value::<ProductVisibility>(p.visible.clone()).unwrap(),
                name_long: p.name_long.clone(),
                description_long: p.description_long.clone(),
            })
            .collect();

        Ok(mapped)
    }

    pub async fn insert_many(
        products: Vec<Product>,
        db: &DbConn,
    ) -> Result<InsertResult<products::ActiveModel>, DbErr> {
        let entities = products.into_iter().map(|pdt| products::ActiveModel {
            sku: Set(pdt.sku),
            name: Set(pdt.name),
            company: Set(pdt.company),
            variants: Set(json!(pdt.variants)),
            variant_groups: Set(json!(pdt.variant_groups)),
            images: Set(json!(pdt.images)),
            tags: Set(json!(pdt.tags)),
            description: Set(pdt.description),
            specifications: Set(json!(pdt.specifications)),
            identification: Set(json!(pdt.identification)),
            visible: Set(json!(pdt.visible)),
            name_long: Set(pdt.name_long),
            description_long: Set(pdt.description_long),
        });

        match Products::insert_many(entities).exec(db).await {
            Ok(res) => Ok(res),
            Err(err) => Err(err),
        }
    }

    pub async fn generate(db: &DbConn) -> Result<Vec<Product>, DbErr> {
        let products = example_products();

        match Product::insert_many(products, db).await {
            Ok(_) => match Product::fetch_all(db).await {
                Ok(res) => Ok(res),
                Err(e) => Err(e),
            },
            Err(e) => Err(e),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FulfillmentStatus {
    pub pick_status: PickStatus,
    pub pick_history: Vec<History<PickStatus>>,
    pub last_updated: DateTime<Utc>,
    pub notes: Vec<Note>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum PickStatus {
    Pending,
    Picked,
    Failed,
    Uncertain,
    Processing,
    Other(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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

pub type ProductCode = String;
pub type ProductPurchaseList = Vec<ProductPurchase>;

fn example_products() -> Vec<Product> {
    let mt_wellington = Location {
        store_code: "001".into(),
        store_id: "628f74d7-de00-4956-a5b6-2031e0c72128".to_string(),
        contact: ContactInformation {
            name: "Torpedo7 Mt Wellington".into(),
            mobile: MobileNumber {
                number: "+6421212120".into(),
                valid: true,
            },
            email: Email {
                root: "order".into(),
                domain: "torpedo7.com".into(),
                full: "order@torpedo7.com".into(),
            },
            landline: "".into(),
            address: Address {
                street: "315-375 Mount Wellington Highway".into(),
                street2: "Mount Wellington".into(),
                city: "Auckland".into(),
                country: "New Zealand".into(),
                po_code: "1060".into(),
                lat: -36.915501,
                lon: 174.838745,
            },
        },
    };

    let westfield = Location {
        store_code: "002".into(),
        store_id: "c4a1d88b-e8a0-4dcd-ade2-1eea82254816".to_string(),
        contact: ContactInformation {
            name: "Torpedo7 Westfield".into(),
            mobile: MobileNumber {
                number: "+6421212120".into(),
                valid: true,
            },
            email: Email {
                root: "order".into(),
                domain: "torpedo7.com".into(),
                full: "order@torpedo7.com".into(),
            },
            landline: "".into(),
            address: Address {
                street: "309 Broadway, Westfield Shopping Centre".into(),
                street2: "Newmarket".into(),
                city: "Auckland".into(),
                country: "New Zealand".into(),
                po_code: "1023".into(),
                lat: -36.871820,
                lon: 174.776730,
            },
        },
    };

    let albany = Location {
        store_code: "003".into(),
        store_id: "a91509fa-2783-43ae-8c3c-5d5bc5cb6c95".to_string(),
        contact: ContactInformation {
            name: "Torpedo7 Albany".into(),
            mobile: MobileNumber {
                number: "+6421212120".into(),
                valid: true,
            },
            email: Email {
                root: "order".into(),
                domain: "torpedo7.com".into(),
                full: "order@torpedo7.com".into(),
            },
            landline: "".into(),
            address: Address {
                street: "6 Mercari Way".into(),
                street2: "Albany".into(),
                city: "Auckland".into(),
                country: "New Zealand".into(),
                po_code: "0632".into(),
                lat: -36.7323515,
                lon: 174.7082982,
            },
        },
    };

    vec![
        Product {
            name: "Explore Graphic Tee".into(),
            company: "Torpedo7".into(),
            identification: ProductIdentification::default(),
            visible: ProductVisibility::ShowWhenInStock,
            name_long: String::new(),
            description_long: String::new(),
            variant_groups: vec![
                VariantCategory {
                    category: "Colour".into(),
                    variants: vec![
                        Variant {
                            name: "White".into(),
                            images: vec![
                                "https://www.torpedo7.co.nz/images/products/T7TEO23YBHT_zoom---men-s-ecopulse-short-sleeve-explore-graphic-t-shirt-blanc-du-blanc.jpg?v=845eb9a5288642009c05".into(),
                            ],
                            marginal_price: 550.00,
                            variant_code: "01".into(),
                            order_history: vec![],
                        },
                        Variant {
                            name: "Black".into(),
                            images: vec![
                                "https://www.torpedo7.co.nz/images/products/T7TEO23YEAA_zoom---men-s-ecopulse-short-sleeve-explore-graphic-t-shirt-black.jpg?v=845eb9a5288642009c05".into(),
                            ],
                            marginal_price: 550.00,
                            variant_code: "02".into(),
                            order_history: vec![],
                        },
                        Variant {
                            name: "Hot Sauce".into(),
                            images: vec![
                                "https://www.torpedo7.co.nz/images/products/T7TEO23YDHS_zoom---men-s-ecopulse-short-sleeve-explore-graphic-t-shirts-hot-sauce.jpg?v=845eb9a5288642009c05".into(),
                            ],
                            marginal_price: 550.00,
                            variant_code: "03".into(),
                            order_history: vec![],
                        },
                        Variant {
                            name: "Tourmaline".into(),
                            images: vec![
                                "https://www.torpedo7.co.nz/images/products/T7TEO23YCJZ_zoom---men-s-ecopulse-short-sleeve-explore-graphic-t-shirt-tourmaline.jpg?v=845eb9a5288642009c05".into(),
                            ],
                            marginal_price: 550.00,
                            variant_code: "04".into(),
                            order_history: vec![],
                        },
                        Variant {
                            name: "Navy Blazer".into(),
                            images: vec![
                                "https://www.torpedo7.co.nz/images/products/T7TEO23YIWJ_zoom---men-s-ecopulse-short-sleeve-organic-chest-print-t-shirt-navy-blazer.jpg?v=845eb9a5288642009c05".into(),
                            ],
                            marginal_price: 550.00,
                            variant_code: "05".into(),
                            order_history: vec![],
                        }
                    ]
                },
                VariantCategory {
                    category: "Size".into(),
                    variants: vec![
                        Variant {
                            name: "Small".into(),
                            images: vec![
                                "https://www.torpedo7.co.nz/images/products/T7TEO23YIWJ_zoom---men-s-ecopulse-short-sleeve-organic-chest-print-t-shirt-navy-blazer.jpg?v=845eb9a5288642009c05".into()
                            ],
                            marginal_price: 550.00,
                            variant_code: "21".into(),
                            order_history: vec![],
                        },
                        Variant {
                            name: "Medium".into(),
                            images: vec![
                                "https://www.torpedo7.co.nz/images/products/T7TEO23YIWJ_zoom---men-s-ecopulse-short-sleeve-organic-chest-print-t-shirt-navy-blazer.jpg?v=845eb9a5288642009c05".into(),
                            ],
                            marginal_price: 550.00,
                            variant_code: "22".into(),
                            order_history: vec![],
                        },
                        Variant {
                            name: "Large".into(),
                            images: vec![
                                "https://www.torpedo7.co.nz/images/products/T7TEO23YIWJ_zoom---men-s-ecopulse-short-sleeve-organic-chest-print-t-shirt-navy-blazer.jpg?v=845eb9a5288642009c05".into(),
                            ],
                            marginal_price: 550.00,
                            variant_code: "23".into(),
                            order_history: vec![],
                        },
                        Variant {
                            name: "Extra Large".into(),
                            images: vec![
                                "https://www.torpedo7.co.nz/images/products/T7TEO23YIWJ_zoom---men-s-ecopulse-short-sleeve-organic-chest-print-t-shirt-navy-blazer.jpg?v=845eb9a5288642009c05".into()
                            ],
                            marginal_price: 550.00,
                            variant_code: "24".into(),
                            order_history: vec![],
                        }
                    ]
                }
            ],
            variants: vec![
                VariantInformation {
                    id: "SM-BLK-ITM".to_string(),
                    name: "Small Black".into(),
                    buy_min: 1.0,
                    buy_max: -1.0,
                    identification: ProductIdentification::default(),
                    stock_tracking: true,
                    stock: vec![
                        Stock {
                            store: mt_wellington.clone(),
                            quantity: Quantity {
                                quantity_sellable: 0.0,
                                quantity_on_order: 0.0,
                                quantity_unsellable: 0.0,
                                quantity_allocated: 0.0
                            }
                        },
                        Stock {
                            store: westfield.clone(),
                            quantity: Quantity {
                                quantity_sellable: 4.0,
                                quantity_on_order: 2.0,
                                quantity_unsellable: 2.0,
                                quantity_allocated: 0.0
                            }
                        },
                        Stock {
                            store: albany.clone(),
                            quantity: Quantity {
                                quantity_sellable: 1.0,
                                quantity_on_order: 0.0,
                                quantity_unsellable: 2.0,
                                quantity_allocated: 0.0
                            }
                        },
                    ],
                    images: vec![
                        "https://www.torpedo7.co.nz/images/products/T7TEO23YEAA_zoom---men-s-ecopulse-short-sleeve-explore-graphic-t-shirt-black.jpg?v=845eb9a5288642009c05".into()
                    ],
                    marginal_price: 10.99,
                    retail_price: 44.99,
                    variant_code: vec!["02".into(), "21".into()],
                    order_history: vec![],
                    barcode: "51890723908812".into(),
                    stock_information: StockInformation {
                        stock_group: "RANDOM".into(),
                        sales_group: "RANDOM".into(),
                        value_stream: "RANDOM".into(),
                        brand: "SELLER_GROUP".into(),
                        tax_code: "GSL".into(),
                        weight: "5.6".into(),
                        volume: "0.123".into(),
                        max_volume: "6.00".into(),
                        back_order: false,
                        discontinued: false,
                        non_diminishing: false,
                        shippable: true,
                        min_stock_before_alert: 2.0,
                        min_stock_level: 0.0,
                        colli: String::new(),
                        size_x: 0.0,
                        size_y: 0.0,
                        size_z: 0.0,
                        size_x_unit: "m".to_string(),
                        size_y_unit: "m".to_string(),
                        size_z_unit: "m".to_string(),
                        size_override_unit: "m".to_string()
                    },
                    loyalty_discount: DiscountValue::Absolute(15)
                },
                VariantInformation {
                    id: "M-BLK-ITM".to_string(),
                    name: "Medium Black".into(),
                    buy_min: 1.0,
                    buy_max: -1.0,
                    identification: ProductIdentification::default(),
                    stock_tracking: true,
                    stock: vec![
                        Stock {
                            store: mt_wellington.clone(),
                            quantity: Quantity {
                                quantity_sellable: 7.0,
                                quantity_unsellable: 2.0,
                                quantity_on_order: 4.0,
                                quantity_allocated: 0.0
                            }
                        },
                        Stock {
                            store: westfield.clone(),
                            quantity: Quantity {
                                quantity_sellable: 0.0,
                                quantity_on_order: 1.0,
                                quantity_unsellable: 0.0,
                                quantity_allocated: 0.0
                            }
                        },
                        Stock {
                            store: albany.clone(),
                            quantity: Quantity {
                                quantity_sellable: 1.0,
                                quantity_on_order: 0.0,
                                quantity_unsellable: 2.0,
                                quantity_allocated: 0.0
                            }
                        },
                    ],
                    images: vec![
                        "https://www.torpedo7.co.nz/images/products/T7TEO23YEAA_zoom---men-s-ecopulse-short-sleeve-explore-graphic-t-shirt-black.jpg?v=845eb9a5288642009c05".into()
                    ],
                    marginal_price: 12.49,
                    retail_price: 46.99,
                    variant_code: vec!["02".into(), "22".into()],
                    order_history: vec![],
                    barcode: "51150723152813".into(),
                    stock_information: StockInformation {
                        stock_group: "RANDOM".into(),
                        sales_group: "RANDOM".into(),
                        value_stream: "RANDOM".into(),
                        brand: "SELLER_GROUP".into(),
                        tax_code: "GSL".into(),
                        weight: "5.6".into(),
                        volume: "0.123".into(),
                        max_volume: "6.00".into(),
                        back_order: false,
                        discontinued: false,
                        non_diminishing: false,
                        shippable: true,
                        min_stock_before_alert: 2.0,
                        min_stock_level: 0.0,
                        colli: String::new(),
                        size_x: 0.0,
                        size_y: 0.0,
                        size_z: 0.0,
                        size_x_unit: "m".to_string(),
                        size_y_unit: "m".to_string(),
                        size_z_unit: "m".to_string(),
                        size_override_unit: "m".to_string()
                    },
                    loyalty_discount: DiscountValue::Absolute(25)
                },
                VariantInformation {
                    id: "LG-WHT-ITM".to_string(),
                    name: "Large White".into(),
                    buy_min: 1.0,
                    buy_max: -1.0,
                    identification: ProductIdentification::default(),
                    stock_tracking: true,
                    stock: vec![
                        Stock {
                            store: mt_wellington.clone(),
                            quantity: Quantity {
                                quantity_sellable: 1.0,
                                quantity_on_order: 0.0,
                                quantity_unsellable: 2.0,
                                quantity_allocated: 0.0
                            }
                        },
                        Stock {
                            store: westfield.clone(),
                            quantity: Quantity {
                                quantity_sellable: 3.0,
                                quantity_on_order: 0.0,
                                quantity_unsellable: 1.0,
                                quantity_allocated: 0.0
                            }
                        },
                        Stock {
                            store: albany.clone(),
                            quantity: Quantity {
                                quantity_sellable: 1.0,
                                quantity_on_order: 0.0,
                                quantity_unsellable: 2.0,
                                quantity_allocated: 0.0
                            }
                        },
                    ],
                    images: vec![
                        "https://www.torpedo7.co.nz/images/products/T7TEO23YBHT_zoom---men-s-ecopulse-short-sleeve-explore-graphic-t-shirt-blanc-du-blanc.jpg?v=845eb9a5288642009c05".into()
                    ],
                    variant_code: vec!["01".into(), "23".into()],
                    order_history: vec![],
                    marginal_price: 16.09,
                    retail_price: 49.99,
                    barcode: "51150723159173".into(),
                    stock_information: StockInformation {
                        stock_group: "RANDOM".into(),
                        sales_group: "RANDOM".into(),
                        value_stream: "RANDOM".into(),
                        brand: "SELLER_GROUP".into(),
                        tax_code: "GSL".into(),
                        weight: "5.6".into(),
                        volume: "0.123".into(),
                        max_volume: "6.00".into(),
                        back_order: false,
                        discontinued: false,
                        non_diminishing: false,
                        shippable: true,
                        min_stock_before_alert: 2.0,
                        min_stock_level: 0.0,
                        colli: String::new(),
                        size_x: 0.0,
                        size_y: 0.0,
                        size_z: 0.0,
                        size_x_unit: "m".to_string(),
                        size_y_unit: "m".to_string(),
                        size_z_unit: "m".to_string(),
                        size_override_unit: "m".to_string()
                    },
                    loyalty_discount: DiscountValue::Absolute(5)
                },
            ],
            sku: 123456.to_string(),
            images: vec![
                "https://www.torpedo7.co.nz/images/products/T7TEOQR5NDD_zoom---men-s-short-sleeve-explore-graphic-tee-ochre-rose.jpg".into()
            ],
            tags: vec![
                "Tee".into(),
                "Cotton".into(),
                "Organic".into()
            ],
            description: "Made with organically grown cotton to reflect our love of the planet and the people on it.".into(),
            specifications: vec![
                ("".into(), "Soft cotton tee".into()),
                ("".into(), "100% Organically Grown Cotton. Uses Less Water. No pesticides used on crops. Supports Regenerative Agriculture".into()),
                ("".into(), "Composition: 100% Organic cotton".into())
            ]
        },
        Product {
            name: "Nippers Kids Kayak & Paddle".into(),
            company: "Torpedo7".into(),
            identification: ProductIdentification::default(),
            visible: ProductVisibility::ShowWhenInStock,
            name_long: String::new(),
            description_long: String::new(),
            variant_groups: vec![
                VariantCategory {
                    category: "Colour".into(),
                    variants: vec![
                        Variant {
                            name: "Beaches".into(),
                            images: vec![
                                "https://www.torpedo7.co.nz/images/products/T7KKK23NB0Z_zoom---2023-nippers-kids-kayak---paddle-1-83m-beaches.jpg?v=99f4b292748848b5b1d6".into(),
                            ],
                            marginal_price: 399.99,
                            variant_code: "01".into(),
                            order_history: vec![],
                        },
                        Variant {
                            name: "Tropics".into(),
                            images: vec![
                                "https://www.torpedo7.co.nz/images/products/T7KKK23NTOY_zoom---2023-nippers-kids-kayak---paddle-1-83m-tropics.jpg?v=99f4b292748848b5b1d6".into(),
                            ],
                            marginal_price: 399.99,
                            variant_code: "02".into(),
                            order_history: vec![],
                        }
                    ]
                },
                VariantCategory {
                    category: "Size".into(),
                    variants: vec![
                        Variant {
                            name: "1.83m".into(),
                            images: vec![
                                "https://www.torpedo7.co.nz/images/products/T7KKK23NTOY_zoom---2023-nippers-kids-kayak---paddle-1-83m-tropics.jpg?v=99f4b292748848b5b1d6".into()
                            ],
                            marginal_price: 399.99,
                            variant_code: "21".into(),
                            order_history: vec![],
                        }
                    ]
                }
            ],
            variants: vec![
                VariantInformation {
                    id: "1.83-BEACHES".to_string(),
                    name: "1.83m Beaches".into(),
                    buy_min: 1.0,
                    buy_max: -1.0,
                    identification: ProductIdentification::default(),
                    stock_tracking: true,
                    stock: vec![
                        Stock {
                            store: mt_wellington.clone(),
                            quantity: Quantity {
                                quantity_sellable: 4.0,
                                quantity_on_order: 0.0,
                                quantity_unsellable: 2.0,
                                quantity_allocated: 0.0
                            }
                        },
                        Stock {
                            store: westfield.clone(),
                            quantity: Quantity {
                                quantity_sellable: 1.0,
                                quantity_on_order: 1.0,
                                quantity_unsellable: 0.0,
                                quantity_allocated: 0.0
                            }
                        },
                        Stock {
                            store: albany.clone(),
                            quantity: Quantity {
                                quantity_sellable: 0.0,
                                quantity_on_order: 2.0,
                                quantity_unsellable: 1.0,
                                quantity_allocated: 0.0
                            }
                        },
                    ],
                    images: vec![
                        "https://www.torpedo7.co.nz/images/products/T7KKK23NB0Z_zoom---2023-nippers-kids-kayak---paddle-1-83m-beaches.jpg".into()
                    ],
                    marginal_price: 85.99,
                    retail_price: 399.99,
                    variant_code: vec!["01".into(), "21".into()],
                    order_history: vec![],
                    barcode: "51891743988214".into(),
                    stock_information: StockInformation {
                        stock_group: "RANDOM".into(),
                        sales_group: "RANDOM".into(),
                        value_stream: "RANDOM".into(),
                        brand: "SELLER_GROUP".into(),
                        tax_code: "GSL".into(),
                        weight: "5.6".into(),
                        volume: "0.123".into(),
                        max_volume: "6.00".into(),
                        back_order: false,
                        discontinued: false,
                        non_diminishing: false,
                        shippable: true,
                        min_stock_before_alert: 2.0,
                        min_stock_level: 0.0,
                        colli: String::new(),
                        size_x: 0.0,
                        size_y: 0.0,
                        size_z: 0.0,
                        size_x_unit: "m".to_string(),
                        size_y_unit: "m".to_string(),
                        size_z_unit: "m".to_string(),
                        size_override_unit: "m".to_string()
                    },
                    loyalty_discount: DiscountValue::Absolute(15)
                },
                VariantInformation {
                    id: "1.83-TROPICS".to_string(),
                    name: "1.83m Tropics".into(),
                    buy_min: 1.0,
                    buy_max: -1.0,
                    identification: ProductIdentification::default(),
                    stock_tracking: true,
                    stock: vec![
                        Stock {
                            store: mt_wellington.clone(),
                            quantity: Quantity {
                                quantity_sellable: 7.0,
                                quantity_unsellable: 2.0,
                                quantity_on_order: 4.0,
                                quantity_allocated: 0.0
                            }
                        },
                        Stock {
                            store: westfield.clone(),
                            quantity: Quantity {
                                quantity_sellable: 0.0,
                                quantity_on_order: 1.0,
                                quantity_unsellable: 0.0,
                                quantity_allocated: 0.0
                            }
                        },
                        Stock {
                            store: albany.clone(),
                            quantity: Quantity {
                                quantity_sellable: 1.0,
                                quantity_on_order: 0.0,
                                quantity_unsellable: 2.0,
                                quantity_allocated: 0.0
                            }
                        },
                    ],
                    images: vec![
                        "https://www.torpedo7.co.nz/images/products/T7KKK23NB0Z_zoom---2023-nippers-kids-kayak---paddle-1-83m-beaches.jpg".into()
                    ],
                    marginal_price: 85.99,
                    retail_price: 399.99,
                    variant_code: vec!["02".into(), "21".into()],
                    order_history: vec![],
                    barcode: "54897443288214".into(),
                    stock_information: StockInformation {
                        stock_group: "RANDOM".into(),
                        sales_group: "RANDOM".into(),
                        value_stream: "RANDOM".into(),
                        brand: "SELLER_GROUP".into(),
                        tax_code: "GSL".into(),
                        weight: "5.6".into(),
                        volume: "0.123".into(),
                        max_volume: "6.00".into(),
                        back_order: false,
                        discontinued: false,
                        non_diminishing: false,
                        shippable: true,
                        min_stock_before_alert: 2.0,
                        min_stock_level: 0.0,
                        colli: String::new(),
                        size_x: 0.0,
                        size_y: 0.0,
                        size_z: 0.0,
                        size_x_unit: "m".to_string(),
                        size_y_unit: "m".to_string(),
                        size_z_unit: "m".to_string(),
                        size_override_unit: "m".to_string()
                    },
                    loyalty_discount: DiscountValue::Absolute(25)
                },
            ],
            sku: 654321.to_string(),
            images: vec![
                "https://www.torpedo7.co.nz/images/products/T7KKK23NB0Z_zoom---2023-nippers-kids-kayak---paddle-1-83m-beaches.jpg".into()
            ],
            tags: vec![
                "Kayak".into(),
                "Kids".into(),
                "Recreational".into(),
                "Water".into()
            ],
            description: "The Nippers Kayak is the ideal size for kid's wanting their own independence. The compact, lightweight design with a V shaped hull and centre fin allows the kayak to track and manoeuver easily. They’ll also enjoy the multiple foot holds to accommodate various heights and a lightweight, flexible paddle so they can push through water without too much resistance. With the Nippers Kayak from Torpedo7, there are no excuses for a bad day with mum and dad.".into(),
            specifications: vec![
                ("Length".into(), "183cm".into()),
                ("Width".into(), "70cm".into()),
                ("Height".into(), "23cm".into()),
                ("Gross Weight".into(), "9kg".into()),
                ("Weight Capacity".into(), "50kg".into())
            ]
        },
        Product {
            name: "Kids Voyager II Paddle Vest".into(),
            company: "Torpedo7".into(),
            identification: ProductIdentification::default(),
            visible: ProductVisibility::ShowWhenInStock,
            name_long: String::new(),
            description_long: String::new(),
            variant_groups: vec![
                VariantCategory {
                    category: "Colour".into(),
                    variants: vec![
                        Variant {
                            name: "Red".into(),
                            images: vec![
                                "https://www.torpedo7.co.nz/images/products/T7LJJ22DFRA_zoom---kids-voyager-ii-paddle-vest-red.jpg?v=99f4b292748848b5b1d6".into(),
                            ],
                            marginal_price: 139.99,
                            variant_code: "01".into(),
                            order_history: vec![],
                        }
                    ]
                },
                VariantCategory {
                    category: "Size".into(),
                    variants: vec![
                        Variant {
                            name: "4-6".into(),
                            images: vec![
                                "https://www.torpedo7.co.nz/images/products/T7LJJ22DFRA_zoom---kids-voyager-ii-paddle-vest-red.jpg?v=99f4b292748848b5b1d6".into()
                            ],
                            marginal_price: 139.99,
                            variant_code: "21".into(),
                            order_history: vec![],
                        },
                        Variant {
                            name: "8-10".into(),
                            images: vec![
                                "https://www.torpedo7.co.nz/images/products/T7LJJ22DFRA_zoom---kids-voyager-ii-paddle-vest-red.jpg?v=99f4b292748848b5b1d6".into()
                            ],
                            marginal_price: 139.99,
                            variant_code: "22".into(),
                            order_history: vec![],
                        },
                        Variant {
                            name: "12-14".into(),
                            images: vec![
                                "https://www.torpedo7.co.nz/images/products/T7LJJ22DFRA_zoom---kids-voyager-ii-paddle-vest-red.jpg?v=99f4b292748848b5b1d6".into()
                            ],
                            marginal_price: 139.99,
                            variant_code: "23".into(),
                            order_history: vec![],
                        },
                    ]
                }
            ],
            variants: vec![
                VariantInformation {
                    id: "S-RED".to_string(),
                    name: "Small Red (4-6y)".into(),
                    buy_min: 1.0,
                    buy_max: -1.0,
                    identification: ProductIdentification::default(),
                    stock_tracking: true,
                    stock: vec![
                        Stock {
                            store: mt_wellington.clone(),
                            quantity: Quantity {
                                quantity_sellable: 4.0,
                                quantity_on_order: 0.0,
                                quantity_unsellable: 2.0,
                                quantity_allocated: 0.0
                            }
                        },
                        Stock {
                            store: westfield.clone(),
                            quantity: Quantity {
                                quantity_sellable: 1.0,
                                quantity_on_order: 1.0,
                                quantity_unsellable: 0.0,
                                quantity_allocated: 0.0
                            }
                        },
                        Stock {
                            store: albany.clone(),
                            quantity: Quantity {
                                quantity_sellable: 0.0,
                                quantity_on_order: 2.0,
                                quantity_unsellable: 1.0,
                                quantity_allocated: 0.0
                            }
                        },
                    ],
                    images: vec![
                        "https://www.torpedo7.co.nz/images/products/T7LJJ22DFRA_zoom---kids-voyager-ii-paddle-vest-red.jpg?v=99f4b292748848b5b1d6".into()
                    ],
                    marginal_price: 45.99,
                    retail_price: 139.99,
                    variant_code: vec!["01".into(), "21".into()],
                    order_history: vec![],
                    barcode: "51891265958214".into(),
                    stock_information: StockInformation {
                        stock_group: "RANDOM".into(),
                        sales_group: "RANDOM".into(),
                        value_stream: "RANDOM".into(),
                        brand: "SELLER_GROUP".into(),
                        tax_code: "GSL".into(),
                        weight: "5.6".into(),
                        volume: "0.123".into(),
                        max_volume: "6.00".into(),
                        back_order: false,
                        discontinued: false,
                        non_diminishing: false,
                        shippable: true,
                        min_stock_before_alert: 2.0,
                        min_stock_level: 0.0,
                        colli: String::new(),
                        size_x: 0.0,
                        size_y: 0.0,
                        size_z: 0.0,
                        size_x_unit: "m".to_string(),
                        size_y_unit: "m".to_string(),
                        size_z_unit: "m".to_string(),
                        size_override_unit: "m".to_string()
                    },
                    loyalty_discount: DiscountValue::Absolute(15)
                },
                VariantInformation {
                    id: "M-RED".to_string(),
                    name: "Medium Red (8-10y)".into(),
                    buy_min: 1.0,
                    buy_max: -1.0,
                    identification: ProductIdentification::default(),
                    stock_tracking: true,
                    stock: vec![
                        Stock {
                            store: mt_wellington.clone(),
                            quantity: Quantity {
                                quantity_sellable: 4.0,
                                quantity_on_order: 0.0,
                                quantity_unsellable: 2.0,
                                quantity_allocated: 0.0
                            }
                        },
                        Stock {
                            store: westfield.clone(),
                            quantity: Quantity {
                                quantity_sellable: 1.0,
                                quantity_on_order: 1.0,
                                quantity_unsellable: 0.0,
                                quantity_allocated: 0.0
                            }
                        },
                        Stock {
                            store: albany.clone(),
                            quantity: Quantity {
                                quantity_sellable: 0.0,
                                quantity_on_order: 2.0,
                                quantity_unsellable: 1.0,
                                quantity_allocated: 0.0
                            }
                        },
                    ],
                    images: vec![
                        "https://www.torpedo7.co.nz/images/products/T7LJJ22DFRA_zoom---kids-voyager-ii-paddle-vest-red.jpg?v=99f4b292748848b5b1d6".into()
                    ],
                    marginal_price: 45.99,
                    retail_price: 139.99,
                    variant_code: vec!["01".into(), "22".into()],
                    order_history: vec![],
                    barcode: "51893261953216".into(),
                    stock_information: StockInformation {
                        stock_group: "RANDOM".into(),
                        sales_group: "RANDOM".into(),
                        value_stream: "RANDOM".into(),
                        brand: "SELLER_GROUP".into(),
                        tax_code: "GSL".into(),
                        weight: "5.6".into(),
                        volume: "0.123".into(),
                        max_volume: "6.00".into(),
                        back_order: false,
                        discontinued: false,
                        non_diminishing: false,
                        shippable: true,
                        min_stock_before_alert: 2.0,
                        min_stock_level: 0.0,
                        colli: String::new(),
                        size_x: 0.0,
                        size_y: 0.0,
                        size_z: 0.0,
                        size_x_unit: "m".to_string(),
                        size_y_unit: "m".to_string(),
                        size_z_unit: "m".to_string(),
                        size_override_unit: "m".to_string()
                    },
                    loyalty_discount: DiscountValue::Absolute(15)
                },
                VariantInformation {
                    id: "L-RED".to_string(),
                    name: "Large Red (12-14y)".into(),
                    buy_min: 1.0,
                    buy_max: -1.0,
                    identification: ProductIdentification::default(),
                    stock_tracking: true,
                    stock: vec![
                        Stock {
                            store: mt_wellington,
                            quantity: Quantity {
                                quantity_sellable: 4.0,
                                quantity_on_order: 0.0,
                                quantity_unsellable: 2.0,
                                quantity_allocated: 0.0
                            }
                        },
                        Stock {
                            store: westfield,
                            quantity: Quantity {
                                quantity_sellable: 1.0,
                                quantity_on_order: 1.0,
                                quantity_unsellable: 0.0,
                                quantity_allocated: 0.0
                            }
                        },
                        Stock {
                            store: albany,
                            quantity: Quantity {
                                quantity_sellable: 0.0,
                                quantity_on_order: 2.0,
                                quantity_unsellable: 1.0,
                                quantity_allocated: 0.0
                            }
                        },
                    ],
                    images: vec![
                        "https://www.torpedo7.co.nz/images/products/T7LJJ22DFRA_zoom---kids-voyager-ii-paddle-vest-red.jpg?v=99f4b292748848b5b1d6".into()
                    ],
                    marginal_price: 45.99,
                    retail_price: 139.99,
                    variant_code: vec!["01".into(), "23".into()],
                    order_history: vec![],
                    barcode: "52496265958214".into(),
                    stock_information: StockInformation {
                        stock_group: "RANDOM".into(),
                        sales_group: "RANDOM".into(),
                        value_stream: "RANDOM".into(),
                        brand: "SELLER_GROUP".into(),
                        tax_code: "GSL".into(),
                        weight: "5.6".into(),
                        volume: "0.123".into(),
                        max_volume: "6.00".into(),
                        back_order: false,
                        discontinued: false,
                        non_diminishing: false,
                        shippable: true,
                        min_stock_before_alert: 2.0,
                        min_stock_level: 0.0,
                        colli: String::new(),
                        size_x: 0.0,
                        size_y: 0.0,
                        size_z: 0.0,
                        size_x_unit: "m".to_string(),
                        size_y_unit: "m".to_string(),
                        size_z_unit: "m".to_string(),
                        size_override_unit: "m".to_string()
                    },
                    loyalty_discount: DiscountValue::Absolute(15)
                },
            ],
            sku: 162534.to_string(),
            images: vec![
                "https://www.torpedo7.co.nz/images/products/T7LJJ22DFRA_zoom---kids-voyager-ii-paddle-vest-red.jpg".into()
            ],
            tags: vec![
                "Kayak".into(),
                "Kids".into(),
                "Recreational".into(),
                "Water".into()
            ],
            description: "The Nippers Kayak is the ideal size for kid's wanting their own independence.  The compact, lightweight design with a V shaped hull and centre fin allows the kayak to track and manoeuvre easily. They’ll also enjoy the multiple foot holds to accommodate various heights and a lightweight, flexible paddle so they can push through water without too much resistance. With the Nippers Kayak from Torpedo7, there are no excuses for a bad day with mum and dad.".into(),
            specifications: vec![
                ("Length".into(), "183cm".into()),
                ("Width".into(), "70cm".into()),
                ("Height".into(), "23cm".into()),
                ("Gross Weight".into(), "9kg".into()),
                ("Weight Capacity".into(), "50kg".into())
            ]
        }
    ]
}
