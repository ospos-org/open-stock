use std::{str::FromStr, fmt::Display};

use sea_orm::{DbConn, DbErr, EntityTrait, Set, QuerySelect, ColumnTrait, InsertResult, ActiveModelTrait};
use serde::{Serialize, Deserialize};
use serde_json::json;

use crate::{methods::{Url, TagList, DiscountValue, Location, ContactInformation, Stock, MobileNumber, Email, Address, Quantity}, entities::{sea_orm_active_enums::TransactionType, products}};
use super::{VariantCategoryList, VariantIdTag, VariantCategory, Variant, StockInformation};
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

    pub async fn update(pdt: Product, id: &str, db: &DbConn) -> Result<Product, DbErr> {
        products::ActiveModel {
            sku: Set(pdt.sku),
            name: Set(pdt.name),
            company: Set(pdt.company),
            variants: Set(json!(pdt.variants)),
            loyalty_discount: Set(DiscountValue::to_string(&pdt.loyalty_discount)),
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
            TransactionType::PendingIn => "PENDING-IN",
            TransactionType::PendingOut => "PENDING-OUT"
        };

        write!(f, "{}: {}-{} x{}", method, self.product_code, self.variant.concat(), self.quantity)
    }
}

pub type ProductCode = String;
pub type ProductPurchaseList = Vec<ProductPurchase>;

fn example_product() -> Product {
    Product { 
        name: "Wakeboard".into(), 
        company: "Torq".into(), 
        variants: vec![
            VariantCategory { 
                category: "Colour".into(), 
                variants: vec![
                    Variant { 
                        name: "White".into(), 
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
                                    quantity_on_hand: 2, 
                                    quantity_on_order: 1, 
                                    quantity_on_floor: 1 
                                }   
                            }
                        ], 
                        images: vec![
                            "https://www.torpedo7.co.nz/images/products/F1S8CN8VAXX_zoom---surfboard-7ft-6in-fun-white.jpg?v=075e26aa5b6847e8bbd2".into(),
                            "https://www.torpedo7.co.nz/images/products/F1S8CN8VAXX_zoom_1---surfboard-7ft-6in-fun-white.jpg?v=075e26aa5b6847e8bbd2".into(),
                            "https://www.torpedo7.co.nz/images/products/F1S8CN8VAXX_zoom_2---surfboard-7ft-6in-fun-white.jpg?v=075e26aa5b6847e8bbd2".into()
                        ], 
                        marginal_price: 550, 
                        variant_code: "01".into(), 
                        order_history: vec![], 
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
                        }
                    }
                ] 
            }
        ], 
        sku: "123858".into(), 
        loyalty_discount: DiscountValue::Absolute(15), 
        images: vec![
            "https://www.torpedo7.co.nz/images/products/F1S8CN8VAXX_zoom---surfboard-7ft-6in-fun-white.jpg?v=075e26aa5b6847e8bbd2".into(),
            "https://www.torpedo7.co.nz/images/products/F1S8CN8VAXX_zoom_1---surfboard-7ft-6in-fun-white.jpg?v=075e26aa5b6847e8bbd2".into(),
            "https://www.torpedo7.co.nz/images/products/F1S8CN8VAXX_zoom_2---surfboard-7ft-6in-fun-white.jpg?v=075e26aa5b6847e8bbd2".into()
        ], 
        tags: vec![
            "Surfboard".into(),
            "Water".into()
        ], 
        description: "This crossover range caters to all levels of surfers in virtually every condition. From waist high mush to overhead and hollow. The versatility of the Mod Fun makes them an excellent choice if you need one board to handle all the conditions where you live and travel.\n        Featuring a medium full nose and shallow mid-entry there is enough volume for smaller days and weaker surf. As the surf jumps up, step back and the board transforms. You'll find a board that feels shorter than it's length, delivering predictable handling and performance.  Tri-fin set-up.\n        Our fin system is designed by Futures Fins of California - one of the most respected fin systems on the planet.\n        Torq TET surfboards all come with fins. The ModFun shapes come with 3 fin boxes and a Thruster fin \n        set offering an even balance of drive and release for all round surfing.".into(), 
        specifications: vec![
            ("Difficulty".into(), "Expert".into()),
            ("Wave Height".into(), "2-6ft".into()),
            ("Dimensions".into(), "7'6\" x 21 1/2\" x 2 7/8\"".into())
        ] 
    }
}