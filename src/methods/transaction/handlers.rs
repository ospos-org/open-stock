use okapi::openapi3::OpenApi;
use rocket::get;
use rocket::http::CookieJar;
use rocket::serde::json::Json;
use rocket::{post};
use rocket_db_pools::Connection;
use rocket_okapi::{openapi, openapi_get_routes_spec};
use rocket_okapi::settings::OpenApiSettings;
use crate::catchers::Validated;

use super::{Transaction, TransactionInit, TransactionInput};
use crate::methods::employee::Action;
use crate::methods::{cookie_status_wrapper, Error, ErrorResponse, QuantityAlterationIntent};
use crate::pool::Db;
use crate::{apply_discount, check_permissions, Order, OrderStatus, ProductStatusUpdate, TransactionType};

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
pub async fn get(
    conn: Connection<Db>,
    id: String,
    cookies: &CookieJar<'_>,
) -> Result<Json<Transaction>, Error> {
    let db = conn.into_inner();

    let session = cookie_status_wrapper(&db, cookies).await?;
    check_permissions!(session.clone(), Action::FetchTransaction);

    match Transaction::fetch_by_id(&id, session, &db).await {
        Ok(transaction) => Ok(Json(transaction)),
        Err(reason) => Err(ErrorResponse::db_err(reason)),
    }
}

#[openapi(tag = "Transaction")]
#[get("/saved")]
pub async fn get_all_saved(
    conn: Connection<Db>,
    cookies: &CookieJar<'_>,
) -> Result<Json<Vec<Transaction>>, Error> {
    let db = conn.into_inner();

    let session = cookie_status_wrapper(&db, cookies).await?;
    check_permissions!(session.clone(), Action::FetchTransaction);

    match Transaction::fetch_all_saved(session, &db).await {
        Ok(transaction) => Ok(Json(transaction)),
        Err(reason) => Err(ErrorResponse::db_err(reason)),
    }
}

#[openapi(tag = "Transaction")]
#[get("/ref/<name>")]
pub async fn get_by_name(
    conn: Connection<Db>,
    name: &str,
    cookies: &CookieJar<'_>,
) -> Result<Json<Vec<Transaction>>, Error> {
    let db = conn.into_inner();

    let session = cookie_status_wrapper(&db, cookies).await?;
    check_permissions!(session.clone(), Action::FetchTransaction);

    match Transaction::fetch_by_ref(name, session, &db).await {
        Ok(transaction) => Ok(Json(transaction)),
        Err(reason) => Err(ErrorResponse::db_err(reason)),
    }
}

#[openapi(tag = "Transaction")]
#[get("/product/<sku>")]
pub async fn get_by_product_sku(
    conn: Connection<Db>,
    sku: &str,
    cookies: &CookieJar<'_>,
) -> Result<Json<Vec<Transaction>>, Error> {
    let db = conn.into_inner();

    let session = cookie_status_wrapper(&db, cookies).await?;
    check_permissions!(session.clone(), Action::FetchTransaction);

    match Transaction::fetch_by_ref(sku, session, &db).await {
        Ok(transaction) => Ok(Json(transaction)),
        Err(reason) => Err(ErrorResponse::db_err(reason)),
    }
}

#[openapi(tag = "Transaction")]
#[get("/deliverables/<store_id>")]
pub async fn deliverables_search(
    conn: Connection<Db>,
    store_id: &str,
    cookies: &CookieJar<'_>,
) -> Result<Json<Vec<Order>>, Error> {
    let db = conn.into_inner();

    let session = cookie_status_wrapper(&db, cookies).await?;
    check_permissions!(session.clone(), Action::FetchTransaction);

    match Transaction::fetch_deliverable_jobs(store_id, session, &db).await {
        Ok(transaction) => Ok(Json(transaction)),
        Err(reason) => Err(ErrorResponse::db_err(reason)),
    }
}

#[openapi(tag = "Transaction")]
#[get("/receivables/<store_id>")]
pub async fn receivables_search(
    conn: Connection<Db>,
    store_id: &str,
    cookies: &CookieJar<'_>,
) -> Result<Json<Vec<Order>>, Error> {
    let db = conn.into_inner();
    let session = cookie_status_wrapper(&db, cookies).await?;
    check_permissions!(session.clone(), Action::FetchTransaction);

    match Transaction::fetch_receivable_jobs(store_id, session, &db).await {
        Ok(transaction) => Ok(Json(transaction)),
        Err(reason) => Err(ErrorResponse::db_err(reason)),
    }
}

#[openapi(tag = "Transaction")]
#[post("/<id>", data = "<input_data>")]
async fn update(
    conn: Connection<Db>,
    id: &str,
    input_data: Validated<Json<TransactionInput>>,
    cookies: &CookieJar<'_>,
) -> Result<Json<Transaction>, Error> {
    let input_data = input_data.clone().0.into_inner();
    let db = conn.into_inner();

    let session = cookie_status_wrapper(&db, cookies).await?;
    check_permissions!(session.clone(), Action::ModifyTransaction);

    match Transaction::update(input_data, session, id, &db).await {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(ErrorResponse::input_error()),
    }
}

#[openapi(tag = "Transaction")]
#[post("/status/order/<refer>", data = "<status>")]
async fn update_order_status(
    conn: Connection<Db>,
    refer: &str,
    status: Json<OrderStatus>,
    cookies: &CookieJar<'_>,
) -> Result<Json<Transaction>, Error> {
    let status = status.clone().into_inner();
    let db = conn.into_inner();

    let session = cookie_status_wrapper(&db, cookies).await?;
    check_permissions!(session.clone(), Action::ModifyTransaction);

    let tsn = Transaction::fetch_by_ref(refer, session.clone(), &db)
        .await
        .unwrap();
    let id = tsn.get(0).unwrap().id.as_str();

    match Transaction::update_order_status(id, refer, status, session, &db).await {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(ErrorResponse::input_error()),
    }
}

#[openapi(tag = "Transaction")]
#[post("/status/product", data = "<data>")]
async fn update_product_status(
    conn: Connection<Db>,
    data: Validated<Json<ProductStatusUpdate>>,
    cookies: &CookieJar<'_>,
) -> Result<Json<Transaction>, Error> {
    let db = conn.into_inner();
    let data = data.clone().0.into_inner();

    let session = cookie_status_wrapper(&db, cookies).await?;
    check_permissions!(session.clone(), Action::ModifyTransaction);

    let tsn = Transaction::fetch_by_ref(&data.transaction_id.clone(), session.clone(), &db)
        .await
        .unwrap();
    let id = tsn.get(0).unwrap().id.as_str();

    match Transaction::update_product_status(id, data, session, &db).await
    {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(ErrorResponse::input_error()),
    }
}

#[openapi(tag = "Transaction")]
#[post("/generate/<customer_id>")]
async fn generate(
    conn: Connection<Db>,
    customer_id: &str,
    cookies: &CookieJar<'_>,
) -> Result<Json<Transaction>, Error> {
    let db = conn.into_inner();

    let session = cookie_status_wrapper(&db, cookies).await?;
    check_permissions!(session.clone(), Action::GenerateTemplateContent);

    match Transaction::generate(&db, customer_id, session).await {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(ErrorResponse::input_error()),
    }
}

#[openapi(tag = "Transaction")]
#[post("/", data = "<input_data>")]
pub async fn create(
    conn: Connection<Db>,
    input_data: Validated<Json<TransactionInit>>,
    cookies: &CookieJar<'_>,
) -> Result<Json<Transaction>, Error> {
    let new_transaction = input_data.clone().0.into_inner();
    let db = conn.into_inner();

    let session = cookie_status_wrapper(&db, cookies).await?;
    check_permissions!(session.clone(), Action::CreateTransaction);

    let mut quantity_alteration_intents: Vec<QuantityAlterationIntent> = vec![];

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
            Transaction::insert(new_transaction, session.clone(), &db).await?
        }
        _ => {
            // As we are removing inventory via a purchase,
            // we need to process the intents.

            if (total_paid - total_cost).abs() > 0.1 {
                return Err(ErrorResponse::create_error(
                    "Payment amount does not match product costs.",
                ));
            }

            let data = Transaction::insert(new_transaction, session.clone(), &db).await?;
            Transaction::process_intents(session.clone(), &db, quantity_alteration_intents).await;

            data
        }
    };

    Ok(Json(Transaction::fetch_by_id(&insertion.last_insert_id, session, &db).await?))
}

#[openapi(tag = "Transaction")]
#[post("/delete/<id>")]
async fn delete(conn: Connection<Db>, id: &str, cookies: &CookieJar<'_>) -> Result<(), Error> {
    let db = conn.into_inner();

    let session = cookie_status_wrapper(&db, cookies).await?;
    check_permissions!(session.clone(), Action::DeleteTransaction);

    match Transaction::delete(id, session, &db).await {
        Ok(_res) => Ok(()),
        Err(_) => Err(ErrorResponse::input_error()),
    }
}
