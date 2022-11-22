use core::fmt;
use std::fmt::{Display};

use chrono::{Utc, DateTime};
use sea_orm::*;
use serde_json::json;

use crate::{methods::{OrderList, NoteList, HistoryList, Payment, Id}, entities::{transactions, sea_orm_active_enums::TransactionType}};
use sea_orm::{DbConn};
use crate::entities::prelude::Transactions;

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
    pub async fn insert(tsn: Transaction, db: &DbConn) -> Result<(), DbErr> {
        let insert_crud = transactions::ActiveModel {
            id: Set(tsn.id),
            customer: Set(tsn.customer),
            transaction_type: Set(tsn.transaction_type),
            products: Set(json!(tsn.products)),
            order_total: Set(tsn.order_total),
            payment: Set(json!(tsn.payment)),
            order_date: Set(tsn.order_date.naive_utc()),
            order_notes: Set(json!(tsn.order_notes)),
            order_history: Set(json!(tsn.order_history)),
            salesperson: Set(tsn.salesperson),
            till: Set(tsn.till)
        };

        match Transactions::insert(insert_crud).exec(db).await {
            Ok(_) => Ok(()),
            Err(err) => Err(err)
        }
    }

    pub async fn fetch_by_id(id: &str, db: &DbConn) -> Result<Transaction, DbErr> {
        let tsn = Transactions::find_by_id(id.to_string()).one(db).await?;
        let t = tsn.unwrap();

        Ok(Transaction {
            id: t.id,
            customer: t.customer,
            transaction_type: t.transaction_type,
            products: serde_json::from_value::<OrderList>(t.products).unwrap(),
            order_total: t.order_total,
            payment: serde_json::from_value::<Payment>(t.payment).unwrap(),
            order_date: DateTime::from_utc(t.order_date, Utc),
            order_notes: serde_json::from_value::<NoteList>(t.order_notes).unwrap(),
            order_history: serde_json::from_value::<HistoryList>(t.order_history).unwrap(),
            salesperson: t.salesperson,
            till: t.till,
        })
    }
}

impl Display for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let products: String = self.products.iter()
            .map(|f| {
                let pdts: String = f.products
                    .iter()
                    .map(|p| 
                        format!(
                            "\t{}: ${} ({}:{})  [-]{}\n", 
                            p.quantity, 
                            p.product_cost, 
                            p.product_code, 
                            p.variant.concat(), 
                            p.discount.to_string()
                        )
                    ).collect();

                let notes: String = f.order_notes
                    .iter()
                    .map(|p| 
                        format!(
                            "{}", p, 
                        )
                    ).collect();

                format!(
                    "-\t{} {} {} -> {} {} [-]{} \n{}\n\t{}\n", 
                    f.reference, f.status, f.origin.code, f.destination.code, f.creation_date.format("%d/%m/%Y %H:%M"), f.discount.to_string(), pdts, notes
                )
            }).collect();

        let notes: String = self.order_notes
            .iter()
            .map(|p| 
                format!(
                    "{}", p
                )
            ).collect();

        let order_history: String = self.order_history.iter()
            .map(|f| {
                format!(
                    "{}: {}\n", 
                    f.timestamp.format("%d/%m/%Y %H:%M"), 
                    f.item,
                )
            }).collect();

        write!(f, "Transaction ({}) {}\nOrders:\n{}\n---\nTotal: ${}\nPayment: {}\nNotes:\n{}\nHistory:\n{}\n{} on {}", self.id, self.order_date.format("%d/%m/%Y %H:%M"), products, self.order_total, self.payment, notes, order_history, self.salesperson, self.till)
    }
}

// impl! Implement the intent as a builder. 
pub struct Intent {
    request: Transaction,
    // Employee ID for the dispatcher (instigator) for an In-store Purchase (i.e. Tills person) or website deployment ID
    dispatcher: Id,
}