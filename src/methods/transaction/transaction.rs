use core::fmt;
use std::fmt::{Display};

use chrono::{Utc, DateTime};
use serde::{Serialize, Deserialize};
use sqlx::{Transaction as SqlxTransaction, MySql, mysql::{MySqlQueryResult}, FromRow};

use crate::methods::{OrderList, NoteList, HistoryList, Payment, Id, Order};

// Discounts on the transaction are applied per-order - such that they are unique to each item, i.e. each item can be discounted individually where needed to close a sale.
// A discount placed upon the payment object is an order-discount, such that it will act upon the basket: 

/*
    -- Transaction --
    An order group is parented by a transaction, this can include 1 or more orders. 
    It is attached to a customer, and represents the transaction for the purchase or sale of goods.

    The products attribute: An order list which is often comprised of 1 order.
    -   Why would there be more than 1 order in a transaction?
            If a consumer purchases multiple goods which need to be dealt with separately, the transaction will do so, An example might be:
            A surfboard which is shipped to the consumer whilst 3 accessories are taken from the shop directly, thus two orders (1 shipment and 1 direct),
            whereby the 2nd order will contain multiple (3) products and the 1st only one.

    IN:     As a purchase order it's transaction type takes the form of "In", the customer object will be treated as the company bought from and the payment as an outward payment in exchange for the goods.
    OUT:    A sale - It can occur in-store or online and is comprised of the sale of goods outlined in the order list.
*/

#[derive(Debug, FromRow)]
pub struct Transaction {
    pub id: Id,

    pub customer: Id,
    pub transaction_type: TransactionType,

    pub products: OrderList,
    pub order_total: i32,
    pub payment: Payment,

    pub order_date: DateTime<Utc>,
    pub order_notes: NoteList,
    pub order_history: HistoryList,

    pub salesperson: Id,
    pub till: Id,
}

impl Transaction {
    pub async fn insert_transaction(tsn: Transaction, mut conn: SqlxTransaction<'_, MySql>) -> Result<MySqlQueryResult, sqlx::Error> {
        match sqlx::query!(
            "INSERT INTO Transactions (id, customer, transaction_type, products, order_total, payment, order_date, order_notes, order_history, salesperson, till) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)", 
            tsn.id.to_string(), 
            tsn.customer, 
            tsn.transaction_type.to_string(), 
            serde_json::to_string(&tsn.products).unwrap(), 
            tsn.order_total.to_string(),
            serde_json::to_string(&tsn.payment).unwrap(),
            tsn.order_date.to_rfc3339(),
            serde_json::to_string(&tsn.order_notes).unwrap(),
            serde_json::to_string(&tsn.order_history).unwrap(), 
            tsn.salesperson,
            tsn.till
        ).execute(&mut conn).await {
            Ok(res) => {
                match conn.commit().await {
                    Ok(_) => Ok(res),
                    Err(error) => Err(error)
                }
            },
            Err(error) => Err(error)
        }
    }

    // pub async fn fetch_all_transactions(mut conn: SqlxTransaction<'_, MySql>) -> Result<sqlx::Result<Transaction>, sqlx::Error> {
    //     match sqlx::query_as!(Transaction, "SELECT * FROM Transactions LIMIT 50").fetch_all(&mut conn).await {
    //         Ok(res) => {
    //             match conn.commit().await {
    //                 Ok(_) => Ok(res),
    //                 Err(e) => Err(e)
    //             }
    //         }
    //         Err(error) => Err(error)
    //     }
    // }

    pub async fn fetch_transaction_by_id(id: &str, mut conn: SqlxTransaction<'_, MySql>) -> Result<Transaction, sqlx::Error> {
        match sqlx::query_as!(
            Transaction,
            r#"SELECT id, customer, transaction_type as "transaction_type: TransactionType", products as "products: Vec<Order>", order_total, payment as "payment: Payment", order_date as "order_date: DateTime<Utc>", order_notes as "order_notes: NoteList", order_history as "order_history: HistoryList", salesperson, till FROM Transactions WHERE id = ?"#,
            id
        ).fetch_one(&mut conn).await {
            Ok(res) => {
                Ok(res)
            },
            Err(err) => Err(err),
        }
    }
}

impl Display for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Transaction ({}) \nProducts:\t\n", self.id)
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, sqlx::Type)]
pub enum TransactionType {
    In, Out
}

impl fmt::Display for TransactionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

// impl Type<MySql> for TransactionType {
//     fn compatible(ty: &<MySql as Database>::TypeInfo) -> bool {
//         *ty == Self::type_info()
//     }

//     fn type_info() -> MySqlTypeInfo {
//         MySqlTypeInfo::from(MySqlTypeInfo::name("json"))
//     }
// }

// impl TypeInfo for TransactionType {
//     fn is_null(&self) -> bool {
//         false
//     }

//     fn name(&self) -> &str {
//         "json"
//     }

//     fn is_void(&self) -> bool {
//         false
//     }
// }

// impl<'r> Decode<'r, MySql> for TransactionType {
//     fn decode(
//         value: <MySql as HasValueRef<'r>>::ValueRef,
//     ) -> std::result::Result<TransactionType, Box<(dyn std::error::Error + Send + Sync + 'static)>> {
//         let value = <&str as Decode<MySql>>::decode(value)?;

//         match value {
//             "in" | "IN" => Ok(TransactionType::In),
//             "out" | "OUT" => Ok(TransactionType::Out),
//             _ => Err(Box::new(sqlx::Error::Decode(Box::new(serde_json::Error::invalid_value(Unexpected::Str(&format!("Unexpected Value: {}", value)), &"IN or OUT")))))
//         }
//     }
// }

// impl! Implement the intent as a builder. 
pub struct Intent {
    request: Transaction,
    // Employee ID for the dispatcher (instigator) for an In-store Purchase (i.e. Tills person) or website deployment ID
    dispatcher: Id,
}