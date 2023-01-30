use rocket::{routes, get, http::{CookieJar}, serde::json::Json, post};
use sea_orm_rocket::{Connection};

use crate::{pool::Db, methods::{cookie_status_wrapper, Action, Error, ErrorResponse}, check_permissions};

use super::{Store};

pub fn routes() -> Vec<rocket::Route> {
    routes![get, get_all, get_by_code, generate, update]
}

#[get("/")]
pub async fn get_all(conn: Connection<'_, Db>, cookies: &CookieJar<'_>) -> Result<Json<Vec<Store>>, Error> {
    let db = conn.into_inner();

    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session.clone(), Action::FetchStore);

    let stores = Store::fetch_all(db).await.unwrap();
    Ok(Json(stores))
}

#[get("/<id>")]
pub async fn get(conn: Connection<'_, Db>, id: &str, cookies: &CookieJar<'_>) -> Result<Json<Store>, Error> {
    let db = conn.into_inner();

    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session.clone(), Action::FetchStore);

    let store = Store::fetch_by_id(&id.to_string(), db).await.unwrap();
    Ok(Json(store))
}

#[get("/code/<code>")]
pub async fn get_by_code(conn: Connection<'_, Db>, code: &str, cookies: &CookieJar<'_>) -> Result<Json<Store>, Error> {
    let db = conn.into_inner();

    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session.clone(), Action::FetchStore);

    let store = Store::fetch_by_code(&code.to_string(), db).await.unwrap();
    Ok(Json(store))
}

#[post("/generate")]
async fn generate(
    conn: Connection<'_, Db>,
    cookies: &CookieJar<'_>
) -> Result<Json<Vec<Store>>, Error> {
    let db = conn.into_inner();

    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session.clone(), Action::GenerateTemplateContent);

    match Store::generate(db).await {
        Ok(res) => Ok(Json(res)),
        Err(err) => Err(ErrorResponse::db_err(err))
    }
}

#[post("/<id>", data = "<input_data>")]
async fn update(
    conn: Connection<'_, Db>,
    id: &str,
    cookies: &CookieJar<'_>,
    input_data: Json<Store>,
) -> Result<Json<Store>, Error> {
    let input_data = input_data.clone().into_inner();
    let db = conn.into_inner();

    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session.clone(), Action::ModifyStore);

    if session.employee.level.into_iter().find(| x | x.action == Action::ModifyStore).unwrap().authority >= 1 {
        match Store::update(input_data, id, db).await {
            Ok(res) => {
                Ok(Json(res))
            },
            Err(reason) => Err(ErrorResponse::db_err(reason)),
        }
    }else {
        Err(ErrorResponse::unauthorized(Action::ModifyStore))
    }
}