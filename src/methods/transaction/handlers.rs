use rocket::http::CookieJar;
use rocket::{get};
use rocket::{routes, post};
use rocket::serde::json::Json;
use sea_orm_rocket::{Connection};

use crate::check_permissions;
use crate::methods::{cookie_status_wrapper, Error, ErrorResponse};
use crate::pool::Db;
use super::{Transaction, TransactionInput, TransactionInit};
use crate::methods::employee::Action;

pub fn routes() -> Vec<rocket::Route> {
    routes![get, get_by_name, get_by_product_sku, create, update, generate]
}

#[get("/<id>")]
pub async fn get(conn: Connection<'_, Db>, id: String, cookies: &CookieJar<'_>) -> Result<Json<Transaction>, Error> {
    let db = conn.into_inner();
 
    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session.clone(), Action::FetchTransaction);

    let transaction = Transaction::fetch_by_id(&id, db).await.unwrap();
    Ok(Json(transaction))
}

#[get("/ref/<name>")]
pub async fn get_by_name(conn: Connection<'_, Db>, name: &str, cookies: &CookieJar<'_>) -> Result<Json<Vec<Transaction>>, Error> {
    let db = conn.into_inner();

    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session.clone(), Action::FetchTransaction);

    let transaction = Transaction::fetch_by_ref(name, db).await.unwrap();
    Ok(Json(transaction))
}

#[get("/product/<sku>")]
pub async fn get_by_product_sku(conn: Connection<'_, Db>, sku: &str, cookies: &CookieJar<'_>) -> Result<Json<Vec<Transaction>>, Error> {
    let db = conn.into_inner();
 
    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session.clone(), Action::FetchTransaction);

    let transaction = Transaction::fetch_by_ref(sku, db).await.unwrap();
    Ok(Json(transaction))
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

#[post("/generate")]
async fn generate(
    conn: Connection<'_, Db>,
    cookies: &CookieJar<'_>
) -> Result<Json<Transaction>, Error> {
    let db = conn.into_inner();

    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session.clone(), Action::GenerateTemplateContent);

    match Transaction::generate(db, session).await {
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

    match Transaction::insert(new_transaction, session, db).await {
        Ok(data) => {
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