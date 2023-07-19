use core::fmt;
use std::fmt::Display;

use chrono::{DateTime, Days, Duration, NaiveDateTime, Utc};
use sea_orm::{
    sea_query::{Expr, Func},
    *,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::task::JoinError;
use uuid::Uuid;

use sea_orm::FromQueryResult;

use crate::entities::prelude::Transactions;
use crate::{
    entities::{sea_orm_active_enums::TransactionType, transactions},
    methods::{
        Address, ContactInformation, DiscountValue, Email, History, Id, Location, MobileNumber,
        Note, NoteList, Order, OrderList, OrderStatus, OrderStatusAssignment, Payment,
        PaymentAction, PaymentProcessor, PaymentStatus, Price, Product, ProductPurchase, Session,
        Stock, TransitInformation, VariantInformation,
    },
    PickStatus, ProductInstance,
};
use sea_orm::DbConn;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TransactionCustomer {
    pub customer_type: CustomerType,
    pub customer_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum CustomerType {
    Store,
    Individual,
    Commercial,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct QuantityAlterationIntent {
    pub variant_code: String,
    pub product_sku: String,
    pub transaction_store_code: String,
    pub transaction_store_id: String,
    pub transaction_type: TransactionType,
    pub quantity_to_transact: f32,
}

// Discounts on the transaction are applied per-order - such that they are unique to each item, i.e. each item can be discounted individually where needed to close a sale.
// A discount placed upon the payment object is an order-discount, such that it will act upon the basket:

/// **Transaction** <br />
/// An order group is parented by a transaction, this can include 1 or more orders.
/// It is attached to a customer, and represents the transaction for the purchase or sale of goods. <br />
///
/// The products attribute: An order list which is often comprised of 1 order.
/// -   Why would there be more than 1 order in a transaction?
///     - If a consumer purchases multiple goods which need to be dealt with separately, the transaction will do so, An example might be: A surfboard which is shipped to the consumer whilst 3 accessories are taken from the shop directly, thus two orders (1 shipment and 1 direct), whereby the 2nd order will contain multiple (3) products and the 1st only one.
///
/// `IN:`     As a purchase order it's transaction type takes the form of "In", the customer object will be treated as the company bought from and the payment as an outward payment in exchange for the goods. <br />
/// `OUT:`    A sale - It can occur in-store or online and is comprised of the sale of goods outlined in the order list.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Transaction {
    pub id: Id,

    pub customer: TransactionCustomer,
    pub transaction_type: TransactionType,

    pub products: OrderList,
    pub order_total: f32,
    pub payment: Vec<Payment>,

    pub order_date: DateTime<Utc>,
    pub order_notes: NoteList,

    pub salesperson: Id,
    pub till: Id,
}

#[derive(Serialize, Deserialize, Clone, FromQueryResult)]
pub struct DerivableTransaction {
    pub id: Id,

    pub customer: JsonValue,
    pub transaction_type: TransactionType,

    pub products: JsonValue,
    pub order_total: f32,
    pub payment: JsonValue,

    pub order_date: NaiveDateTime,
    pub order_notes: JsonValue,

    pub salesperson: Id,
    pub till: Id,
}

#[derive(Deserialize, Clone)]
pub struct TransactionInput {
    pub customer: TransactionCustomer,
    pub transaction_type: TransactionType,

    pub products: OrderList,
    pub order_total: f32,
    pub payment: Vec<Payment>,

    pub order_date: DateTime<Utc>,
    pub order_notes: NoteList,

    pub salesperson: Id,
    pub till: Id,
}

#[derive(Deserialize, Clone)]
pub struct TransactionInit {
    pub customer: TransactionCustomer,
    pub transaction_type: TransactionType,

    pub products: OrderList,
    pub order_total: f32,
    pub payment: Vec<Payment>,

    pub order_date: DateTime<Utc>,
    pub order_notes: NoteList,

    pub till: Id,
}

impl Transaction {
    pub async fn insert(
        tsn: TransactionInit,
        session: Session,
        db: &DbConn,
    ) -> Result<InsertResult<transactions::ActiveModel>, DbErr> {
        let id = Uuid::new_v4().to_string();

        let insert_crud = transactions::ActiveModel {
            id: Set(id),
            customer: Set(json!(tsn.customer)),
            transaction_type: Set(tsn.transaction_type),
            products: Set(json!(tsn.products)),
            order_total: Set(tsn.order_total),
            payment: Set(json!(tsn.payment)),
            order_date: Set(tsn.order_date.naive_utc()),
            order_notes: Set(json!(tsn.order_notes)),
            salesperson: Set(session.employee.id),
            till: Set(tsn.till),
        };

        match Transactions::insert(insert_crud).exec(db).await {
            Ok(res) => Ok(res),
            Err(err) => Err(err),
        }
    }

    pub async fn fetch_deliverable_jobs(query: &str, db: &DbConn) -> Result<Vec<Order>, DbErr> {
        let as_str: Vec<DerivableTransaction> =
            DerivableTransaction::find_by_statement(Statement::from_sql_and_values(
                DbBackend::MySql,
                &format!(
                    "SELECT * FROM Transactions WHERE Transactions.products LIKE '%{}%'",
                    query
                ),
                vec![],
            ))
            .all(db)
            .await?;

        let mapped = as_str
            .iter()
            .flat_map(|t| {
                // Conditions are:
                // 1. Must be distributed from the query location
                // 2. Must be an actively queued job
                let products = serde_json::from_value::<OrderList>(t.products.clone()).unwrap();
                let orders = products
                    .iter()
                    .filter(|o| o.origin.store_id == query && o.status.status.is_queued());

                orders.cloned().collect::<Vec<Order>>()
            })
            .collect();

        Ok(mapped)
    }

    pub async fn fetch_receivable_jobs(query: &str, db: &DbConn) -> Result<Vec<Order>, DbErr> {
        let as_str: Vec<DerivableTransaction> =
            DerivableTransaction::find_by_statement(Statement::from_sql_and_values(
                DbBackend::MySql,
                &format!(
                    "SELECT * FROM Transactions WHERE Transactions.products LIKE '%{}%'",
                    query
                ),
                vec![],
            ))
            .all(db)
            .await?;

        let mapped = as_str
            .iter()
            .flat_map(|t| {
                // Conditions are:
                // 1. Must be distributed from the query location
                // 2. Must be an actively queued job
                let products = serde_json::from_value::<OrderList>(t.products.clone()).unwrap();
                let orders = products.iter().filter(|o| {
                    o.destination.store_id == query && o.status.status.is_not_fulfilled_nor_failed()
                });

                orders.cloned().collect::<Vec<Order>>()
            })
            .collect();

        Ok(mapped)
    }

    pub async fn fetch_by_id(id: &str, db: &DbConn) -> Result<Transaction, DbErr> {
        let tsn = Transactions::find_by_id(id.to_string()).one(db).await?;
        let t = tsn.unwrap();

        let t = Transaction {
            id: t.id,
            customer: serde_json::from_value::<TransactionCustomer>(t.customer).unwrap(),
            transaction_type: t.transaction_type,
            products: serde_json::from_value::<OrderList>(t.products).unwrap(),
            order_total: t.order_total,
            payment: serde_json::from_value::<Vec<Payment>>(t.payment).unwrap(),
            order_date: DateTime::from_utc(t.order_date, Utc),
            order_notes: serde_json::from_value::<NoteList>(t.order_notes).unwrap(),
            salesperson: t.salesperson,
            till: t.till,
        };

        Ok(t)
    }

    pub async fn fetch_all_saved(db: &DbConn) -> Result<Vec<Transaction>, DbErr> {
        let res = Transactions::find()
            .having(
                Expr::expr(Func::lower(Expr::col(
                    transactions::Column::TransactionType,
                )))
                .like("%saved%".to_string()),
            )
            .limit(25)
            .all(db)
            .await?;

        let mapped = res
            .iter()
            .map(|t| Transaction {
                id: t.id.clone(),
                customer: serde_json::from_value::<TransactionCustomer>(t.customer.clone())
                    .unwrap(),
                transaction_type: t.transaction_type.clone(),
                products: serde_json::from_value::<OrderList>(t.products.clone()).unwrap(),
                order_total: t.order_total,
                payment: serde_json::from_value::<Vec<Payment>>(t.payment.clone()).unwrap(),
                order_date: DateTime::from_utc(t.order_date, Utc),
                order_notes: serde_json::from_value::<NoteList>(t.order_notes.clone()).unwrap(),
                salesperson: t.salesperson.clone(),
                till: t.till.clone(),
            })
            .collect();

        Ok(mapped)
    }

    pub async fn fetch_by_ref(reference: &str, db: &DbConn) -> Result<Vec<Transaction>, DbErr> {
        let res = Transactions::find()
            .having(
                Expr::expr(Func::lower(Expr::col(transactions::Column::Products)))
                    .like(format!("%{}%", reference.to_lowercase())),
            )
            .having(transactions::Column::TransactionType.not_like("Saved"))
            .limit(25)
            .all(db)
            .await?;

        let mapped = res
            .iter()
            .map(|t| Transaction {
                id: t.id.clone(),
                customer: serde_json::from_value::<TransactionCustomer>(t.customer.clone())
                    .unwrap(),
                transaction_type: t.transaction_type.clone(),
                products: serde_json::from_value::<OrderList>(t.products.clone()).unwrap(),
                order_total: t.order_total,
                payment: serde_json::from_value::<Vec<Payment>>(t.payment.clone()).unwrap(),
                order_date: DateTime::from_utc(t.order_date, Utc),
                order_notes: serde_json::from_value::<NoteList>(t.order_notes.clone()).unwrap(),
                salesperson: t.salesperson.clone(),
                till: t.till.clone(),
            })
            .collect();

        Ok(mapped)
    }

    pub async fn fetch_by_client_id(id: &str, db: &DbConn) -> Result<Vec<Transaction>, DbErr> {
        let tsn = Transactions::find()
            .having(transactions::Column::Customer.contains(id))
            .all(db)
            .await?;

        let mapped = tsn
            .iter()
            .map(|t| Transaction {
                id: t.id.clone(),
                customer: serde_json::from_value::<TransactionCustomer>(t.customer.clone())
                    .unwrap(),
                transaction_type: t.transaction_type.clone(),
                products: serde_json::from_value::<OrderList>(t.products.clone()).unwrap(),
                order_total: t.order_total,
                payment: serde_json::from_value::<Vec<Payment>>(t.payment.clone()).unwrap(),
                order_date: DateTime::from_utc(t.order_date, Utc),
                order_notes: serde_json::from_value::<NoteList>(t.order_notes.clone()).unwrap(),
                salesperson: t.salesperson.clone(),
                till: t.till.clone(),
            })
            .collect();

        Ok(mapped)
    }

    pub async fn update(
        tsn: TransactionInput,
        id: &str,
        db: &DbConn,
    ) -> Result<Transaction, DbErr> {
        transactions::ActiveModel {
            id: Set(id.to_string()),
            customer: Set(json!(tsn.customer)),
            transaction_type: Set(tsn.transaction_type),
            products: Set(json!(tsn.products)),
            order_total: Set(tsn.order_total),
            payment: Set(json!(tsn.payment)),
            order_date: Set(tsn.order_date.naive_utc()),
            order_notes: Set(json!(tsn.order_notes)),
            salesperson: Set(tsn.salesperson),
            till: Set(tsn.till),
        }
        .update(db)
        .await?;

        Self::fetch_by_id(id, db).await
    }

    pub async fn update_value(
        tsn: Transaction,
        id: &str,
        db: &DbConn,
    ) -> Result<Transaction, DbErr> {
        transactions::ActiveModel {
            id: Set(id.to_string()),
            customer: Set(json!(tsn.customer)),
            transaction_type: Set(tsn.transaction_type),
            products: Set(json!(tsn.products)),
            order_total: Set(tsn.order_total),
            payment: Set(json!(tsn.payment)),
            order_date: Set(tsn.order_date.naive_utc()),
            order_notes: Set(json!(tsn.order_notes)),
            salesperson: Set(tsn.salesperson),
            till: Set(tsn.till),
        }
        .update(db)
        .await?;

        Self::fetch_by_id(id, db).await
    }

    pub async fn update_order_status(
        id: &str,
        refer: &str,
        status: OrderStatus,
        db: &DbConn,
    ) -> Result<Transaction, DbErr> {
        let mut transaction = Transaction::fetch_by_id(id, db).await?;

        let new_orders = transaction
            .clone()
            .products
            .into_iter()
            .map(|mut v| {
                if v.reference == refer {
                    let new_status = OrderStatusAssignment {
                        status: status.clone(),
                        assigned_products: v.products.iter().map(|el| el.id.clone()).collect(),
                        timestamp: Utc::now(),
                    };

                    v.status = new_status.clone();

                    v.status_history.push(History {
                        item: new_status,
                        reason: "Supered Update".to_string(),
                        timestamp: v.status.timestamp,
                    });
                }

                v
            })
            .collect::<Vec<Order>>();

        transaction.products = new_orders;

        Self::update_value(transaction, id, db).await
    }

    pub async fn update_product_status(
        id: &str,
        refer: &str,
        pid: &str,
        iid: &str,
        status: PickStatus,
        db: &DbConn,
    ) -> Result<Transaction, DbErr> {
        let mut transaction = Transaction::fetch_by_id(id, db).await?;

        let new_orders = transaction
            .clone()
            .products
            .into_iter()
            .map(|mut v| {
                if v.reference == refer {
                    let new_products = v
                        .products
                        .into_iter()
                        .map(|mut p| {
                            if p.id == pid {
                                p.instances = p
                                    .instances
                                    .into_iter()
                                    .map(|mut i| {
                                        if i.id == iid {
                                            i.fulfillment_status.pick_history.push(History {
                                                item: i.fulfillment_status.pick_status,
                                                reason: "Standard Update Bump".to_string(),
                                                timestamp: i.fulfillment_status.last_updated,
                                            });
                                            i.fulfillment_status.last_updated = Utc::now();
                                            i.fulfillment_status.pick_status = status.clone();
                                        }

                                        i
                                    })
                                    .collect::<Vec<ProductInstance>>();
                            }

                            p
                        })
                        .collect();

                    v.products = new_products;
                }

                v
            })
            .collect::<Vec<Order>>();

        transaction.products = new_orders;

        Self::update_value(transaction, id, db).await
    }

    pub async fn generate(
        db: &DbConn,
        customer_id: &str,
        session: Session,
    ) -> Result<Transaction, DbErr> {
        // Create Transaction
        let tsn = example_transaction(customer_id);

        // Insert & Fetch Transaction
        match Transaction::insert(tsn, session, db).await {
            Ok(data) => match Transaction::fetch_by_id(&data.last_insert_id, db).await {
                Ok(res) => Ok(res),
                Err(e) => Err(e),
            },
            Err(e) => Err(e),
        }
    }

    pub async fn process_intents(
        db: &DbConn,
        intents: Vec<QuantityAlterationIntent>,
    ) -> Vec<Result<Product, JoinError>> {
        let intent_processor = intents
            .iter()
            .map(|intent| async move {
                let intent = intent.clone();
                let database = db.clone();

                tokio::spawn(async move {
                    let db_ = database.clone();

                    match Product::fetch_by_id(&intent.product_sku, &db_).await {
                        Ok(mut val) => {
                            let variants: Vec<VariantInformation> = val
                                .variants
                                .iter_mut()
                                .map(|var| {
                                    let stock_info: Vec<Stock> = if var.barcode
                                        == intent.variant_code
                                    {
                                        var.stock
                                            .iter_mut()
                                            .map(|stock| {
                                                if stock.store.store_code
                                                    == intent.transaction_store_code
                                                {
                                                    match intent.transaction_type {
                                                        TransactionType::In => {
                                                            stock.quantity.quantity_sellable +=
                                                                intent.quantity_to_transact
                                                        }
                                                        TransactionType::Out => {
                                                            stock.quantity.quantity_sellable -=
                                                                intent.quantity_to_transact
                                                        }
                                                        TransactionType::PendingIn => {
                                                            stock.quantity.quantity_on_order +=
                                                                intent.quantity_to_transact
                                                        }
                                                        TransactionType::PendingOut => {
                                                            stock.quantity.quantity_allocated +=
                                                                intent.quantity_to_transact
                                                        }
                                                        TransactionType::Saved => {
                                                            // A saved transaction should not be processed, but should be shifted into a specified IN or OUT variant.
                                                            // As this should never happen, the modified changes are left alone.
                                                            stock.quantity.quantity_allocated += 0.0
                                                        }
                                                        TransactionType::Quote => {
                                                            // A saved transaction should not be processed, but should be shifted into a specified IN or OUT variant.
                                                            // As this should never happen, the modified changes are left alone.
                                                            stock.quantity.quantity_allocated += 0.0
                                                        }
                                                    }
                                                }

                                                stock.clone()
                                            })
                                            .collect::<Vec<Stock>>()
                                    } else {
                                        var.stock.clone()
                                    };

                                    var.stock = stock_info;
                                    var.clone()
                                })
                                .collect::<Vec<VariantInformation>>();

                            val.variants = variants;

                            // Possible chance for an alternate client to have a modification during this time-frame, try implementing a queued solution.
                            match Product::update(val, &intent.product_sku, &db_).await {
                                Ok(val) => Ok(val),
                                Err(_) => Err(DbErr::Custom(String::new())),
                            }
                        }
                        Err(_) => Err(DbErr::Custom(String::new())),
                    }
                    .unwrap()
                })
                .await
            })
            .collect::<Vec<_>>();

        futures::future::join_all(intent_processor).await
    }

    pub async fn delete(id: &str, db: &DbConn) -> Result<DeleteResult, DbErr> {
        Transactions::delete(transactions::ActiveModel {
            id: Set(id.to_string()),
            ..Default::default()
        })
        .exec(db)
        .await
    }
}

impl Display for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let products: String = self
            .products
            .iter()
            .map(|f| {
                let pdts: String = f
                    .products
                    .iter()
                    .map(|p| {
                        format!(
                            "\t{}: ${} ({}) {} {}  [-]{}\n",
                            p.quantity,
                            p.product_cost,
                            p.product_code,
                            p.product_name,
                            p.product_variant_name,
                            p.discount.to_string() // greatest_discount(p.discount.clone(), p.product_cost).to_string()
                        )
                    })
                    .collect();

                let notes: String = f.order_notes.iter().map(|p| format!("{}", p,)).collect();

                let statuses: String = format!("{}", f.status,);

                format!(
                    "-\t{} {} {} -> {} {} [-]{} \n{}\n\t{}\n",
                    f.reference,
                    statuses,
                    f.origin.store_code,
                    f.destination.store_code,
                    f.creation_date.format("%d/%m/%Y %H:%M"),
                    f.discount.to_string(),
                    pdts,
                    notes
                )
            })
            .collect();

        let notes: String = self.order_notes.iter().map(|p| format!("{}", p)).collect();

        // let order_history: String = self.order_history.iter()
        //     .map(|f| {
        //         format!(
        //             "{}: {}\n",
        //             f.timestamp.format("%d/%m/%Y %H:%M"),
        //             f.item,
        //         )
        //     }).collect();

        write!(
            f,
            "Transaction ({}) {} {}\nOrders:\n{}\n---\nTotal: ${}\nPayment: {:?}\nNotes:\n{}\n{}",
            self.id,
            self.order_date.format("%d/%m/%Y %H:%M"),
            self.till,
            products,
            self.order_total,
            self.payment,
            notes,
            self.salesperson
        )
    }
}

// // impl! Implement the intent as a builder.
// pub struct Intent {
//     request: Transaction,
//     // Employee ID for the dispatcher (instigator) for an In-store Purchase (i.e. Tills person) or website deployment ID
//     dispatcher: Id,
// }

// impl Intent {
//     //...
// }

pub fn example_transaction(customer_id: &str) -> TransactionInit {
    let torpedo7 = ContactInformation {
        name: "Torpedo7 Mt Wellington".into(),
        mobile: MobileNumber {
            number: "+6421212120".into(),
            valid: true,
        },
        email: Email {
            root: "order".into(),
            domain: "torpedo7.com".into(),
            full: "order@torpedo7.com".into(),
        },
        landline: "".into(),
        address: Address {
            street: "315-375 Mount Wellington Highway".into(),
            street2: "Mount Wellington".into(),
            city: "Auckland".into(),
            country: "New Zealand".into(),
            po_code: "1060".into(),
            lat: -36.915501,
            lon: 174.838745,
        },
    };

    let order = Order {
        destination: Location {
            store_code: "001".into(),
            store_id: "628f74d7-de00-4956-a5b6-2031e0c72128".to_string(),
            contact: torpedo7.clone(),
        },
        order_type: crate::methods::OrderType::Shipment,
        origin: Location {
            store_code: "002".into(),
            store_id: "c4a1d88b-e8a0-4dcd-ade2-1eea82254816".to_string(),
            contact: torpedo7.clone(),
        },
        products: vec![
            ProductPurchase {
                product_name: "Torpedo7 Nippers Kids Kayak & Paddle".to_string(),
                product_variant_name: "1.83m Beaches".to_string(),
                id: "PDT-KAYAK-PURCHASE-ID-1".to_string(),
                product_sku: "".into(),
                product_code: "54897443288214".into(),
                discount: DiscountValue::Absolute(0),
                product_cost: 399.99,
                quantity: 1.0,
                transaction_type: TransactionType::Out,
                tags: vec!["Tee".into(), "Cotton".into(), "Organic".into()],
                instances: vec![ProductInstance {
                    id: "def".to_string(),
                    fulfillment_status: crate::FulfillmentStatus {
                        pick_status: PickStatus::Pending,
                        pick_history: vec![],
                        last_updated: Utc::now(),
                        notes: vec![],
                    },
                }],
            },
            ProductPurchase {
                product_name: "Torpedo7 Kids Voyager II Paddle Vest".to_string(),
                product_variant_name: "Small Red (4-6y)".to_string(),
                id: "PDT-LIFEJACKET-PURCHASE-ID-1".to_string(),
                product_sku: "".into(),
                product_code: "51891265958214".into(),
                discount: DiscountValue::Absolute(0),
                product_cost: 139.99,
                quantity: 1.0,
                transaction_type: TransactionType::Out,
                tags: vec!["Tee".into(), "Cotton".into(), "Organic".into()],
                instances: vec![ProductInstance {
                    id: "def".to_string(),
                    fulfillment_status: crate::FulfillmentStatus {
                        pick_status: PickStatus::Pending,
                        pick_history: vec![],
                        last_updated: Utc::now(),
                        notes: vec![],
                    },
                }],
            },
        ],
        previous_failed_fulfillment_attempts: vec![],
        status: OrderStatusAssignment {
            // status: OrderStatus::Transit(
            //     TransitInformation {
            //         shipping_company: torpedo7.clone(),
            //         query_url: "https://www.fedex.com/fedextrack/?trknbr=".into(),
            //         tracking_code: "1523123".into(),
            //         assigned_products: vec!["132522-22".to_string()]
            //     }
            // )
            status: OrderStatus::Fulfilled(Utc::now()),
            assigned_products: vec!["132522-22".to_string()],
            timestamp: Utc::now(),
        },
        order_history: vec![],
        order_notes: vec![Note {
            message: "Order shipped from warehouse.".into(),
            timestamp: Utc::now(),
            author: Uuid::new_v4().to_string(),
        }],
        reference: "TOR-19592".into(),
        creation_date: Utc::now(),
        id: Uuid::new_v4().to_string(),
        status_history: vec![
            History::<OrderStatusAssignment> {
                item: OrderStatusAssignment {
                    status: OrderStatus::Queued(Utc::now()),
                    timestamp: Utc::now(),
                    assigned_products: vec!["PDT-KAYAK-PURCHASE-ID-1".to_string()],
                },
                timestamp: Utc::now(),
                reason: "Order Placed".to_string(),
            },
            History::<OrderStatusAssignment> {
                item: OrderStatusAssignment {
                    status: OrderStatus::Processing(
                        Utc::now().checked_add_signed(Duration::hours(1)).unwrap(),
                    ),
                    timestamp: Utc::now().checked_add_signed(Duration::hours(1)).unwrap(),
                    assigned_products: vec!["PDT-KAYAK-PURCHASE-ID-1".to_string()],
                },
                timestamp: Utc::now().checked_add_signed(Duration::hours(1)).unwrap(),
                reason: "Order received by store crew.".to_string(),
            },
            History::<OrderStatusAssignment> {
                item: OrderStatusAssignment {
                    status: OrderStatus::Transit(Box::new(TransitInformation {
                        shipping_company: torpedo7,
                        query_url: "https://www.fedex.com/fedextrack/?trknbr=".into(),
                        tracking_code: "1523123".into(),
                        assigned_products: vec!["132522-22".to_string()],
                    })),
                    timestamp: Utc::now().checked_add_signed(Duration::hours(2)).unwrap(),
                    assigned_products: vec!["132522-22".to_string()],
                },
                timestamp: Utc::now().checked_add_signed(Duration::hours(2)).unwrap(),
                reason: "Order shipped from warehouse.".to_string(),
            },
            History::<OrderStatusAssignment> {
                item: OrderStatusAssignment {
                    status: OrderStatus::Fulfilled(
                        Utc::now().checked_add_days(Days::new(2)).unwrap(),
                    ),
                    timestamp: Utc::now().checked_add_days(Days::new(2)).unwrap(),
                    assigned_products: vec!["132522-22".to_string()],
                },
                timestamp: Utc::now().checked_add_days(Days::new(2)).unwrap(),
                reason: "Item Delivered".to_string(),
            },
        ],
        discount: DiscountValue::Absolute(0),
    };

    TransactionInit {
        customer: TransactionCustomer {
            customer_id: customer_id.into(),
            customer_type: CustomerType::Individual,
        },
        transaction_type: TransactionType::In,
        products: vec![order],
        order_total: 115.00,
        payment: vec![Payment {
            id: Uuid::new_v4().to_string(),
            payment_method: crate::methods::PaymentMethod::Card,
            fulfillment_date: Utc::now(),
            amount: Price {
                quantity: 115.00,
                currency: "NZD".to_string(),
            },
            processing_fee: Price {
                quantity: 0.10,
                currency: "NZD".to_string(),
            },
            status: PaymentStatus::Unfulfilled(String::from(
                "Unable to fulfil payment requirements - insufficient funds.",
            )),
            processor: PaymentProcessor {
                location: "001".to_string(),
                employee: "EMPLOYEE_ID".to_string(),
                software_version: "k0.5.2".to_string(),
                token: Uuid::new_v4().to_string(),
            },
            order_ids: vec![Uuid::new_v4().to_string()],
            delay_action: PaymentAction::Cancel,
            delay_duration: "PT12H".to_string(),
        }],
        order_date: Utc::now(),
        order_notes: vec![Note {
            message: "Order packaged from warehouse.".into(),
            timestamp: Utc::now(),
            author: Uuid::new_v4().to_string(),
        }],
        // order_history: vec![History { item: ProductExchange { method_type: TransactionType::Out, product_code: "132522".into(), variant: vec!["22".into()], quantity: 1 }, reason: "Faulty Product".into(), timestamp: Utc::now() }],
        till: "...".into(),
    }
}
