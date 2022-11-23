use rocket::{http::Status, get};
use rocket::routes;
use rocket::serde::json::Json;
use sea_orm_rocket::{Connection};
use crate::pool::Db;

use super::Customer;

pub fn routes() -> Vec<rocket::Route> {
    routes![get]
}

#[get("/<id>")]
pub async fn get(conn: Connection<'_, Db>, id: i32) -> Result<Json<Customer>, Status> {
    let db = conn.into_inner();

    let customer = Customer::fetch_by_id(&id.to_string(), db).await.unwrap();
    Ok(Json(customer))
}
