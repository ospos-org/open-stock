use rocket::http::CookieJar;
use rocket::{http::Status, get, put, patch};
use rocket::{routes, post};
use rocket::serde::json::Json;
use sea_orm_rocket::{Connection};

use crate::methods::cookie_status_wrapper;
use crate::pool::Db;
use super::{Transaction, TransactionInput, TransactionInit};


pub fn routes() -> Vec<rocket::Route> {
    routes![get, get_by_name, get_by_product_sku, create, update, generate]
}

#[get("/<id>")]
pub async fn get(conn: Connection<'_, Db>, id: String, cookies: &CookieJar<'_>) -> Result<Json<Transaction>, Status> {
    let db = conn.into_inner();
    let _session = cookie_status_wrapper(db, cookies).await?;

    let transaction = Transaction::fetch_by_id(&id, db).await.unwrap();
    Ok(Json(transaction))
}

#[get("/ref/<name>")]
pub async fn get_by_name(conn: Connection<'_, Db>, name: &str, cookies: &CookieJar<'_>) -> Result<Json<Vec<Transaction>>, Status> {
    let db = conn.into_inner();
    let _session = cookie_status_wrapper(db, cookies).await?;

    let transaction = Transaction::fetch_by_ref(name, db).await.unwrap();
    Ok(Json(transaction))
}

#[get("/product/<sku>")]
pub async fn get_by_product_sku(conn: Connection<'_, Db>, sku: &str, cookies: &CookieJar<'_>) -> Result<Json<Vec<Transaction>>, Status> {
    let db = conn.into_inner();
    let _session = cookie_status_wrapper(db, cookies).await?;

    let transaction = Transaction::fetch_by_ref(sku, db).await.unwrap();
    Ok(Json(transaction))
}

#[put("/<id>", data = "<input_data>")]
async fn update(
    conn: Connection<'_, Db>,
    id: &str,
    input_data: Json<TransactionInput>,
    cookies: &CookieJar<'_>
) -> Result<Json<Transaction>, Status> {
    let input_data = input_data.clone().into_inner();
    let db = conn.into_inner();
    let _session = cookie_status_wrapper(db, cookies).await?;

    match Transaction::update(input_data, id, db).await {
        Ok(res) => {
            Ok(Json(res))
        },
        Err(_) => Err(Status::BadRequest),
    }
}

#[patch("/generate")]
async fn generate(
    conn: Connection<'_, Db>,
    cookies: &CookieJar<'_>
) -> Result<Json<Transaction>, Status> {
    let db = conn.into_inner();
    let session = cookie_status_wrapper(db, cookies).await?;

    match Transaction::generate(db, session).await {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(Status::BadRequest)
    }
}

#[post("/", data = "<input_data>")]
pub async fn create(conn: Connection<'_, Db>, input_data: Json<TransactionInit>, cookies: &CookieJar<'_>) -> Result<Json<Transaction>, Status> {
    let new_transaction = input_data.clone().into_inner();
    let db = conn.into_inner();
    let session = cookie_status_wrapper(db, cookies).await?;

    match Transaction::insert(new_transaction, session, db).await {
        Ok(data) => {
            match Transaction::fetch_by_id(&data.last_insert_id, db).await {
                Ok(res) => {
                    Ok(Json(res))
                },  
                Err(_) => Err(Status::InternalServerError)
            }
        },
        Err(_) => Err(Status::InternalServerError),
    }
}