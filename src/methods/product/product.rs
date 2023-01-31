use std::{fmt::Display};

use chrono::{Utc, DateTime};
use sea_orm::{DbConn, DbErr, EntityTrait, Set, QuerySelect, ColumnTrait, InsertResult, ActiveModelTrait, Condition, QueryFilter, sea_query::{Expr, Func}};
use serde::{Serialize, Deserialize};
use serde_json::json;

use crate::{methods::{Url, TagList, DiscountValue, Location, ContactInformation, Stock, MobileNumber, Email, Address, Quantity, DiscountMap}, entities::{sea_orm_active_enums::TransactionType, products, promotion}};
use super::{VariantCategoryList, VariantIdTag, VariantCategory, Variant, StockInformation, VariantInformation, Promotion, PromotionGet, PromotionBuy};
use crate::entities::prelude::Products;
use crate::entities::prelude::Promotion as Promotions;
use futures::future::join_all;

#[derive(Deserialize, Serialize, Clone)]
/// A product, containing a list of `Vec<Variant>`, an identifiable `sku` along with identifying information such as `tags`, `description` and `specifications`.
/// > Stock-relevant information about a product is kept under each variant, thus allowing for modularity of different variants and a fine-grained control over your inventory. 
pub struct Product {
    pub name: String,
    pub company: String,

    pub variant_groups: VariantCategoryList,
    /// Lists all the **possible** combinations of a product in terms of its variants.
    pub variants: Vec<VariantInformation>, 

    pub sku: String,
    pub images: Vec<Url>,
    pub tags: TagList,
    pub description: String,
    pub specifications: Vec<(String, String)>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct ProductWPromotion {
    pub product: Product,
    pub promotions: Vec<Promotion>
}

impl Display for Product {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let variant_categories: String = self.variant_groups
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
            variant_groups: Set(json!(pdt.variant_groups)),
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
            variant_groups: serde_json::from_value::<VariantCategoryList>(p.variant_groups).unwrap(), 
            variants: serde_json::from_value::<Vec<VariantInformation>>(p.variants).unwrap(), 
            sku: p.sku, 
            images: serde_json::from_value::<Vec<Url>>(p.images).unwrap(), 
            tags: serde_json::from_value::<TagList>(p.tags).unwrap(), 
            description: p.description, 
            specifications: serde_json::from_value::<Vec<(String, String)>>(p.specifications).unwrap() 
        })
    }

    pub async fn fetch_by_id_with_promotion(id: &str, db: &DbConn) -> Result<ProductWPromotion, DbErr> {
        let pdt = Products::find_by_id(id.to_string()).one(db).await?;
        let promos = Promotions::find()
            .filter(
                Condition::any()
                    .add(promotion::Column::Buy.contains(id))
                    .add(promotion::Column::Get.contains(id))
                    .add(promotion::Column::ValidTill.gte(Utc::now()))
            )
            .all(db).await?;

        let mapped: Vec<Promotion> = promos.iter().map(|p| 
            Promotion { 
                name: p.name.clone(), 
                buy: serde_json::from_value::<PromotionBuy>(p.buy.clone()).unwrap(),
                get: serde_json::from_value::<PromotionGet>(p.get.clone()).unwrap(), 
                id: p.id.clone(), 
                valid_till: DateTime::from_utc(p.valid_till, Utc), 
                timestamp: DateTime::from_utc(p.timestamp, Utc), 
            }
        ).collect();
    
        let p = pdt.unwrap();

        Ok(
            ProductWPromotion {
                product: Product { 
                    name: p.name, 
                    company: p.company,
                    variant_groups: serde_json::from_value::<VariantCategoryList>(p.variant_groups).unwrap(), 
                    variants: serde_json::from_value::<Vec<VariantInformation>>(p.variants).unwrap(), 
                    sku: p.sku, 
                    images: serde_json::from_value::<Vec<Url>>(p.images).unwrap(), 
                    tags: serde_json::from_value::<TagList>(p.tags).unwrap(), 
                    description: p.description, 
                    specifications: serde_json::from_value::<Vec<(String, String)>>(p.specifications).unwrap() 
                },
                promotions: mapped
            }
        )
    }

    pub async fn search(query: &str, db: &DbConn) -> Result<Vec<Product>, DbErr> {
        let res = products::Entity::find()
            .filter(
                Condition::any()
                    .add(Expr::expr(Func::lower(Expr::col(products::Column::Name))).like(format!("%{}%", query)))
                    .add(products::Column::Sku.contains(query))
                    .add(products::Column::Variants.contains(query))
            )
            .limit(25)
            .all(db).await?;

        let mapped = res.iter().map(|p| 
            Product { 
                name: p.name.clone(), 
                company: p.company.clone(),
                variant_groups: serde_json::from_value::<VariantCategoryList>(p.variant_groups.clone()).unwrap(), 
                variants: serde_json::from_value::<Vec<VariantInformation>>(p.variants.clone()).unwrap(), 
                sku: p.sku.clone(), 
                images: serde_json::from_value::<Vec<Url>>(p.images.clone()).unwrap(), 
                tags: serde_json::from_value::<TagList>(p.tags.clone()).unwrap(), 
                description: p.description.clone(), 
                specifications: serde_json::from_value::<Vec<(String, String)>>(p.specifications.clone()).unwrap() 
            }
        ).collect();

        Ok(mapped)
    }

    pub async fn search_with_promotion(query: &str, db: &DbConn) -> Result<Vec<ProductWPromotion>, DbErr> {
        let res = products::Entity::find()
            .filter(
                Condition::any()
                    .add(Expr::expr(Func::lower(Expr::col(products::Column::Name))).like(format!("%{}%", query)))
                    .add(products::Column::Sku.contains(query))
                    .add(products::Column::Variants.contains(query))
            )
            .limit(25)
            .all(db).await?;
        
        let mapped: Vec<ProductWPromotion> = res.iter().map(|p| {
            ProductWPromotion {
                product: Product { 
                    name: p.name.clone(), 
                    company: p.company.clone(),
                    variant_groups: serde_json::from_value::<VariantCategoryList>(p.variant_groups.clone()).unwrap(), 
                    variants: serde_json::from_value::<Vec<VariantInformation>>(p.variants.clone()).unwrap(), 
                    sku: p.sku.clone(), 
                    images: serde_json::from_value::<Vec<Url>>(p.images.clone()).unwrap(), 
                    tags: serde_json::from_value::<TagList>(p.tags.clone()).unwrap(), 
                    description: p.description.clone(), 
                    specifications: serde_json::from_value::<Vec<(String, String)>>(p.specifications.clone()).unwrap() 
                },
                promotions: vec![]
            }
        }).collect();

        let with_promotions = join_all(mapped.iter().map(|p| async move {
            let b = db.clone();

            let promos 
                = Promotions::find()
                .filter(
                    Condition::any()
                        .add(promotion::Column::Buy.contains(&p.product.sku))
                        .add(promotion::Column::Get.contains(&p.product.sku))
                        .add(promotion::Column::ValidTill.gte(Utc::now()))
                )
                .limit(25)
                .all(&b).await.unwrap();

            let mapped: Vec<Promotion> = promos.iter().map(|p| 
                Promotion { 
                    name: p.name.clone(), 
                    buy: serde_json::from_value::<PromotionBuy>(p.buy.clone()).unwrap(),
                    get: serde_json::from_value::<PromotionGet>(p.get.clone()).unwrap(), 
                    id: p.id.clone(), 
                    valid_till: DateTime::from_utc(p.valid_till, Utc), 
                    timestamp: DateTime::from_utc(p.timestamp, Utc), 
                }
            ).collect();

            ProductWPromotion {
                product: p.product.clone(),
                promotions: mapped
            }
        })).await;

        Ok(with_promotions)
    }

    pub async fn fetch_by_name(name: &str, db: &DbConn) -> Result<Vec<Product>, DbErr> {
        let res = products::Entity::find()
            .having(products::Column::Name.contains(name))
            .limit(25)
            .all(db).await?;
            
        let mapped = res.iter().map(|p| 
            Product { 
                name: p.name.clone(), 
                company: p.company.clone(),
                variant_groups: serde_json::from_value::<VariantCategoryList>(p.variant_groups.clone()).unwrap(), 
                variants: serde_json::from_value::<Vec<VariantInformation>>(p.variants.clone()).unwrap(), 
                sku: p.sku.clone(), 
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
            .limit(25)
            .all(db).await?;
            
        let mapped = res.iter().map(|p| 
            Product { 
                name: p.name.clone(), 
                company: p.company.clone(),
                variant_groups: serde_json::from_value::<VariantCategoryList>(p.variant_groups.clone()).unwrap(), 
                variants: serde_json::from_value::<Vec<VariantInformation>>(p.variants.clone()).unwrap(),  
                sku: p.sku.clone(), 
                images: serde_json::from_value::<Vec<Url>>(p.images.clone()).unwrap(), 
                tags: serde_json::from_value::<TagList>(p.tags.clone()).unwrap(), 
                description: p.description.clone(), 
                specifications: serde_json::from_value::<Vec<(String, String)>>(p.specifications.clone()).unwrap() 
            }
        ).collect();

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
        }.update(db).await?;

        Self::fetch_by_id(id, db).await
    }

    pub async fn fetch_all(db: &DbConn) -> Result<Vec<Product>, DbErr> {
        let products = Products::find().all(db).await?;

        let mapped = products.iter().map(|p| 
            Product { 
                name: p.name.clone(), 
                company: p.company.clone(),
                variant_groups: serde_json::from_value::<VariantCategoryList>(p.variant_groups.clone()).unwrap(), 
                variants: serde_json::from_value::<Vec<VariantInformation>>(p.variants.clone()).unwrap(),  
                sku: p.sku.clone(), 
                images: serde_json::from_value::<Vec<Url>>(p.images.clone()).unwrap(), 
                tags: serde_json::from_value::<TagList>(p.tags.clone()).unwrap(), 
                description: p.description.clone(), 
                specifications: serde_json::from_value::<Vec<(String, String)>>(p.specifications.clone()).unwrap() 
            }
        ).collect();
        
        Ok(mapped)
    }

    pub async fn insert_many(products: Vec<Product>, db: &DbConn) -> Result<InsertResult<products::ActiveModel>, DbErr> {
        let entities = products.into_iter().map(|pdt| {
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
            }
        });

        match Products::insert_many(entities).exec(db).await {
            Ok(res) => Ok(res),
            Err(err) => Err(err)
        }
    }

    pub async fn generate(db: &DbConn) -> Result<Vec<Product>, DbErr> {
        let products = example_products();

        match Product::insert_many(products, db).await {
            Ok(_) => {
                match Product::fetch_all(db).await {
                    Ok(res) => {
                        Ok(res)
                    },  
                    Err(e) => Err(e)
                }
            },
            Err(e) => Err(e)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductPurchase {
    // Includes variant
    pub product_code: ProductCode,
    pub variant: VariantIdTag,
    pub discount: DiscountMap,
    pub product_name: String,

    pub id: String,

    // Cost before discount, discount will be applied on the product cost.
    pub product_cost: f32,
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
            TransactionType::PendingIn => "PENDING-IN",
            TransactionType::PendingOut => "PENDING-OUT"
        };

        write!(f, "{}: {}-{} x{}", method, self.product_code, self.variant.concat(), self.quantity)
    }
}

pub type ProductCode = String;
pub type ProductPurchaseList = Vec<ProductPurchase>;

fn example_products() -> Vec<Product> {
    let mt_wellington = Location {
        code: "001".into(),
        contact: ContactInformation {
            name: "Torpedo7 Mt Wellington".into(),
            mobile: MobileNumber {
                region_code: "+64".into(),
                root: "021212120".into()
            },
            email: Email {
                root: "order".into(),
                domain: "torpedo7.com".into(),
                full: "order@torpedo7.com".into()
            },
            landline: "".into(),
            address: Address {
                street: "315-375 Mount Wellington Highway".into(),
                street2: "Mount Wellington".into(),
                city: "Auckland".into(),
                country: "New Zealand".into(),
                po_code: "1060".into(),
                lat: -36.915501,
                lon: 174.838745
            }
        }
    };

    let westfield = Location {
        code: "002".into(),
        contact: ContactInformation {
            name: "Torpedo7 Westfield".into(),
            mobile: MobileNumber {
                region_code: "+64".into(),
                root: "021212120".into()
            },
            email: Email {
                root: "order".into(),
                domain: "torpedo7.com".into(),
                full: "order@torpedo7.com".into()
            },
            landline: "".into(),
            address: Address {
                street: "309 Broadway, Westfield Shopping Centre".into(),
                street2: "Newmarket".into(),
                city: "Auckland".into(),
                country: "New Zealand".into(),
                po_code: "1023".into(),
                lat: -36.871820,
                lon: 174.776730
            }
        }
    };

    let albany = Location {
        code: "003".into(),
        contact: ContactInformation {
            name: "Torpedo7 Albany".into(),
            mobile: MobileNumber {
                region_code: "+64".into(),
                root: "021212120".into()
            },
            email: Email {
                root: "order".into(),
                domain: "torpedo7.com".into(),
                full: "order@torpedo7.com".into()
            },
            landline: "".into(),
            address: Address {
                street: "6 Mercari Way".into(),
                street2: "Albany".into(),
                city: "Auckland".into(),
                country: "New Zealand".into(),
                po_code: "0632".into(),
                lat: -36.7323515,
                lon: 174.7082982
            }
        }
    };

    vec![
        Product { 
            name: "Explore Graphic Tee".into(), 
            company: "Torpedo7".into(), 
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
                        unit: "".into(), 
                        tax_code: "GSL".into(), 
                        weight: "5.6".into(), 
                        volume: "0.123".into(), 
                        max_volume: "6.00".into(), 
                        back_order: false, 
                        discontinued: false, 
                        non_diminishing: false,
                        shippable: true
                    },
                    loyalty_discount: DiscountValue::Absolute(15)
                },
                VariantInformation { 
                    id: "M-BLK-ITM".to_string(),
                    name: "Medium Black".into(), 
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
                        unit: "".into(), 
                        tax_code: "GSL".into(), 
                        weight: "5.6".into(), 
                        volume: "0.123".into(), 
                        max_volume: "6.00".into(), 
                        back_order: false, 
                        discontinued: false, 
                        non_diminishing: false,
                        shippable: true
                    },
                    loyalty_discount: DiscountValue::Absolute(25)
                },
                VariantInformation { 
                    id: "LG-WHT-ITM".to_string(),
                    name: "Large White".into(), 
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
                        unit: "".into(), 
                        tax_code: "GSL".into(), 
                        weight: "5.6".into(), 
                        volume: "0.123".into(), 
                        max_volume: "6.00".into(), 
                        back_order: false, 
                        discontinued: false, 
                        non_diminishing: false,
                        shippable: true
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
                        unit: "".into(), 
                        tax_code: "GSL".into(), 
                        weight: "5.6".into(), 
                        volume: "0.123".into(), 
                        max_volume: "6.00".into(), 
                        back_order: false, 
                        discontinued: false, 
                        non_diminishing: false, 
                        shippable: true
                    },
                    loyalty_discount: DiscountValue::Absolute(15)
                },
                VariantInformation { 
                    id: "1.83-TROPICS".to_string(),
                    name: "1.83m Tropics".into(), 
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
                    variant_code: vec!["01".into(), "22".into()], 
                    order_history: vec![], 
                    barcode: "54897443288214".into(),
                    stock_information: StockInformation { 
                        stock_group: "RANDOM".into(), 
                        sales_group: "RANDOM".into(), 
                        value_stream: "RANDOM".into(), 
                        brand: "SELLER_GROUP".into(), 
                        unit: "".into(), 
                        tax_code: "GSL".into(), 
                        weight: "5.6".into(), 
                        volume: "0.123".into(), 
                        max_volume: "6.00".into(), 
                        back_order: false, 
                        discontinued: false, 
                        non_diminishing: false,
                        shippable: true
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
            description: "The Nippers Kayak is the ideal size for kid's wanting their own independence.  The compact, lightweight design with a V shaped hull and centre fin allows the kayak to track and manoeuvre easily. They’ll also enjoy the multiple foot holds to accommodate various heights and a lightweight, flexible paddle so they can push through water without too much resistance. With the Nippers Kayak from Torpedo7, there are no excuses for a bad day with mum and dad.".into(), 
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
                        unit: "".into(), 
                        tax_code: "GSL".into(), 
                        weight: "5.6".into(), 
                        volume: "0.123".into(), 
                        max_volume: "6.00".into(), 
                        back_order: false, 
                        discontinued: false, 
                        non_diminishing: false,
                        shippable: true
                    },
                    loyalty_discount: DiscountValue::Absolute(15)
                },
                VariantInformation { 
                    id: "M-RED".to_string(),
                    name: "Medium Red (8-10y)".into(), 
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
                        unit: "".into(), 
                        tax_code: "GSL".into(), 
                        weight: "5.6".into(), 
                        volume: "0.123".into(), 
                        max_volume: "6.00".into(), 
                        back_order: false, 
                        discontinued: false, 
                        non_diminishing: false,
                        shippable: true
                    },
                    loyalty_discount: DiscountValue::Absolute(15)
                },
                VariantInformation { 
                    id: "L-RED".to_string(),
                    name: "Large Red (12-14y)".into(), 
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
                    variant_code: vec!["01".into(), "23".into()], 
                    order_history: vec![], 
                    barcode: "52496265958214".into(),
                    stock_information: StockInformation { 
                        stock_group: "RANDOM".into(), 
                        sales_group: "RANDOM".into(), 
                        value_stream: "RANDOM".into(), 
                        brand: "SELLER_GROUP".into(), 
                        unit: "".into(), 
                        tax_code: "GSL".into(), 
                        weight: "5.6".into(), 
                        volume: "0.123".into(), 
                        max_volume: "6.00".into(), 
                        back_order: false, 
                        discontinued: false, 
                        non_diminishing: false,
                        shippable: true
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