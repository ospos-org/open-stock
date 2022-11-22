use crate::{methods::{Name, ContactInformation, OrderList, NoteList, Id}, entities::customer};
use sea_orm::{DbConn, DbErr, Set, EntityTrait};
use serde_json::json;
use crate::entities::prelude::Customer as Cust;

pub struct Customer {
    pub id: Id,
    pub name: Name,
    pub contact: ContactInformation,
    pub order_history: OrderList,
    pub customer_notes: NoteList,
    pub balance: i32,
}

impl Customer {
    pub async fn insert(cust: Customer, db: &DbConn) -> Result<(), DbErr> {
        let insert_crud = customer::ActiveModel {
            id: Set(cust.id),
            name: Set(json!(cust.name)),
            contact: Set(json!(cust.contact)),
            order_history: Set(json!(cust.order_history)),
            customer_notes: Set(json!(cust.customer_notes)),
            balance: Set(cust.balance),
        };

        match Cust::insert(insert_crud).exec(db).await {
            Ok(_) => Ok(()),
            Err(err) => Err(err)
        }
    }

    pub async fn fetch_transaction_by_id(id: &str, db: &DbConn) -> Result<Customer, DbErr> {
        let cust = Cust::find_by_id(id.to_string()).one(db).await?;
        let c = cust.unwrap();

        Ok(Customer { 
            id: c.id, 
            name: serde_json::from_value::<Name>(c.name).unwrap(), 
            contact: serde_json::from_value::<ContactInformation>(c.contact).unwrap(),
            order_history: serde_json::from_value::<OrderList>(c.order_history).unwrap(),
            customer_notes: serde_json::from_value::<NoteList>(c.customer_notes).unwrap(),
            balance: c.balance, 
        })
    }
}