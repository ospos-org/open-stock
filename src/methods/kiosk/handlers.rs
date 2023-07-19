// Kiosk
// -> Get
// -> Initialize
// -> Edit All
// -> Delete
// -> Append Auth Log
// -> Edit Preference (Lower Security Requirement)
// -> Call from `auth_rid` to submit auth request

use crate::Kiosk;

// #[get("/<id>")]
// pub async fn get(
//     conn: Connection<'_, Db>,
//     id: &str,
//     cookies: &CookieJar<'_>,
// ) -> Result<Json<Kiosk>, Error> {
//     let db = conn.into_inner();

//     let session = cookie_status_wrapper(db, cookies).await?;
//     check_permissions!(session.clone(), Action::FetchEmployee);

//     if session.employee.id == id {
//         Ok(Json(session.employee))
//     } else {
//         match Employee::fetch_by_id(id, db).await {
//             Ok(employee) => Ok(Json(employee)),
//             Err(err) => Err(ErrorResponse::db_err(err)),
//         }
//     }
// }
