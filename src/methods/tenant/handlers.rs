use rocket::get;
use rocket::routes;
use rocket::serde::json::Json;
use sea_orm_rocket::Connection;

use crate::methods::Error;
use crate::{Db, ErrorResponse, Tenant};

pub fn routes() -> Vec<rocket::Route> {
    routes![get]
}

#[get("/<id>")]
pub async fn get(conn: Connection<'_, Db>, id: String) -> Result<Json<Tenant>, Error> {
    let db = conn.into_inner();

    match Tenant::fetch_by_id(&id, db).await {
        Ok(tenant) => Ok(Json(tenant)),
        Err(reason) => Err(ErrorResponse::db_err(reason)),
    }
}
