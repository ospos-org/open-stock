use chrono::{Utc, DateTime};

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

#[derive(Debug)]
pub enum TransactionType {
    In, Out
}

// impl! Implement the intent as a builder. 
pub struct Intent {
    request: Transaction,
    // Employee ID for the dispatcher (instigator) for an In-store Purchase (i.e. Tills person) or website deployment ID
    dispatcher: Id,
}