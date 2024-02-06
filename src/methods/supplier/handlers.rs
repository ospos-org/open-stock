use crate::catchers::Validated;
use crate::guards::Convert;
use crate::methods::employee::Action;
use crate::methods::{cookie_status_wrapper, Error, ErrorResponse};
use crate::pool::{Db, InternalDb};
use crate::{check_permissions, Session};
use okapi::openapi3::OpenApi;
use rocket::get;
use rocket::http::CookieJar;
use rocket::post;
use rocket::serde::json::Json;
use rocket_db_pools::Connection;
use rocket_okapi::settings::OpenApiSettings;
use rocket_okapi::{openapi, openapi_get_routes_spec};

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
pub async fn get(db: InternalDb, session: Session, id: &str) -> Convert<Supplier> {
    check_permissions!(session.clone(), Action::FetchSupplier);
    Supplier::fetch_by_id(id, session, &db.0).await.into()
}

#[openapi(tag = "Supplier")]
#[get("/name/<name>")]
pub async fn get_by_name(
    db: InternalDb,
    session: Session,
    name: &str,
) -> Result<Json<Vec<Supplier>>, Error> {
    check_permissions!(session.clone(), Action::FetchSupplier);
    Supplier::fetch_by_name(name, session, &db.0).await.into()
}

#[openapi(tag = "Supplier")]
#[get("/phone/<phone>")]
pub async fn get_by_phone(db: InternalDb, session: Session, phone: &str) -> Convert<Vec<Supplier>> {
    check_permissions!(session.clone(), Action::FetchSupplier);
    Supplier::fetch_by_phone(phone, session, &db.0).await.into()
}

#[openapi(tag = "Supplier")]
#[get("/addr/<addr>")]
pub async fn get_by_addr(db: InternalDb, session: Session, addr: &str) -> Convert<Vec<Supplier>> {
    check_permissions!(session.clone(), Action::FetchSupplier);
    Supplier::fetch_by_addr(addr, session, &db.0).await.into()
}

#[openapi(tag = "Supplier")]
#[post("/generate")]
async fn generate(session: Session, db: InternalDb) -> Convert<Supplier> {
    check_permissions!(session.clone(), Action::GenerateTemplateContent);
    Supplier::generate(session, &db.0).await.into()
}

#[openapi(tag = "Supplier")]
#[post("/<id>", data = "<input_data>")]
async fn update(
    session: Session,
    db: InternalDb,
    input_data: Validated<Json<SupplierInput>>,
    id: &str,
) -> Convert<Supplier> {
    check_permissions!(session.clone(), Action::ModifySupplier);
    Supplier::update(input_data.data(), session, id, &db.0)
        .await
        .into()
}

#[openapi(tag = "Supplier")]
#[post("/", data = "<input_data>")]
pub async fn create(
    session: Session,
    db: InternalDb,
    input_data: Json<SupplierInput>,
) -> Result<Json<Supplier>, Error> {
    check_permissions!(session.clone(), Action::ModifySupplier);

    let data = Supplier::insert(input_data.data(), session.clone(), &db.0).await?;
    let converted: Convert<Supplier> = Supplier::fetch_by_id(&data.last_insert_id, session, &db.0)
        .await
        .into();
    converted.0
}
