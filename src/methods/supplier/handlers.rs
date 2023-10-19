use okapi::openapi3::OpenApi;
use crate::check_permissions;
use crate::methods::employee::Action;
use crate::methods::{cookie_status_wrapper, Error, ErrorResponse};
use crate::pool::Db;
use rocket::get;
use rocket::http::CookieJar;
use rocket::serde::json::Json;
use rocket::{post};
use rocket_db_pools::Connection;
use rocket_okapi::{openapi, openapi_get_routes_spec};
use rocket_okapi::settings::OpenApiSettings;

use super::{Supplier, SupplierInput};

pub fn documented_routes(settings: &OpenApiSettings) -> (Vec<rocket::Route>, OpenApi) {
    openapi_get_routes_spec![
        settings:
        get,
        get_by_name,
        get_by_phone,
        get_by_addr,
        create,
        update,
        generate
    ]
}

#[openapi(tag = "Supplier")]
#[get("/<id>")]
pub async fn get(
    conn: Connection<Db>,
    id: &str,
    cookies: &CookieJar<'_>,
) -> Result<Json<Supplier>, Error> {
    let db = conn.into_inner();

    let session = cookie_status_wrapper(&db, cookies).await?;
    check_permissions!(session.clone(), Action::FetchSupplier);

    match Supplier::fetch_by_id(id, session, &db).await {
        Ok(supplier) => Ok(Json(supplier)),
        Err(reason) => Err(ErrorResponse::db_err(reason)),
    }
}

#[openapi(tag = "Supplier")]
#[get("/name/<name>")]
pub async fn get_by_name(
    conn: Connection<Db>,
    name: &str,
    cookies: &CookieJar<'_>,
) -> Result<Json<Vec<Supplier>>, Error> {
    let db = conn.into_inner();

    let session = cookie_status_wrapper(&db, cookies).await?;
    check_permissions!(session.clone(), Action::FetchSupplier);

    match Supplier::fetch_by_name(name, session, &db).await {
        Ok(suppliers) => Ok(Json(suppliers)),
        Err(reason) => Err(ErrorResponse::db_err(reason)),
    }
}

#[openapi(tag = "Supplier")]
#[get("/phone/<phone>")]
pub async fn get_by_phone(
    conn: Connection<Db>,
    phone: &str,
    cookies: &CookieJar<'_>,
) -> Result<Json<Vec<Supplier>>, Error> {
    let db = conn.into_inner();

    let session = cookie_status_wrapper(&db, cookies).await?;
    check_permissions!(session.clone(), Action::FetchSupplier);

    match Supplier::fetch_by_phone(phone, session, &db).await {
        Ok(suppliers) => Ok(Json(suppliers)),
        Err(reason) => Err(ErrorResponse::db_err(reason)),
    }
}

#[openapi(tag = "Supplier")]
#[get("/addr/<addr>")]
pub async fn get_by_addr(
    conn: Connection<Db>,
    addr: &str,
    cookies: &CookieJar<'_>,
) -> Result<Json<Vec<Supplier>>, Error> {
    let db = conn.into_inner();

    let session = cookie_status_wrapper(&db, cookies).await?;
    check_permissions!(session.clone(), Action::FetchSupplier);

    match Supplier::fetch_by_addr(addr, session, &db).await {
        Ok(suppliers) => Ok(Json(suppliers)),
        Err(reason) => Err(ErrorResponse::db_err(reason)),
    }
}

#[openapi(tag = "Supplier")]
#[post("/generate")]
async fn generate(
    conn: Connection<Db>,
    cookies: &CookieJar<'_>,
) -> Result<Json<Supplier>, Error> {
    let db = conn.into_inner();

    let session = cookie_status_wrapper(&db, cookies).await?;
    check_permissions!(session.clone(), Action::GenerateTemplateContent);

    match Supplier::generate(session, &db).await {
        Ok(res) => Ok(Json(res)),
        Err(err) => Err(ErrorResponse::db_err(err)),
    }
}

#[openapi(tag = "Supplier")]
#[post("/<id>", data = "<input_data>")]
async fn update(
    conn: Connection<Db>,
    id: &str,
    cookies: &CookieJar<'_>,
    input_data: Json<SupplierInput>,
) -> Result<Json<Supplier>, Error> {
    let input_data = input_data.clone().into_inner();
    let db = conn.into_inner();

    let session = cookie_status_wrapper(&db, cookies).await?;
    check_permissions!(session.clone(), Action::ModifySupplier);

    match Supplier::update(input_data, session, id, &db).await {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(ErrorResponse::input_error()),
    }
}

#[openapi(tag = "Supplier")]
#[post("/", data = "<input_data>")]
pub async fn create(
    conn: Connection<Db>,
    cookies: &CookieJar<'_>,
    input_data: Json<SupplierInput>,
) -> Result<Json<Supplier>, Error> {
    let new_data = input_data.clone().into_inner();
    let db = conn.into_inner();

    let session = cookie_status_wrapper(&db, cookies).await?;
    check_permissions!(session.clone(), Action::ModifySupplier);

    match Supplier::insert(new_data, session.clone(), &db).await {
        Ok(data) =>
            match Supplier::fetch_by_id(
                &data.last_insert_id, session, &db
            ).await {
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
