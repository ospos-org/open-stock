use crate::check_permissions;
use crate::methods::employee::Action;
use crate::methods::{cookie_status_wrapper, Error, ErrorResponse};
use crate::pool::Db;
use rocket::get;
use rocket::http::CookieJar;
use rocket::serde::json::Json;
use rocket::{post, routes};
use rocket_db_pools::Connection;

use super::{Supplier, SupplierInput};

pub fn routes() -> Vec<rocket::Route> {
    routes![
        get,
        get_by_name,
        get_by_phone,
        get_by_addr,
        create,
        update,
        generate
    ]
}

#[get("/<id>")]
pub async fn get(
    conn: Connection<Db>,
    id: &str,
    cookies: &CookieJar<'_>,
) -> Result<Json<Supplier>, Error> {
    let db = conn.into_inner();

    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session.clone(), Action::FetchSupplier);

    match Supplier::fetch_by_id(id, session, db).await {
        Ok(supplier) => Ok(Json(supplier)),
        Err(reason) => Err(ErrorResponse::db_err(reason)),
    }
}

#[get("/name/<name>")]
pub async fn get_by_name(
    conn: Connection<Db>,
    name: &str,
    cookies: &CookieJar<'_>,
) -> Result<Json<Vec<Supplier>>, Error> {
    let db = conn.into_inner();

    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session.clone(), Action::FetchSupplier);

    match Supplier::fetch_by_name(name, session, db).await {
        Ok(suppliers) => Ok(Json(suppliers)),
        Err(reason) => Err(ErrorResponse::db_err(reason)),
    }
}

#[get("/phone/<phone>")]
pub async fn get_by_phone(
    conn: Connection<Db>,
    phone: &str,
    cookies: &CookieJar<'_>,
) -> Result<Json<Vec<Supplier>>, Error> {
    let db = conn.into_inner();

    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session.clone(), Action::FetchSupplier);

    match Supplier::fetch_by_phone(phone, session, db).await {
        Ok(suppliers) => Ok(Json(suppliers)),
        Err(reason) => Err(ErrorResponse::db_err(reason)),
    }
}

#[get("/addr/<addr>")]
pub async fn get_by_addr(
    conn: Connection<Db>,
    addr: &str,
    cookies: &CookieJar<'_>,
) -> Result<Json<Vec<Supplier>>, Error> {
    let db = conn.into_inner();

    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session.clone(), Action::FetchSupplier);

    match Supplier::fetch_by_addr(addr, session, db).await {
        Ok(suppliers) => Ok(Json(suppliers)),
        Err(reason) => Err(ErrorResponse::db_err(reason)),
    }
}

#[post("/generate")]
async fn generate(
    conn: Connection<Db>,
    cookies: &CookieJar<'_>,
) -> Result<Json<Supplier>, Error> {
    let db = conn.into_inner();

    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session.clone(), Action::GenerateTemplateContent);

    match Supplier::generate(session, db).await {
        Ok(res) => Ok(Json(res)),
        Err(err) => Err(ErrorResponse::db_err(err)),
    }
}

#[post("/<id>", data = "<input_data>")]
async fn update(
    conn: Connection<Db>,
    id: &str,
    cookies: &CookieJar<'_>,
    input_data: Json<SupplierInput>,
) -> Result<Json<Supplier>, Error> {
    let input_data = input_data.clone().into_inner();
    let db = conn.into_inner();

    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session.clone(), Action::ModifySupplier);

    match Supplier::update(input_data, session, id, db).await {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(ErrorResponse::input_error()),
    }
}

#[post("/", data = "<input_data>")]
pub async fn create(
    conn: Connection<Db>,
    cookies: &CookieJar<'_>,
    input_data: Json<SupplierInput>,
) -> Result<Json<Supplier>, Error> {
    let new_data = input_data.clone().into_inner();
    let db = conn.into_inner();

    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session.clone(), Action::ModifySupplier);

    match Supplier::insert(new_data, session.clone(), db).await {
        Ok(data) => match Supplier::fetch_by_id(&data.last_insert_id, session, db).await {
            Ok(res) => Ok(Json(res)),
            Err(reason) => {
                println!("[dberr]: {}", reason);
                Err(ErrorResponse::db_err(reason))
            }
        },
        Err(reason) => {
            println!("[dberr]: {}", reason);
            Err(ErrorResponse::input_error())
        }
    }
}
