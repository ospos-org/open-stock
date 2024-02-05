use crate::catchers::Validated;
use crate::methods::{Error};
use crate::pool::{InternalDb};
use crate::{AuthenticationLog, Kiosk, KioskInit, KioskPreferences, Session};
use okapi::openapi3::OpenApi;
use rocket::serde::json::Json;
use rocket::{get, post};
use rocket_okapi::settings::OpenApiSettings;
use rocket_okapi::{openapi, openapi_get_routes_spec};
use crate::{
    check_permissions,
    methods::{Action},
};
use crate::guards::Convert;

pub fn documented_routes(settings: &OpenApiSettings) -> (Vec<rocket::Route>, OpenApi) {
    openapi_get_routes_spec![
        settings:
        get,
        initialize,
        update,
        update_preferences,
        update_online_status,
        delete,
        auth_log
    ]
}

#[openapi(tag = "Kiosk")]
#[get("/<id>")]
pub async fn get(db: InternalDb, id: &str, session: Session) -> Convert<Kiosk> {
    check_permissions!(session.clone(), Action::FetchKiosk);
    Kiosk::fetch_by_id(id, session, &db.0).await.into()
}

#[openapi(tag = "Kiosk")]
#[post("/", data = "<input_data>")]
pub async fn initialize(
    db: InternalDb,
    input_data: Validated<Json<KioskInit>>,
    session: Session,
) -> Result<Json<Kiosk>, Error> {
    check_permissions!(session.clone(), Action::AccessAdminPanel);

    let kiosk = Kiosk::insert(input_data.data(), session.clone(), &db.0, None).await?;
    Kiosk::fetch_by_id(&kiosk.last_insert_id, session, &db.0).await.map(|v| Json(v))
}

#[openapi(tag = "Kiosk")]
#[post("/<id>", data = "<input_data>")]
pub async fn update(
    conn: InternalDb,
    id: &str,
    input_data: Validated<Json<KioskInit>>,
    session: Session,
) -> Convert<Kiosk> {
    check_permissions!(session.clone(), Action::AccessAdminPanel);

    Kiosk::update(input_data.data(), session, id, &conn.0).await.into()
}

#[openapi(tag = "Kiosk")]
#[post("/preferences/<id>", data = "<input_data>")]
pub async fn update_preferences(
    db: InternalDb,
    id: &str,
    input_data: Validated<Json<KioskPreferences>>,
    session: Session,
) -> Convert<Kiosk> {
    check_permissions!(session.clone(), Action::ModifyKioskPreferences);
    Kiosk::update_preferences(id, session, input_data.data(), &db.0).await.into()
}

#[openapi(tag = "Kiosk")]
#[post("/online/<id>")]
pub async fn update_online_status(
    db: InternalDb,
    id: &str,
    session: Session,
) -> Convert<Kiosk> {
    check_permissions!(session.clone(), Action::FetchKiosk);
    Kiosk::update_online_to_now(id, session, &db.0).await.into()
}

#[openapi(tag = "Kiosk")]
#[post("/delete/<id>")]
pub async fn delete(db: InternalDb, id: &str, session: Session) -> Result<(), Error> {
    check_permissions!(session.clone(), Action::AccessAdminPanel);
    Kiosk::delete(id, session, &db.0).await.map(|_| ())
}

#[openapi(tag = "Kiosk")]
#[post("/log/<id>", data = "<input_data>")]
pub async fn auth_log(
    db: InternalDb,
    session: Session,
    id: &str,
    input_data: Validated<Json<AuthenticationLog>>,
) -> Result<(), Error> {
    check_permissions!(session.clone(), Action::AccessAdminPanel);
    Kiosk::auth_log(id, session, input_data.data(), &db.0).await.map(|_| ())
}
