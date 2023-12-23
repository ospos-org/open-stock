use okapi::openapi3::OpenApi;
use rocket::{get, http::CookieJar, post, serde::json::Json};
use rocket_db_pools::Connection;
use rocket_okapi::{openapi, openapi_get_routes_spec};
use rocket_okapi::settings::OpenApiSettings;
use crate::catchers::Validated;

use crate::{
    check_permissions,
    methods::{cookie_status_wrapper, Action, Error, ErrorResponse},
    pool::Db,
};

use super::Store;

pub fn documented_routes(settings: &OpenApiSettings) -> (Vec<rocket::Route>, OpenApi) {
    openapi_get_routes_spec![
        settings: get, get_all, get_by_code, generate, update
    ]
}

#[openapi(tag = "Store")]
#[get("/")]
pub async fn get_all(
    conn: Connection<Db>,
    cookies: &CookieJar<'_>,
) -> Result<Json<Vec<Store>>, Error> {
    let db = conn.into_inner();

    let session = cookie_status_wrapper(&db, cookies).await?;
    check_permissions!(session.clone(), Action::FetchStore);

    match Store::fetch_all(session, &db).await {
        Ok(stores) => Ok(Json(stores)),
        Err(reason) => Err(ErrorResponse::db_err(reason)),
    }
}

#[openapi(tag = "Store")]
#[get("/<id>")]
pub async fn get(
    conn: Connection<Db>,
    id: &str,
    cookies: &CookieJar<'_>,
) -> Result<Json<Store>, Error> {
    let db = conn.into_inner();

    let session = cookie_status_wrapper(&db, cookies).await?;
    check_permissions!(session.clone(), Action::FetchStore);

    match Store::fetch_by_id(id, session, &db).await {
        Ok(store) => Ok(Json(store)),
        Err(reason) => Err(ErrorResponse::db_err(reason)),
    }
}

#[openapi(tag = "Store")]
#[get("/code/<code>")]
pub async fn get_by_code(
    conn: Connection<Db>,
    code: &str,
    cookies: &CookieJar<'_>,
) -> Result<Json<Store>, Error> {
    let db = conn.into_inner();

    let session = cookie_status_wrapper(&db, cookies).await?;
    check_permissions!(session.clone(), Action::FetchStore);

    match Store::fetch_by_code(code, session, &db).await {
        Ok(store) => Ok(Json(store)),
        Err(reason) => Err(ErrorResponse::db_err(reason)),
    }
}

#[openapi(tag = "Store")]
#[post("/generate")]
async fn generate(
    conn: Connection<Db>,
    cookies: &CookieJar<'_>,
) -> Result<Json<Vec<Store>>, Error> {
    let db = conn.into_inner();

    let session = cookie_status_wrapper(&db, cookies).await?;
    check_permissions!(session.clone(), Action::GenerateTemplateContent);

    match Store::generate(session, &db).await {
        Ok(res) => Ok(Json(res)),
        Err(err) => Err(ErrorResponse::db_err(err)),
    }
}

#[openapi(tag = "Store")]
#[post("/<id>", data = "<input_data>")]
async fn update(
    conn: Connection<Db>,
    id: &str,
    cookies: &CookieJar<'_>,
    input_data: Validated<Json<Store>>,
) -> Result<Json<Store>, Error> {
    let input_data = input_data.clone().0.into_inner();
    let db = conn.into_inner();

    let session = cookie_status_wrapper(&db, cookies).await?;
    check_permissions!(session.clone(), Action::ModifyStore);

    if session
        .clone()
        .employee
        .level
        .into_iter()
        .find(|x| x.action == Action::ModifyStore)
        .unwrap()
        .authority
        >= 1
    {
        match Store::update(input_data, session, id, &db).await {
            Ok(res) => Ok(Json(res)),
            Err(reason) => Err(ErrorResponse::db_err(reason)),
        }
    } else {
        Err(ErrorResponse::unauthorized(Action::ModifyStore))
    }
}
