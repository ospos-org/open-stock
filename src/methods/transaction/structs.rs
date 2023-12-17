use core::fmt;
use std::fmt::Display;

use chrono::{DateTime, NaiveDateTime, Utc};
use schemars::JsonSchema;
#[cfg(feature = "process")]
use sea_orm::{
    sea_query::{Expr, Func},
    *,
};
use serde::{Deserialize, Serialize};
#[cfg(feature = "process")]
use tokio::task::JoinError;
use uuid::Uuid;

#[cfg(feature = "process")]
use sea_orm::FromQueryResult;

#[cfg(feature = "process")]
use crate::entities::{
    prelude::Transactions, sea_orm_active_enums::TransactionType as SeaORMTType, transactions,
};
use crate::{
    methods::{
        History, Id, NoteList, Order, OrderList, OrderStatus, OrderStatusAssignment, Payment,
        Product, Session, Stock, VariantInformation,
    },
    PickStatus, ProductInstance,
};
#[cfg(feature = "process")]
use sea_orm::DbConn;
use crate::transaction::example::example_transaction;

#[cfg(feature = "types")]
#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct TransactionCustomer {
    pub customer_type: CustomerType,
    pub customer_id: String,
}

#[cfg(feature = "types")]
#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub enum CustomerType {
    Store,
    Individual,
    Commercial,
}

#[cfg(feature = "process")]
#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct QuantityAlterationIntent {
    pub variant_code: String,
    pub product_sku: String,
    pub transaction_store_code: String,
    pub transaction_store_id: String,
    pub transaction_type: TransactionType,
    pub quantity_to_transact: f32,
}

#[cfg(feature = "types")]
#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub enum TransactionType {
    In,
    Out,
    PendingIn,
    PendingOut,
    Saved,
    Quote,
}

// Discounts on the transaction are applied per-order - such that they are unique to each item,
// i.e. each item can be discounted individually where needed to close a sale.
// A discount placed upon the payment object is an order-discount,
// such that it will act upon the basket:

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
#[cfg(feature = "types")]
#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct Transaction {
    pub id: Id,

    pub customer: TransactionCustomer,
    pub transaction_type: TransactionType,

    pub products: OrderList,
    pub order_total: i64,
    pub payment: Vec<Payment>,

    pub order_date: DateTime<Utc>,
    pub order_notes: NoteList,

    pub salesperson: Id,
    pub kiosk: Id,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>
}

#[cfg(feature = "process")]
#[cfg(feature = "types")]
#[derive(Serialize, Deserialize, Clone, FromQueryResult, JsonSchema)]
pub struct DerivableTransaction {
    pub id: Id,

    pub customer: JsonValue,
    pub transaction_type: SeaORMTType,

    pub products: JsonValue,
    pub order_total: i64,
    pub payment: JsonValue,

    pub order_date: NaiveDateTime,
    pub order_notes: JsonValue,

    pub salesperson: Id,
    pub kiosk: Id,
}

#[cfg(feature = "types")]
#[derive(Deserialize, Clone, JsonSchema)]
pub struct TransactionInput {
    pub customer: TransactionCustomer,
    pub transaction_type: TransactionType,

    pub products: OrderList,
    pub order_total: i64,
    pub payment: Vec<Payment>,

    pub order_date: DateTime<Utc>,
    pub order_notes: NoteList,

    pub salesperson: Id,
    pub kiosk: Id,
}

#[cfg(feature = "types")]
#[derive(Deserialize, Clone, JsonSchema)]
pub struct TransactionInit {
    pub customer: TransactionCustomer,
    pub transaction_type: TransactionType,

    pub products: OrderList,
    pub order_total: i64,
    pub payment: Vec<Payment>,

    pub order_date: DateTime<Utc>,
    pub order_notes: NoteList,

    pub kiosk: Id,
}

#[cfg(feature = "methods")]
impl Transaction {
    pub async fn insert(
        tsn: TransactionInit,
        session: Session,
        db: &DbConn,
    ) -> Result<InsertResult<transactions::ActiveModel>, DbErr> {
        let id = Uuid::new_v4().to_string();

        match Transactions::insert(tsn.into_active(id, session)).exec(db).await {
            Ok(res) => Ok(res),
            Err(err) => Err(err),
        }
    }

    pub async fn insert_raw(
        tsn: Transaction,
        session: Session,
        db: &DbConn,
    ) -> Result<InsertResult<transactions::ActiveModel>, DbErr> {
        match Transactions::insert(tsn.into_active(session.tenant_id)).exec(db).await {
            Ok(res) => Ok(res),
            Err(err) => Err(err),
        }
    }

    pub async fn fetch_deliverable_jobs(
        query: &str,
        session: Session,
        db: &DbConn,
    ) -> Result<Vec<Order>, DbErr> {
        let as_str: Vec<DerivableTransaction> =
            DerivableTransaction::find_by_statement(Statement::from_sql_and_values(
                DbBackend::MySql,
                &format!(
                    "SELECT * FROM Transactions WHERE Transactions.products LIKE '%{}%' AND Transactions.tenant_id = '{}'",
                    query, session.tenant_id
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

    pub async fn fetch_receivable_jobs(
        query: &str,
        session: Session,
        db: &DbConn,
    ) -> Result<Vec<Order>, DbErr> {
        let as_str: Vec<DerivableTransaction> =
            DerivableTransaction::find_by_statement(Statement::from_sql_and_values(
                DbBackend::MySql,
                &format!(
                    "SELECT * FROM Transactions WHERE Transactions.products LIKE '%{}%' AND Transactions.tenant_id = '{}'",
                    query, session.tenant_id
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

    pub async fn fetch_by_id(
        id: &str,
        session: Session,
        db: &DbConn,
    ) -> Result<Transaction, DbErr> {
        let tsn = Transactions::find_by_id(id.to_string())
            .filter(transactions::Column::TenantId.eq(session.tenant_id))
            .one(db)
            .await?;

        if tsn.is_none() {
            return Err(DbErr::Custom(
                "Unable to query value, returns none".to_string(),
            ));
        }

        Ok(tsn.unwrap().into())
    }

    pub async fn fetch_all_saved(session: Session, db: &DbConn) -> Result<Vec<Transaction>, DbErr> {
        let res = Transactions::find()
            .filter(transactions::Column::TenantId.eq(session.tenant_id))
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
            .map(|t| t.clone().into())
            .collect();

        Ok(mapped)
    }

    pub async fn fetch_by_ref(
        reference: &str,
        session: Session,
        db: &DbConn,
    ) -> Result<Vec<Transaction>, DbErr> {
        let res = Transactions::find()
            .filter(transactions::Column::TenantId.eq(session.tenant_id))
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
            .map(|t| t.clone().into())
            .collect();

        Ok(mapped)
    }

    pub async fn fetch_by_client_id(
        id: &str,
        session: Session,
        db: &DbConn,
    ) -> Result<Vec<Transaction>, DbErr> {
        let tsn = Transactions::find()
            .filter(transactions::Column::TenantId.eq(session.tenant_id))
            .having(transactions::Column::Customer.contains(id))
            .all(db)
            .await?;

        let mapped = tsn
            .iter()
            .map(|t| t.clone().into())
            .collect();

        Ok(mapped)
    }

    pub async fn update(
        tsn: TransactionInput,
        session: Session,
        id: &str,
        db: &DbConn,
    ) -> Result<Transaction, DbErr> {
        tsn.into_active(id.to_string(), session.clone())
            .update(db)
            .await?;

        Self::fetch_by_id(id, session, db).await
    }

    pub async fn update_value(
        tsn: Transaction,
        session: Session,
        id: &str,
        db: &DbConn,
    ) -> Result<Transaction, DbErr> {
        tsn.into_active(session.tenant_id.clone())
            .update(db)
            .await?;

        Self::fetch_by_id(id, session, db).await
    }

    pub async fn update_order_status(
        id: &str,
        refer: &str,
        status: OrderStatus,
        session: Session,
        db: &DbConn,
    ) -> Result<Transaction, DbErr> {
        let mut transaction = Transaction::fetch_by_id(id, session.clone(), db).await?;

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

        Self::update_value(transaction, session, id, db).await
    }

    pub async fn update_product_status(
        id: &str,
        refer: &str,
        pid: &str,
        iid: &str,
        status: PickStatus,
        session: Session,
        db: &DbConn,
    ) -> Result<Transaction, DbErr> {
        let mut transaction = Transaction::fetch_by_id(id, session.clone(), db).await?;

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

        Self::update_value(transaction, session, id, db).await
    }

    pub async fn generate(
        db: &DbConn,
        customer_id: &str,
        session: Session,
    ) -> Result<Transaction, DbErr> {
        // Create Transaction
        let tsn = example_transaction(customer_id);

        // Insert & Fetch Transaction
        match Transaction::insert(tsn, session.clone(), db).await {
            Ok(data) => match Transaction::fetch_by_id(&data.last_insert_id, session, db).await {
                Ok(res) => Ok(res),
                Err(e) => Err(e),
            },
            Err(e) => Err(e),
        }
    }

    pub async fn process_intents(
        session: Session,
        db: &DbConn,
        intents: Vec<QuantityAlterationIntent>,
    ) -> Vec<Result<Product, JoinError>> {
        let intent_processor = intents
            .iter()
            .map(|intent| async {
                let intent = intent.clone();
                let database = db.clone();
                let session = session.clone();

                println!("{}", session.employee);

                tokio::spawn(async move {
                    let db_ = database.clone();
                    let session_clone = session.clone();

                    match Product::fetch_by_id(&intent.product_sku, session.clone(), &db_).await {
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
                            match Product::update(val, session_clone, &intent.product_sku, &db_)
                                .await
                            {
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

    pub async fn delete(id: &str, session: Session, db: &DbConn) -> Result<DeleteResult, DbErr> {
        Transactions::delete(transactions::ActiveModel {
            id: Set(id.to_string()),
            tenant_id: Set(session.tenant_id),
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

        write!(
            f,
            "Transaction ({}) {} {}\nOrders:\n{}\n---\nTotal: ${}\nPayment: {:?}\nNotes:\n{}\n{}",
            self.id,
            self.order_date.format("%d/%m/%Y %H:%M"),
            self.kiosk,
            products,
            self.order_total,
            self.payment,
            notes,
            self.salesperson
        )
    }
}
