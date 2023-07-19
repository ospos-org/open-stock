// Kiosk
// - [x] Get
// - [ ] Initialize
// - [ ] Edit All
// - [ ] Delete
// - [ ] Append Auth Log
// - [ ] Edit Preference (Lower Security Requirement)
// - [ ] Call from `auth_rid` to submit auth request

use crate::methods::{Error, ErrorResponse};
use crate::pool::Db;
use crate::Kiosk;
use rocket::get;
use rocket::http::CookieJar;
use rocket::serde::json::Json;
use sea_orm_rocket::Connection;

use crate::{
    check_permissions,
    methods::{cookie_status_wrapper, Action},
};

#[get("/<id>")]
pub async fn get(
    conn: Connection<'_, Db>,
    id: &str,
    cookies: &CookieJar<'_>,
) -> Result<Json<Kiosk>, Error> {
    let db = conn.into_inner();

    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session.clone(), Action::FetchEmployee);

    match Kiosk::fetch_by_id(id, db).await {
        Ok(employee) => Ok(Json(employee)),
        Err(err) => Err(ErrorResponse::db_err(err)),
    }
}
