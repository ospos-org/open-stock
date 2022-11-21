use core::fmt;
use std::fmt::{Display};

use chrono::{Utc, DateTime};
use serde::{Serialize, Deserialize};
use sqlx::{Transaction as SqlxTransaction, MySql, mysql::MySqlQueryResult};

use crate::methods::{OrderList, NoteList, HistoryList, Payment, Id};
use uuid::Uuid;

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

#[derive(Debug)]
pub struct Transaction {
    pub id: Uuid,

    pub customer: Id,
    pub transaction_type: TransactionType,

    pub products: OrderList,
    pub order_total: i128,
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
}

impl Display for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Transaction ({}) \nProducts:\t\n", self.id)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum TransactionType {
    In, Out
}

impl TransactionType {
    fn to_string(&self) -> String {
        match self {
            TransactionType::In => "in".to_string(),
            TransactionType::Out => "out".to_string(),
        }
    }
}

// impl! Implement the intent as a builder. 
pub struct Intent {
    request: Transaction,
    // Employee ID for the dispatcher (instigator) for an In-store Purchase (i.e. Tills person) or website deployment ID
    dispatcher: Id,
}