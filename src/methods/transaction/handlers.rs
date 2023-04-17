use rocket::http::CookieJar;
use rocket::{get};
use rocket::{routes, post};
use rocket::serde::json::Json;
use sea_orm_rocket::{Connection};

use crate::check_permissions;
use crate::methods::{cookie_status_wrapper, Error, ErrorResponse, QuantityAlterationIntent};
use crate::pool::Db;
use super::{Transaction, TransactionInput, TransactionInit};
use crate::methods::employee::Action;

pub fn routes() -> Vec<rocket::Route> {
    routes![get, get_by_name, get_all_saved, get_by_product_sku, create, update, generate, delete]
}

#[get("/<id>")]
pub async fn get(conn: Connection<'_, Db>, id: String, cookies: &CookieJar<'_>) -> Result<Json<Transaction>, Error> {
    let db = conn.into_inner();
 
    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session.clone(), Action::FetchTransaction);

    match Transaction::fetch_by_id(&id, db).await {
        Ok(transaction) => Ok(Json(transaction)),
        Err(reason) => Err(ErrorResponse::db_err(reason))
    }
}

#[get("/saved")]
pub async fn get_all_saved(conn: Connection<'_, Db>, cookies: &CookieJar<'_>) -> Result<Json<Vec<Transaction>>, Error> {
    let db = conn.into_inner();
 
    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session.clone(), Action::FetchTransaction);

    match Transaction::fetch_all_saved(db).await {
        Ok(transaction) => Ok(Json(transaction)),
        Err(reason) => Err(ErrorResponse::db_err(reason))
    }
}

#[get("/ref/<name>")]
pub async fn get_by_name(conn: Connection<'_, Db>, name: &str, cookies: &CookieJar<'_>) -> Result<Json<Vec<Transaction>>, Error> {
    let db = conn.into_inner();

    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session.clone(), Action::FetchTransaction);

    match Transaction::fetch_by_ref(name, db).await {
        Ok(transaction) => Ok(Json(transaction)),
        Err(reason) => Err(ErrorResponse::db_err(reason))
    }
}

#[get("/product/<sku>")]
pub async fn get_by_product_sku(conn: Connection<'_, Db>, sku: &str, cookies: &CookieJar<'_>) -> Result<Json<Vec<Transaction>>, Error> {
    let db = conn.into_inner();
 
    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session.clone(), Action::FetchTransaction);

    match Transaction::fetch_by_ref(sku, db).await {
        Ok(transaction) => Ok(Json(transaction)),
        Err(reason) => Err(ErrorResponse::db_err(reason))
    }
}

#[get("/deliverables/<store_id>")]
pub async fn deliverables_search(conn: Connection<'_, Db>, store_id: &str, cookies: &CookieJar<'_>) -> Result<Json<Vec<Transaction>>, Error> {
    let db = conn.into_inner();
 
    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session.clone(), Action::FetchTransaction);

    match Transaction::fetch_by_ref(store_id, db).await {
        Ok(transaction) => Ok(Json(transaction)),
        Err(reason) => Err(ErrorResponse::db_err(reason))
    }
}

#[post("/<id>", data = "<input_data>")]
async fn update(
    conn: Connection<'_, Db>,
    id: &str,
    input_data: Json<TransactionInput>,
    cookies: &CookieJar<'_>
) -> Result<Json<Transaction>, Error> {
    let input_data = input_data.clone().into_inner();
    let db = conn.into_inner();

    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session.clone(), Action::ModifyTransaction);

    match Transaction::update(input_data, id, db).await {
        Ok(res) => {
            Ok(Json(res))
        },
        Err(_) => Err(ErrorResponse::input_error()),
    }
}

#[post("/generate/<customer_id>")]
async fn generate(
    conn: Connection<'_, Db>,
    customer_id: &str,
    cookies: &CookieJar<'_>
) -> Result<Json<Transaction>, Error> {
    let db = conn.into_inner();

    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session.clone(), Action::GenerateTemplateContent);

    match Transaction::generate(db, customer_id, session).await {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(ErrorResponse::input_error())
    }
}

#[post("/", data = "<input_data>")]
pub async fn create(conn: Connection<'_, Db>, input_data: Json<TransactionInit>, cookies: &CookieJar<'_>) -> Result<Json<Transaction>, Error> {
    let new_transaction = input_data.clone().into_inner();
    let db = conn.into_inner();

    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session.clone(), Action::CreateTransaction);

    let mut quantity_alteration_intents: Vec<QuantityAlterationIntent> = vec![];

    // Make and modify the required changes to stock levels
    new_transaction.products.iter().for_each(| order | {
        order.products.iter().for_each(| product | {
            quantity_alteration_intents.push(
                QuantityAlterationIntent {
                    variant_code: product.clone().product_code,
                    product_sku: product.clone().product_sku,
                    transaction_store_code: order.clone().origin.code,
                    transaction_type: new_transaction.clone().transaction_type,
                    quantity_to_transact: product.clone().quantity
                }
            );
        });
    });

    match Transaction::insert(new_transaction, session, db).await {
        Ok(data) => {
            Transaction::process_intents(db, quantity_alteration_intents).await;

            match Transaction::fetch_by_id(&data.last_insert_id, db).await {
                Ok(res) => {
                    Ok(Json(res))
                },  
                Err(reason) => Err(ErrorResponse::db_err(reason))
            }
        },
        Err(_) => Err(ErrorResponse::input_error()),
    }
}

#[post("/delete/<id>")]
async fn delete(
    conn: Connection<'_, Db>,
    id: &str,
    cookies: &CookieJar<'_>
) -> Result<(), Error> {
    let db = conn.into_inner();

    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session.clone(), Action::DeleteTransaction);

    match Transaction::delete(id, db).await {
        Ok(_res) => Ok(()),
        Err(_) => Err(ErrorResponse::input_error())
    }
}