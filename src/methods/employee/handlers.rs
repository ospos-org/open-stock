use rocket::{http::Status, get};
use rocket::routes;
use rocket::serde::json::Json;
use sea_orm_rocket::{Connection, Database};
use crate::pool::Db;

use super::Employee;

pub fn routes() -> Vec<rocket::Route> {
    routes![get]
}

#[get("/<id>")]
pub async fn get(conn: Connection<'_, Db>, id: i32) -> Result<Json<Employee>, Status> {
    let db = conn.into_inner();

    let employee = Employee::fetch_by_id(&id.to_string(), db).await.unwrap();
    Ok(Json(employee))
}