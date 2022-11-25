use rocket::{http::Status, get};
use rocket::{routes, post};
use rocket::serde::json::Json;
use sea_orm_rocket::{Connection};
use crate::pool::Db;

use super::Product;

pub fn routes() -> Vec<rocket::Route> {
    routes![get, get_by_name, get_by_name_exact, create]
}

#[get("/<id>")]
pub async fn get(conn: Connection<'_, Db>, id: i32) -> Result<Json<Product>, Status> {
    let db = conn.into_inner();

    let product = Product::fetch_by_id(&id.to_string(), db).await.unwrap();
    Ok(Json(product))
}

#[get("/name/<name>")]
pub async fn get_by_name(conn: Connection<'_, Db>, name: &str) -> Result<Json<Vec<Product>>, Status> {
    let db = conn.into_inner();

    let product = Product::fetch_by_name(name, db).await.unwrap();
    Ok(Json(product))
}

/// References exact name
#[get("/!name/<name>")]
pub async fn get_by_name_exact(conn: Connection<'_, Db>, name: &str) -> Result<Json<Vec<Product>>, Status> {
    let db = conn.into_inner();

    let product = Product::fetch_by_name_exact(name, db).await.unwrap();
    Ok(Json(product))
}

#[post("/", data = "<input_data>")]
pub async fn create(conn: Connection<'_, Db>, input_data: Json<Product>) -> Result<Json<Product>, Status> {
    let new_transaction = input_data.clone().into_inner();
    let db = conn.into_inner();

    match Product::insert(new_transaction, db).await {
        Ok(data) => {
            match Product::fetch_by_id(&data.last_insert_id, db).await {
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