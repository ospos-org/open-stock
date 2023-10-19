use crate::methods::{Error, ErrorResponse};
use crate::pool::Db;
use crate::{AuthenticationLog, Kiosk, KioskInit, KioskPreferences};
use rocket::http::CookieJar;
use rocket::serde::json::Json;
use rocket::{get, post, routes};
use rocket_db_pools::Connection;

use crate::{
    check_permissions,
    methods::{cookie_status_wrapper, Action},
};

pub fn routes() -> Vec<rocket::Route> {
    routes![
        get,
        initialize,
        update,
        update_preferences,
        update_online_status,
        delete,
        auth_log
    ]
}

#[get("/<id>")]
pub async fn get(
    conn: Connection<Db>,
    id: &str,
    cookies: &CookieJar<'_>,
) -> Result<Json<Kiosk>, Error> {
    let db = conn.into_inner();

    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session.clone(), Action::FetchEmployee);

    match Kiosk::fetch_by_id(id, session, db).await {
        Ok(employee) => Ok(Json(employee)),
        Err(err) => Err(ErrorResponse::db_err(err)),
    }
}

#[post("/", data = "<input_data>")]
pub async fn initialize(
    conn: Connection<Db>,
    input_data: Json<KioskInit>,
    cookies: &CookieJar<'_>,
) -> Result<Json<Kiosk>, Error> {
    let input_data = input_data.clone().into_inner();
    let db = conn.into_inner();

    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session.clone(), Action::AccessAdminPanel);

    match Kiosk::insert(input_data, session.clone(), db, None).await {
        Ok(kiosk) => match Kiosk::fetch_by_id(&kiosk.last_insert_id, session, db).await {
            Ok(res) => Ok(Json(res)),
            Err(reason) => Err(ErrorResponse::db_err(reason)),
        },
        Err(err) => Err(ErrorResponse::db_err(err)),
    }
}

#[post("/<id>", data = "<input_data>")]
pub async fn update(
    conn: Connection<Db>,
    id: &str,
    input_data: Json<KioskInit>,
    cookies: &CookieJar<'_>,
) -> Result<Json<Kiosk>, Error> {
    let input_data = input_data.clone().into_inner();
    let db = conn.into_inner();

    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session.clone(), Action::AccessAdminPanel);

    match Kiosk::update(input_data, session, id, db).await {
        Ok(kiosk) => Ok(Json(kiosk)),
        Err(err) => Err(ErrorResponse::db_err(err)),
    }
}

#[post("/preferences/<id>", data = "<input_data>")]
pub async fn update_preferences(
    conn: Connection<Db>,
    id: &str,
    input_data: Json<KioskPreferences>,
    cookies: &CookieJar<'_>,
) -> Result<Json<Kiosk>, Error> {
    let input_data = input_data.clone().into_inner();
    let db = conn.into_inner();

    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session.clone(), Action::ModifyKioskPreferences);

    match Kiosk::update_preferences(id, session, input_data, db).await {
        Ok(kiosk) => Ok(Json(kiosk)),
        Err(err) => Err(ErrorResponse::db_err(err)),
    }
}

#[post("/online/<id>")]
pub async fn update_online_status(
    conn: Connection<Db>,
    id: &str,
    cookies: &CookieJar<'_>,
) -> Result<Json<Kiosk>, Error> {
    let db = conn.into_inner();

    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session.clone(), Action::FetchKiosk);

    match Kiosk::update_online_to_now(id, session, db).await {
        Ok(kiosk) => Ok(Json(kiosk)),
        Err(err) => Err(ErrorResponse::db_err(err)),
    }
}

#[post("/delete/<id>")]
pub async fn delete(
    conn: Connection<Db>,
    id: &str,
    cookies: &CookieJar<'_>,
) -> Result<(), Error> {
    let db = conn.into_inner();

    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session.clone(), Action::AccessAdminPanel);

    match Kiosk::delete(id, session, db).await {
        Ok(_res) => Ok(()),
        Err(err) => Err(ErrorResponse::db_err(err)),
    }
}

#[post("/log/<id>", data = "<input_data>")]
pub async fn auth_log(
    conn: Connection<Db>,
    id: &str,
    input_data: Json<AuthenticationLog>,
    cookies: &CookieJar<'_>,
) -> Result<(), Error> {
    let input_data = input_data.clone().into_inner();
    let db = conn.into_inner();

    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session.clone(), Action::AccessAdminPanel);

    match Kiosk::auth_log(id, session, input_data, db).await {
        Ok(_res) => Ok(()),
        Err(err) => Err(ErrorResponse::db_err(err)),
    }
}
