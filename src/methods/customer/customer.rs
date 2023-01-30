use std::fmt::Display;

use crate::{methods::{ContactInformation, OrderList, NoteList, Id, MobileNumber, Email, Address, Order, Location, ProductPurchase, DiscountValue, OrderStatus, Note, TransitInformation, OrderStatusAssignment, convert_addr_to_geo, History}, entities::customer};
use chrono::Utc;
use sea_orm::{DbConn, DbErr, Set, EntityTrait, ColumnTrait, QuerySelect, InsertResult, ActiveModelTrait, sea_query::{Func, Expr}, QueryFilter, Condition, RuntimeErr};
use serde::{Serialize, Deserialize};
use serde_json::json;
use uuid::Uuid;
use crate::entities::prelude::Customer as Cust;

#[derive(Serialize, Deserialize, Clone)]
pub struct Customer {
    pub id: Id,
    pub name: String,
    pub contact: ContactInformation,
    pub order_history: OrderList,
    pub customer_notes: NoteList,
    pub balance: f32,
    pub special_pricing: String
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CustomerInput {
    pub name: String,
    pub contact: ContactInformation,
    pub order_history: OrderList,
    pub customer_notes: NoteList,
    pub special_pricing: String,
    pub balance: f32,
}

impl Customer {
    pub async fn insert(cust: CustomerInput, db: &DbConn) -> Result<InsertResult<customer::ActiveModel>, DbErr> {
        let id = Uuid::new_v4().to_string();

        let insert_crud = customer::ActiveModel {
            id: Set(id),
            name: Set(cust.name),
            contact: Set(json!(cust.contact)),
            order_history: Set(json!(cust.order_history)),
            customer_notes: Set(json!(cust.customer_notes)),
            balance: Set(cust.balance),
            special_pricing: Set(json!(cust.special_pricing)),
        };

        match Cust::insert(insert_crud).exec(db).await {
            Ok(res) => Ok(res),
            Err(err) => Err(err)
        }
    }

    pub async fn fetch_by_id(id: &str, db: &DbConn) -> Result<Customer, DbErr> {
        let cust = Cust::find_by_id(id.to_string()).one(db).await?;
        let c = cust.unwrap();

        Ok(Customer { 
            id: c.id, 
            name: c.name, 
            contact: serde_json::from_value::<ContactInformation>(c.contact).unwrap(),
            order_history: serde_json::from_value::<OrderList>(c.order_history).unwrap(),
            customer_notes: serde_json::from_value::<NoteList>(c.customer_notes).unwrap(),
            special_pricing: serde_json::from_value::<String>(c.special_pricing).unwrap(),
            balance: c.balance, 
        })
    }

    pub async fn search(query: &str, db: &DbConn) -> Result<Vec<Customer>, DbErr> {
        let res = customer::Entity::find()
            .filter(
                Condition::any()
                    .add(Expr::expr(Func::lower(Expr::col(customer::Column::Name))).like(format!("%{}%", query)))
                    // Phone no. and email.
                    .add(customer::Column::Contact.contains(query))
            )
            .all(db).await?;

        let mapped = res.iter().map(|c| 
            Customer { 
                id: c.id.clone(), 
                name: c.name.clone(), 
                contact: serde_json::from_value::<ContactInformation>(c.contact.clone()).unwrap(),
                order_history: serde_json::from_value::<OrderList>(c.order_history.clone()).unwrap(),
                customer_notes: serde_json::from_value::<NoteList>(c.customer_notes.clone()).unwrap(),
                special_pricing: serde_json::from_value::<String>(c.special_pricing.clone()).unwrap(),
                balance: c.balance
            }
        ).collect();

        Ok(mapped)
    }

    pub async fn fetch_by_name(name: &str, db: &DbConn) -> Result<Vec<Customer>, DbErr> {
        let res = customer::Entity::find()
            .having(Expr::expr(Func::lower(Expr::col(customer::Column::Name))).like(format!("%{}%", name)))
            .all(db).await?;
            
        let mapped = res.iter().map(|c| 
            Customer { 
                id: c.id.clone(), 
                name: c.name.clone(), 
                contact: serde_json::from_value::<ContactInformation>(c.contact.clone()).unwrap(),
                order_history: serde_json::from_value::<OrderList>(c.order_history.clone()).unwrap(),
                customer_notes: serde_json::from_value::<NoteList>(c.customer_notes.clone()).unwrap(),
                special_pricing: serde_json::from_value::<String>(c.special_pricing.clone()).unwrap(),
                balance: c.balance
            }
        ).collect();

        Ok(mapped)
    }

    pub async fn fetch_by_phone(phone: &str, db: &DbConn) -> Result<Vec<Customer>, DbErr> {
        let res = customer::Entity::find()
            .having(customer::Column::Contact.contains(phone))
            .all(db).await?;
            
        let mapped = res.iter().map(|c| 
            Customer { 
                id: c.id.clone(), 
                name: c.name.clone(), 
                contact: serde_json::from_value::<ContactInformation>(c.contact.clone()).unwrap(),
                order_history: serde_json::from_value::<OrderList>(c.order_history.clone()).unwrap(),
                customer_notes: serde_json::from_value::<NoteList>(c.customer_notes.clone()).unwrap(),
                special_pricing: serde_json::from_value::<String>(c.special_pricing.clone()).unwrap(),
                balance: c.balance
            }
        ).collect();

        Ok(mapped)
    }

    pub async fn fetch_by_addr(addr: &str, db: &DbConn) -> Result<Vec<Customer>, DbErr> {
        let res = customer::Entity::find()
            .having(customer::Column::Contact.contains(addr))
            .all(db).await?;
            
        let mapped = res.iter().map(|c| 
            Customer { 
                id: c.id.clone(), 
                name: c.name.clone(), 
                contact: serde_json::from_value::<ContactInformation>(c.contact.clone()).unwrap(),
                order_history: serde_json::from_value::<OrderList>(c.order_history.clone()).unwrap(),
                customer_notes: serde_json::from_value::<NoteList>(c.customer_notes.clone()).unwrap(),
                special_pricing: serde_json::from_value::<String>(c.special_pricing.clone()).unwrap(),
                balance: c.balance
            }
        ).collect();

        Ok(mapped)
    }

    /// Generate and insert a default customer.
    pub async fn generate(db: &DbConn) -> Result<Customer, DbErr> {
        let cust = example_customer();
        // Insert & Fetch Customer
        let r = Customer::insert(cust, &db).await.unwrap();
        match Customer::fetch_by_id(&r.last_insert_id, &db).await {
            Ok(cust) => {
                Ok(cust)
            }
            Err(e) => {
                Err(e)
            }
        }
    }

    pub async fn update(cust: CustomerInput, id: &str, db: &DbConn) -> Result<Customer, DbErr> {
        let addr = convert_addr_to_geo(&format!("{} {} {} {}", cust.contact.address.street, cust.contact.address.street2, cust.contact.address.po_code, cust.contact.address.city));

        // !impl Validate form input/ contact information.

        match addr {
            Ok(ad) => {
                let mut new_contact = cust.contact;
                new_contact.address = ad;

                customer::ActiveModel {
                    id: Set(id.to_string()),
                    name: Set(cust.name),
                    contact: Set(json!(new_contact)),
                    order_history: Set(json!(cust.order_history)),
                    customer_notes: Set(json!(cust.customer_notes)),
                    special_pricing: Set(json!(cust.special_pricing)),
                    balance: Set(cust.balance),
                }.update(db).await?;
        
                Self::fetch_by_id(id, db).await
            }
            Err(_) => {
                Err(DbErr::Query(RuntimeErr::Internal("Invalid address format".to_string())))
            }
        }
    }

    pub async fn update_contact_information(contact: ContactInformation, id: &str, db: &DbConn) -> Result<Customer, DbErr> {
        let customer = Self::fetch_by_id(id, db).await?;
        // Get geo location for new contact information...
        let addr = convert_addr_to_geo(&format!("{} {} {} {}", contact.address.street, contact.address.street2, contact.address.po_code, contact.address.city));

        match addr {
            Ok(ad) => {
                let mut new_contact = contact;
                new_contact.address = ad;

                customer::ActiveModel {
                    id: Set(customer.id),
                    name: Set(customer.name),
                    contact: Set(json!(new_contact)),
                    order_history: Set(json!(customer.order_history)),
                    customer_notes: Set(json!(customer.customer_notes)),
                    special_pricing: Set(json!(customer.special_pricing)),
                    balance: Set(customer.balance),
                }.update(db).await?;
        
                Self::fetch_by_id(id, db).await 
            }
            Err(_) => {
                Err(DbErr::Query(RuntimeErr::Internal("Invalid address format".to_string())))
            }
        }
        
    }
}

impl Display for Customer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let order_history: String = self.order_history.iter()
            .map(|f| 
                format!(
                    "{}: {:?}\n", 
                    f.creation_date.format("%d/%m/%Y %H:%M"), 
                    f.status, 
                )
            ).collect();

        let customer_notes: String = self.customer_notes.iter()
            .map(|f| 
                format!(
                    "{}: {}\n", 
                    f.timestamp.format("%d/%m/%Y %H:%M"), 
                    f.message, 
                )
            ).collect();

        write!(
            f, 
            "{} (${})\n{}\n({}) {} {}\n\n[Clock History]\n{}\n[Notes]\n{}
            ", 
            self.name, self.balance, 
            self.id, 
            self.contact.mobile.region_code, self.contact.mobile.root, self.contact.email.full,
            order_history,
            customer_notes
        )
    }
}

pub fn example_customer() -> CustomerInput {
    let customer = ContactInformation {
        name: "Carl Kennith".into(),
        mobile: MobileNumber::from("0212121204".to_string()),
        email: Email::from("carl@kennith.com".to_string()),
        landline: "".into(),
        address: Address {
            street: "54 Arney Crescent".into(),
            street2: "Remuera".into(),
            city: "Auckland".into(),
            country: "New Zealand".into(),
            po_code: "1050".into(),
            lat: -36.869870,
            lon: 174.790520
        },
    };

    CustomerInput {
        name: "Carl Kennith".into(),
        contact: customer.clone(),
        order_history: vec![
            Order {
                order_type: crate::methods::OrderType::Pickup,
                destination: Location {
                    code: "001".into(),
                    contact: customer.clone()
                },
                origin: Location {
                    code: "002".into(),
                    contact: customer.clone()
                },
                products: vec![
                    ProductPurchase { id: "ANY".to_string(), product_code:"132522".into(), discount: vec![], product_cost: 15.00, variant: vec!["22".into()], quantity: 5 },
                    ProductPurchase { id: "ANY".to_string(), product_code:"132522".into(), discount: vec![], product_cost: 15.00, variant: vec!["23".into()], quantity: 5 }
                ],
                previous_failed_fulfillment_attempts: vec![],
                status: OrderStatusAssignment {
                    status: OrderStatus::Transit(
                        TransitInformation {
                            shipping_company: customer.clone(),
                            query_url: "https://www.fedex.com/fedextrack/?trknbr=".into(),
                            tracking_code: "1523123".into(),
                            assigned_products: vec![]
                        }
                    ),
                    assigned_products: vec![],
                    timestamp: Utc::now()
                },
                order_history: vec![],
                order_notes: vec![
                    Note {
                        message: "Order shipped from warehouse.".into(), 
                        timestamp: Utc::now(), 
                        author: Uuid::new_v4().to_string()
                    }
                ],
                reference: "TOR-19592".into(),
                creation_date: Utc::now(),
                id: Uuid::new_v4().to_string(),
                status_history: vec![ History::<OrderStatusAssignment> { item: OrderStatusAssignment { status: OrderStatus::Queued(Utc::now()), timestamp: Utc::now(), assigned_products: vec![] }, timestamp: Utc::now(), reason: "".to_string() }],
                discount: DiscountValue::Absolute(0),
            }
        ],
        special_pricing: "".into(),
        customer_notes: vec![],
        balance: 0.0,
    }
}