use std::fmt::Display;

use crate::{methods::{Name, ContactInformation, MobileNumber, Email, Address, Transaction, example_transaction, convert_addr_to_geo}, entities::{supplier}};
use sea_orm::{DbConn, DbErr, Set, EntityTrait, ColumnTrait, QuerySelect, InsertResult, ActiveModelTrait, RuntimeErr};
use serde::{Serialize, Deserialize};
use serde_json::json;
use uuid::Uuid;
use crate::entities::prelude::Supplier as Suppl;

#[derive(Serialize, Deserialize, Clone)]
pub struct Supplier {
    pub id: String,
    pub name: Name,
    pub contact: ContactInformation,
    pub transaction_history: Vec<Transaction>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SupplierInput {
    pub name: Name,
    pub contact: ContactInformation,
    pub transaction_history: Vec<Transaction>,
}

impl Supplier {
    pub async fn insert(suppl: SupplierInput, db: &DbConn) -> Result<InsertResult<supplier::ActiveModel>, DbErr> {
        let id = Uuid::new_v4().to_string();

        let insert_crud = supplier::ActiveModel {
            id: Set(id),
            name: Set(json!(suppl.name)),
            contact: Set(json!(suppl.contact)),
            transaction_history: Set(json!(suppl.transaction_history)),
        };

        match Suppl::insert(insert_crud).exec(db).await {
            Ok(res) => Ok(res),
            Err(err) => Err(err)
        }
    }

    pub async fn fetch_by_id(id: &str, db: &DbConn) -> Result<Supplier, DbErr> {
        let suppl = Suppl::find_by_id(id.to_string()).one(db).await?;
        let s = suppl.unwrap();

        Ok(Supplier { 
            id: s.id, 
            name: serde_json::from_value::<Name>(s.name).unwrap(), 
            contact: serde_json::from_value::<ContactInformation>(s.contact).unwrap(),
            transaction_history: serde_json::from_value::<Vec<Transaction>>(s.transaction_history).unwrap(),
        })
    }

    pub async fn fetch_by_name(name: &str, db: &DbConn) -> Result<Vec<Supplier>, DbErr> {
        let res = supplier::Entity::find()
            .having(supplier::Column::Name.contains(name))
            .all(db).await?;
            
        let mapped = res.iter().map(|s| 
            Supplier { 
                id: s.id.clone(), 
                name: serde_json::from_value::<Name>(s.name.clone()).unwrap(), 
                contact: serde_json::from_value::<ContactInformation>(s.contact.clone()).unwrap(),
                transaction_history: serde_json::from_value::<Vec<Transaction>>(s.transaction_history.clone()).unwrap(),
            }
        ).collect();

        Ok(mapped)
    }

    pub async fn fetch_by_phone(phone: &str, db: &DbConn) -> Result<Vec<Supplier>, DbErr> {
        let res = supplier::Entity::find()
            .having(supplier::Column::Contact.contains(phone))
            .all(db).await?;
            
        let mapped = res.iter().map(|s| 
            Supplier { 
                id: s.id.clone(), 
                name: serde_json::from_value::<Name>(s.name.clone()).unwrap(), 
                contact: serde_json::from_value::<ContactInformation>(s.contact.clone()).unwrap(),
                transaction_history: serde_json::from_value::<Vec<Transaction>>(s.transaction_history.clone()).unwrap(),
            }
        ).collect();

        Ok(mapped)
    }

    pub async fn fetch_by_addr(addr: &str, db: &DbConn) -> Result<Vec<Supplier>, DbErr> {
        let res = supplier::Entity::find()
            .having(supplier::Column::Contact.contains(addr))
            .all(db).await?;
            
        let mapped = res.iter().map(|s| 
            Supplier { 
                id: s.id.clone(), 
                name: serde_json::from_value::<Name>(s.name.clone()).unwrap(), 
                contact: serde_json::from_value::<ContactInformation>(s.contact.clone()).unwrap(),
                transaction_history: serde_json::from_value::<Vec<Transaction>>(s.transaction_history.clone()).unwrap(),
            }
        ).collect();

        Ok(mapped)
    }

    /// Generate and insert a default customer.
    pub async fn generate(db: &DbConn) -> Result<Supplier, DbErr> {
        let cust = example_supplier();
        // Insert & Fetch Customer
        let r = Supplier::insert(cust, &db).await.unwrap();
        match Supplier::fetch_by_id(&r.last_insert_id, &db).await {
            Ok(cust) => {
                Ok(cust)
            }
            Err(e) => {
                Err(e)
            }
        }
    }

    pub async fn update(suppl: SupplierInput, id: &str, db: &DbConn) -> Result<Supplier, DbErr> {
        let addr = convert_addr_to_geo(&format!("{} {} {} {}", suppl.contact.address.street, suppl.contact.address.street2, suppl.contact.address.po_code, suppl.contact.address.city));

        match addr {
            Ok(ad) => {
                let mut new_contact = suppl.contact;
                new_contact.address = ad;

                supplier::ActiveModel {
                    id: Set(id.to_string()),
                    name: Set(json!(suppl.name)),
                    contact: Set(json!(new_contact)),
                    transaction_history: Set(json!(suppl.transaction_history)),
                }.update(db).await?;
        
                Self::fetch_by_id(id, db).await
            }
            Err(_) => {
                Err(DbErr::Query(RuntimeErr::Internal("Invalid address format".to_string())))
            }
        }
    }
}

impl Display for Supplier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let order_history: String = self.transaction_history.iter()
            .map(|f| 
                format!(
                    "{}: {:?}\n", 
                    f.order_date.format("%d/%m/%Y %H:%M"), 
                    f.transaction_type, 
                )
            ).collect();

        write!(
            f, 
            "{} {} \n{}\n({}) {} {}\n\n[Clock History]\n{}\n
            ", 
            self.name.first, self.name.last,
            self.id, 
            self.contact.mobile.region_code, self.contact.mobile.root, self.contact.email.full,
            order_history,
        )
    }
}

pub fn example_supplier() -> SupplierInput {
    let customer = ContactInformation {
        name: "Carl Kennith".into(),
        mobile: MobileNumber::from("021212120".to_string()),
        email: Email::from("carl@kennith.com".to_string()),
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
    };

    let _t = example_transaction();

    // Transaction {
    //     id: "".into(),
    //     salesperson: "".into(),
    //     ..t
    // };

    SupplierInput {
        name: Name { first: "".into(), middle: "".into(), last: "".into() },
        contact: customer.clone(),
        transaction_history: vec![]
    }
}