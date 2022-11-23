use rocket::{http::Status, get};
use rocket::{routes, post};
use rocket::serde::json::Json;
use sea_orm_rocket::{Connection};
use serde_json::json;
use crate::methods::Name;
use crate::pool::Db;

use super::{Employee, EmployeeInput};

pub fn routes() -> Vec<rocket::Route> {
    routes![get, get_by_name, get_by_name_exact, get_by_level, create]
}

#[get("/<id>")]
pub async fn get(conn: Connection<'_, Db>, id: &str) -> Result<Json<Employee>, Status> {
    let db = conn.into_inner();

    let employee = Employee::fetch_by_id(&id.to_string(), db).await.unwrap();
    Ok(Json(employee))
}

#[get("/name/<name>")]
pub async fn get_by_name(conn: Connection<'_, Db>, name: &str) -> Result<Json<Vec<Employee>>, Status> {
    let db = conn.into_inner();

    let employee = Employee::fetch_by_name(name, db).await.unwrap();
    Ok(Json(employee))
}

#[get("/!name", data = "<name>")]
pub async fn get_by_name_exact(conn: Connection<'_, Db>, name: Json<Name>) -> Result<Json<Vec<Employee>>, Status> {
    let db = conn.into_inner();
    let new_transaction = name.clone().into_inner();

    println!("{}", json!(new_transaction));

    let employee = Employee::fetch_by_name_exact(json!(new_transaction), db).await.unwrap();
    Ok(Json(employee))
}

#[get("/level/<level>")]
pub async fn get_by_level(conn: Connection<'_, Db>, level: i32) -> Result<Json<Vec<Employee>>, Status> {
    let db = conn.into_inner();
    let new_transaction = level.clone();

    println!("{}", json!(new_transaction));

    let employee = Employee::fetch_by_level(new_transaction, db).await.unwrap();
    Ok(Json(employee))
}

#[post("/", data = "<input_data>")]
pub async fn create(conn: Connection<'_, Db>, input_data: Json<EmployeeInput>) -> Result<Json<Employee>, Status> {
    let new_transaction = input_data.clone().into_inner();
    let db = conn.into_inner();

    match Employee::insert(new_transaction, db).await {
        Ok(data) => {
            match Employee::fetch_by_id(&data.last_insert_id, db).await {
                Ok(res) => {
                    Ok(Json(res))
                },  
                Err(reason) => {
                    println!("[dberr]: {}", reason);
                    Err(Status::InternalServerError)
                }
            }
        },
        Err(reason) => {
            println!("[dberr]: {}", reason);
            Err(Status::InternalServerError)
        },
    }
}