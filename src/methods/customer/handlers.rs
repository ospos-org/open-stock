use rocket::{http::Status, get, put, patch};
use rocket::{routes, post};
use rocket::serde::json::Json;
use sea_orm_rocket::{Connection};
use serde_json::json;
use crate::pool::Db;

use super::{Customer, CustomerInput};

pub fn routes() -> Vec<rocket::Route> {
    routes![get, get_by_name, get_by_phone, get_by_addr, create, update, generate, search_query]
}

#[get("/<id>")]
pub async fn get(conn: Connection<'_, Db>, id: &str) -> Result<Json<Customer>, Status> {
    let db = conn.into_inner();

    let customer = Customer::fetch_by_id(&id, db).await.unwrap();
    Ok(Json(customer))
}

#[get("/name/<name>")]
pub async fn get_by_name(conn: Connection<'_, Db>, name: &str) -> Result<Json<Vec<Customer>>, Status> {
    let db = conn.into_inner();

    let employee = Customer::fetch_by_name(name, db).await.unwrap();
    Ok(Json(employee))
}

/// Will search by both name, phone and email.
#[get("/search/<query>")]
pub async fn search_query(conn: Connection<'_, Db>, query: &str) -> Result<Json<Vec<Customer>>, Status> {
    let db = conn.into_inner();

    let employee = Customer::search(query, db).await.unwrap();
    Ok(Json(employee))
}

#[get("/phone/<phone>")]
pub async fn get_by_phone(conn: Connection<'_, Db>, phone: &str) -> Result<Json<Vec<Customer>>, Status> {
    let db = conn.into_inner();
    let new_transaction = phone.clone();

    println!("{}", json!(new_transaction));

    let employee = Customer::fetch_by_phone(new_transaction, db).await.unwrap();
    Ok(Json(employee))
}

#[get("/addr/<addr>")]
pub async fn get_by_addr(conn: Connection<'_, Db>, addr: &str) -> Result<Json<Vec<Customer>>, Status> {
    let db = conn.into_inner();
    let new_transaction = addr.clone();

    println!("{}", json!(new_transaction));

    let employee = Customer::fetch_by_addr(new_transaction, db).await.unwrap();
    Ok(Json(employee))
}

#[patch("/generate")]
async fn generate(
    conn: Connection<'_, Db>
) -> Result<Json<Customer>, Status> {
    let db = conn.into_inner();

    match Customer::generate(db).await {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(Status::BadRequest)
    }
}

#[put("/<id>", data = "<input_data>")]
async fn update(
    conn: Connection<'_, Db>,
    id: &str,
    input_data: Json<CustomerInput>,
) -> Result<Json<Customer>, Status> {
    let input_data = input_data.clone().into_inner();
    let db = conn.into_inner();

    match Customer::update(input_data, id, db).await {
        Ok(res) => {
            Ok(Json(res))
        },
        Err(_) => Err(Status::BadRequest),
    }
}

#[post("/", data = "<input_data>")]
pub async fn create(conn: Connection<'_, Db>, input_data: Json<CustomerInput>) -> Result<Json<Customer>, Status> {
    let new_transaction = input_data.clone().into_inner();
    let db = conn.into_inner();

    match Customer::insert(new_transaction, db).await {
        Ok(data) => {
            match Customer::fetch_by_id(&data.last_insert_id, db).await {
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