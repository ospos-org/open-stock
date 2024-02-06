use super::{Transaction, TransactionInit, TransactionInput};
use crate::catchers::Validated;
use crate::guards::Convert;
use crate::methods::employee::Action;
use crate::methods::{Error, ErrorResponse, QuantityAlterationIntent};
use crate::{
    apply_discount, check_permissions, Order, OrderStatus, ProductStatusUpdate, TransactionType,
};
use okapi::openapi3::OpenApi;
use open_stock::{InternalDb, Session};
use rocket::get;
use rocket::post;
use rocket::serde::json::Json;
use rocket_okapi::settings::OpenApiSettings;
use rocket_okapi::{openapi, openapi_get_routes_spec};
use sea_orm::DbErr;

pub fn documented_routes(settings: &OpenApiSettings) -> (Vec<rocket::Route>, OpenApi) {
    openapi_get_routes_spec![
        settings:
        get,
        get_by_name,
        get_all_saved,
        get_by_product_sku,
        create,
        update,
        generate,
        delete,
        deliverables_search,
        update_product_status,
        update_order_status
    ]
}

#[openapi(tag = "Transaction")]
#[get("/<id>")]
pub async fn get(db: InternalDb, session: Session, id: &str) -> Convert<Transaction> {
    check_permissions!(session.clone(), Action::FetchTransaction);
    Transaction::fetch_by_id(id, session, &db.0).await.into()
}

#[openapi(tag = "Transaction")]
#[get("/saved")]
pub async fn get_all_saved(db: InternalDb, session: Session) -> Convert<Vec<Transaction>> {
    check_permissions!(session.clone(), Action::FetchTransaction);
    Transaction::fetch_all_saved(session, &db.0).await.into()
}

#[openapi(tag = "Transaction")]
#[get("/ref/<name>")]
pub async fn get_by_name(
    db: InternalDb,
    session: Session,
    name: &str,
) -> Convert<Vec<Transaction>> {
    check_permissions!(session.clone(), Action::FetchTransaction);
    Transaction::fetch_by_ref(name, session, &db.0).await.into()
}

#[openapi(tag = "Transaction")]
#[get("/product/<sku>")]
pub async fn get_by_product_sku(
    db: InternalDb,
    session: Session,
    sku: &str,
) -> Convert<Vec<Transaction>> {
    check_permissions!(session.clone(), Action::FetchTransaction);
    Transaction::fetch_by_ref(sku, session, &db.0).await.into()
}

#[openapi(tag = "Transaction")]
#[get("/deliverables/<store_id>")]
pub async fn deliverables_search(
    session: Session,
    db: InternalDb,
    store_id: &str,
) -> Convert<Vec<Order>> {
    check_permissions!(session.clone(), Action::FetchTransaction);
    Transaction::fetch_deliverable_jobs(store_id, session, &db.0)
        .await
        .into()
}

#[openapi(tag = "Transaction")]
#[get("/receivables/<store_id>")]
pub async fn receivables_search(
    db: InternalDb,
    session: Session,
    store_id: &str,
) -> Convert<Vec<Order>> {
    check_permissions!(session.clone(), Action::FetchTransaction);
    Transaction::fetch_receivable_jobs(store_id, session, &db.0)
        .await
        .into()
}

#[openapi(tag = "Transaction")]
#[post("/<id>", data = "<input_data>")]
async fn update(
    db: InternalDb,
    session: Session,
    input_data: Validated<Json<TransactionInput>>,
    id: &str,
) -> Convert<Transaction> {
    check_permissions!(session.clone(), Action::ModifyTransaction);
    Transaction::update(input_data.data(), session, id, &db.0)
        .await
        .into()
}

#[openapi(tag = "Transaction")]
#[post("/status/order/<refer>", data = "<status>")]
async fn update_order_status(
    db: InternalDb,
    session: Session,
    refer: &str,
    status: Validated<Json<OrderStatus>>,
) -> Result<Json<Transaction>, Error> {
    check_permissions!(session.clone(), Action::ModifyTransaction);

    let fetched_transaction = Transaction::fetch_by_ref(refer, session.clone(), &db.0).await?;

    match fetched_transaction.get(0) {
        Some(transaction) => Transaction::update_order_status(
            transaction.id.as_str(),
            refer,
            status.data(),
            session,
            &db.0,
        )
        .await
        .into(),
        None => Err(DbErr::RecordNotFound("Could not retrieve transaction".to_string()).into()),
    }
}

#[openapi(tag = "Transaction")]
#[post("/status/product", data = "<data>")]
async fn update_product_status(
    db: InternalDb,
    session: Session,
    data: Validated<Json<ProductStatusUpdate>>,
) -> Result<Json<Transaction>, Error> {
    check_permissions!(session.clone(), Action::ModifyTransaction);

    let data = data.data();
    let fetched_transaction =
        Transaction::fetch_by_ref(&data.transaction_id, session.clone(), &db.0).await?;

    match fetched_transaction.get(0) {
        Some(transaction) => {
            Transaction::update_product_status(transaction.id.as_str(), data, session, &db.0)
                .await
                .into()
        }
        None => Err(DbErr::RecordNotFound("Could not retrieve transaction".to_string()).into()),
    }
}

#[openapi(tag = "Transaction")]
#[post("/generate/<customer_id>")]
async fn generate(db: InternalDb, session: Session, customer_id: &str) -> Convert<Transaction> {
    check_permissions!(session.clone(), Action::GenerateTemplateContent);
    Transaction::generate(&db.0, customer_id, session)
        .await
        .into()
}

#[openapi(tag = "Transaction")]
#[post("/", data = "<input_data>")]
pub async fn create(
    db: InternalDb,
    session: Session,
    input_data: Validated<Json<TransactionInit>>,
) -> Result<Json<Transaction>, Error> {
    check_permissions!(session.clone(), Action::CreateTransaction);

    let mut quantity_alteration_intents: Vec<QuantityAlterationIntent> = vec![];
    let new_transaction = input_data.data();

    // Make and modify the required changes to stock levels
    new_transaction.products.iter().for_each(|order| {
        order.products.iter().for_each(|product| {
            quantity_alteration_intents.push(QuantityAlterationIntent {
                variant_code: product.clone().product_code,
                product_sku: product.clone().product_sku,
                transaction_store_code: order.clone().origin.store_code,
                transaction_store_id: order.clone().origin.store_id,
                transaction_type: new_transaction.clone().transaction_type,
                quantity_to_transact: product.clone().quantity,
            });
        });
    });

    let total_paid = new_transaction
        .payment
        .iter()
        .map(|payment| payment.amount.quantity)
        .sum::<f32>();

    let total_cost = new_transaction
        .products
        .iter()
        .map(|order| {
            apply_discount(
                order.discount.clone(),
                order
                    .products
                    .iter()
                    .map(|product| {
                        apply_discount(
                            product.discount.clone(),
                            product.product_cost * product.quantity,
                        )
                    })
                    .sum::<f32>(),
            )
        })
        .sum::<f32>();

    println!("Paid: {}. Cost: {}", total_paid, total_cost);

    let insertion = match new_transaction.transaction_type {
        TransactionType::Saved => {
            // We do not need to process intents. Simply save.
            Transaction::insert(new_transaction, session.clone(), &db.0).await?
        }
        _ => {
            // As we are removing inventory via a purchase,
            // we need to process the intents.

            if (total_paid - total_cost).abs() > 0.1 {
                return Err(ErrorResponse::create_error(
                    "Payment amount does not match product costs.",
                ));
            }

            let data = Transaction::insert(new_transaction, session.clone(), &db.0).await?;
            Transaction::process_intents(session.clone(), &db.0, quantity_alteration_intents).await;

            data
        }
    };

    Transaction::fetch_by_id(&insertion.last_insert_id, session, &db.0)
        .await
        .into()
}

#[openapi(tag = "Transaction")]
#[post("/delete/<id>")]
async fn delete(db: InternalDb, session: Session, id: &str) -> Convert<()> {
    check_permissions!(session.clone(), Action::DeleteTransaction);
    Transaction::delete(id, session, &db.0).await.into()
}
