use std::{fmt::Display};

use rand::Rng;
use sea_orm::{DbConn, DbErr, EntityTrait, Set, QuerySelect, ColumnTrait, InsertResult, ActiveModelTrait, Condition, QueryFilter, sea_query::{Expr, Func}};
use serde::{Serialize, Deserialize};
use serde_json::json;

use crate::{methods::{Url, TagList, DiscountValue, Location, ContactInformation, Stock, MobileNumber, Email, Address, Quantity, DiscountMap}, entities::{sea_orm_active_enums::TransactionType, products}};
use super::{VariantCategoryList, VariantIdTag, VariantCategory, Variant, StockInformation, VariantInformation};
use crate::entities::prelude::Products;

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

    pub async fn search(query: &str, db: &DbConn) -> Result<Vec<Product>, DbErr> {
        let res = products::Entity::find()
            .filter(
                Condition::any()
                    .add(Expr::expr(Func::lower(Expr::col(products::Column::Name))).like(format!("%{}%", query)))
                    .add(products::Column::Sku.contains(query))
                    .add(products::Column::Variants.contains(query))
            )
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

    pub async fn fetch_by_name(name: &str, db: &DbConn) -> Result<Vec<Product>, DbErr> {
        let res = products::Entity::find()
            .having(products::Column::Name.contains(name))
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

    pub async fn generate(db: &DbConn) -> Result<Product, DbErr> {
        let product = example_product();

        match Self::insert(product.clone(), db).await {
            Ok(_) => Ok(product),
            Err(e) => Err(e),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductPurchase {
    // Includes variant
    pub product_code: ProductCode,
    pub variant: VariantIdTag,
    pub discount: DiscountMap,

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

fn example_product() -> Product {
    let num = rand::thread_rng().gen_range(0..999999);

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
                        marginal_price: 550, 
                        variant_code: "01".into(), 
                        order_history: vec![], 
                    },
                    Variant { 
                        name: "Black".into(), 
                        images: vec![
                            "https://www.torpedo7.co.nz/images/products/T7TEO23YEAA_zoom---men-s-ecopulse-short-sleeve-explore-graphic-t-shirt-black.jpg?v=845eb9a5288642009c05".into(),
                        ], 
                        marginal_price: 550, 
                        variant_code: "02".into(), 
                        order_history: vec![], 
                    },
                    Variant { 
                        name: "Hot Sauce".into(), 
                        images: vec![
                            "https://www.torpedo7.co.nz/images/products/T7TEO23YDHS_zoom---men-s-ecopulse-short-sleeve-explore-graphic-t-shirts-hot-sauce.jpg?v=845eb9a5288642009c05".into(),
                        ], 
                        marginal_price: 550, 
                        variant_code: "03".into(), 
                        order_history: vec![], 
                    },
                    Variant { 
                        name: "Tourmaline".into(), 
                        images: vec![
                            "https://www.torpedo7.co.nz/images/products/T7TEO23YCJZ_zoom---men-s-ecopulse-short-sleeve-explore-graphic-t-shirt-tourmaline.jpg?v=845eb9a5288642009c05".into(),
                        ], 
                        marginal_price: 550, 
                        variant_code: "04".into(), 
                        order_history: vec![], 
                    },
                    Variant { 
                        name: "Navy Blazer".into(), 
                        images: vec![
                            "https://www.torpedo7.co.nz/images/products/T7TEO23YIWJ_zoom---men-s-ecopulse-short-sleeve-organic-chest-print-t-shirt-navy-blazer.jpg?v=845eb9a5288642009c05".into(),
                        ], 
                        marginal_price: 550, 
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
                        marginal_price: 550, 
                        variant_code: "21".into(), 
                        order_history: vec![], 
                    },
                    Variant { 
                        name: "Medium".into(), 
                        images: vec![
                            "https://www.torpedo7.co.nz/images/products/T7TEO23YIWJ_zoom---men-s-ecopulse-short-sleeve-organic-chest-print-t-shirt-navy-blazer.jpg?v=845eb9a5288642009c05".into(),
                        ], 
                        marginal_price: 550, 
                        variant_code: "22".into(), 
                        order_history: vec![], 
                    },
                    Variant { 
                        name: "Large".into(), 
                        images: vec![
                            "https://www.torpedo7.co.nz/images/products/T7TEO23YIWJ_zoom---men-s-ecopulse-short-sleeve-organic-chest-print-t-shirt-navy-blazer.jpg?v=845eb9a5288642009c05".into(),
                        ], 
                        marginal_price: 550, 
                        variant_code: "23".into(), 
                        order_history: vec![], 
                    },
                    Variant { 
                        name: "Extra Large".into(), 
                        images: vec![
                            "https://www.torpedo7.co.nz/images/products/T7TEO23YIWJ_zoom---men-s-ecopulse-short-sleeve-organic-chest-print-t-shirt-navy-blazer.jpg?v=845eb9a5288642009c05".into()
                        ], 
                        marginal_price: 550, 
                        variant_code: "24".into(), 
                        order_history: vec![], 
                    }
                ] 
            }
        ], 
        variants: vec![
            VariantInformation { 
                name: "Small Black".into(), 
                stock: vec![
                    Stock { 
                        store: Location {
                            code: "001".into(),
                            contact: ContactInformation {
                                name: "Torpedo7".into(),
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
                                    street: "9 Carbine Road".into(),
                                    street2: "".into(),
                                    city: "Auckland".into(),
                                    country: "New Zealand".into(),
                                    po_code: "100".into()
                                }
                            }
                        }, 
                        quantity: Quantity { 
                            quantity_sellable: 0.0, 
                            quantity_on_order: 0.0, 
                            quantity_unsellable: 0.0 
                        }   
                    },
                    Stock { 
                        store: Location {
                            code: "002".into(),
                            contact: ContactInformation {
                                name: "Torpedo7".into(),
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
                                    street: "9 Carbine Road".into(),
                                    street2: "".into(),
                                    city: "Auckland".into(),
                                    country: "New Zealand".into(),
                                    po_code: "100".into()
                                }
                            }
                        }, 
                        quantity: Quantity { 
                            quantity_sellable: 4.0, 
                            quantity_on_order: 2.0, 
                            quantity_unsellable: 2.0 
                        }   
                    }
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
                    brand: "Torq Group".into(), 
                    unit: "".into(), 
                    tax_code: "GSL".into(), 
                    weight: "5.6".into(), 
                    volume: "0.123".into(), 
                    max_volume: "6.00".into(), 
                    back_order: false, 
                    discontinued: false, 
                    non_diminishing: false 
                },
                loyalty_discount: DiscountValue::Absolute(15)
            },
            VariantInformation { 
                name: "Medium Black".into(), 
                stock: vec![
                    Stock { 
                        store: Location {
                            code: "001".into(),
                            contact: ContactInformation {
                                name: "Torpedo7".into(),
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
                                    street: "9 Carbine Road".into(),
                                    street2: "".into(),
                                    city: "Auckland".into(),
                                    country: "New Zealand".into(),
                                    po_code: "100".into()
                                }
                            }
                        }, 
                        quantity: Quantity { 
                            quantity_sellable: 7.0,
                            quantity_unsellable: 2.0,
                            quantity_on_order: 4.0, 
                        }   
                    },
                    Stock { 
                        store: Location {
                            code: "002".into(),
                            contact: ContactInformation {
                                name: "Torpedo7".into(),
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
                                    street: "9 Carbine Road".into(),
                                    street2: "".into(),
                                    city: "Auckland".into(),
                                    country: "New Zealand".into(),
                                    po_code: "100".into()
                                }
                            }
                        }, 
                        quantity: Quantity { 
                            quantity_sellable: 0.0, 
                            quantity_on_order: 1.0, 
                            quantity_unsellable: 0.0 
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
                    brand: "Torq Group".into(), 
                    unit: "".into(), 
                    tax_code: "GSL".into(), 
                    weight: "5.6".into(), 
                    volume: "0.123".into(), 
                    max_volume: "6.00".into(), 
                    back_order: false, 
                    discontinued: false, 
                    non_diminishing: false 
                },
                loyalty_discount: DiscountValue::Absolute(25)
            },
            VariantInformation { 
                name: "Large White".into(), 
                stock: vec![
                    Stock { 
                        store: Location {
                            code: "001".into(),
                            contact: ContactInformation {
                                name: "Torpedo7".into(),
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
                                    street: "9 Carbine Road".into(),
                                    street2: "".into(),
                                    city: "Auckland".into(),
                                    country: "New Zealand".into(),
                                    po_code: "100".into()
                                }
                            }
                        }, 
                        quantity: Quantity { 
                            quantity_sellable: 0.0, 
                            quantity_on_order: 0.0, 
                            quantity_unsellable: 2.0 
                        }   
                    },
                    Stock { 
                        store: Location {
                            code: "002".into(),
                            contact: ContactInformation {
                                name: "Torpedo7".into(),
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
                                    street: "9 Carbine Road".into(),
                                    street2: "".into(),
                                    city: "Auckland".into(),
                                    country: "New Zealand".into(),
                                    po_code: "100".into()
                                }
                            }
                        }, 
                        quantity: Quantity { 
                            quantity_sellable: 3.0, 
                            quantity_on_order: 0.0, 
                            quantity_unsellable: 1.0 
                        }   
                    }
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
                    brand: "Torq Group".into(), 
                    unit: "".into(), 
                    tax_code: "GSL".into(), 
                    weight: "5.6".into(), 
                    volume: "0.123".into(), 
                    max_volume: "6.00".into(), 
                    back_order: false, 
                    discontinued: false, 
                    non_diminishing: false 
                },
                loyalty_discount: DiscountValue::Absolute(5)
            },
        ],
        sku: num.to_string(), 
        images: vec![
            "https://www.torpedo7.co.nz/images/products/T7TEOQR5NDD_zoom---men-s-short-sleeve-explore-graphic-tee-ochre-rose.jpg?v=845eb9a5288642009c05".into()
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
    }
}