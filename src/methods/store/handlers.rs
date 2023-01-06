use rocket::{routes, patch, get, http::{CookieJar, Status}, serde::json::Json, put};
use sea_orm_rocket::{Connection};

use crate::{pool::Db, methods::{cookie_status_wrapper, Action}};

use super::{Store};

pub fn routes() -> Vec<rocket::Route> {
    routes![get, get_by_code, generate, update]
}

#[get("/<id>")]
pub async fn get(conn: Connection<'_, Db>, id: &str, cookies: &CookieJar<'_>) -> Result<Json<Store>, Status> {
    let db = conn.into_inner();
    let _session = cookie_status_wrapper(db, cookies).await?;

    let store = Store::fetch_by_id(&id.to_string(), db).await.unwrap();
    Ok(Json(store))
}

#[get("/code/<code>")]
pub async fn get_by_code(conn: Connection<'_, Db>, code: &str, cookies: &CookieJar<'_>) -> Result<Json<Store>, Status> {
    let db = conn.into_inner();
    let _session = cookie_status_wrapper(db, cookies).await?;

    let store = Store::fetch_by_code(&code.to_string(), db).await.unwrap();
    Ok(Json(store))
}

#[patch("/generate")]
async fn generate(
    conn: Connection<'_, Db>,
    _cookies: &CookieJar<'_>
) -> Result<Json<Store>, Status> {
    let db = conn.into_inner();

    match Store::generate(db).await {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(Status::BadRequest)
    }
}

#[put("/<id>", data = "<input_data>")]
async fn update(
    conn: Connection<'_, Db>,
    id: &str,
    cookies: &CookieJar<'_>,
    input_data: Json<Store>,
) -> Result<Json<Store>, Status> {
    let input_data = input_data.clone().into_inner();
    let db = conn.into_inner();
    let session = cookie_status_wrapper(db, cookies).await?;

    if session.employee.level.into_iter().find(| x | x.action == Action::ModifyStore).unwrap().authority >= 1 {
        match Store::update(input_data, id, db).await {
            Ok(res) => {
                Ok(Json(res))
            },
            Err(_) => Err(Status::BadRequest),
        }
    }else {
        Err(Status::Unauthorized)
    }
}