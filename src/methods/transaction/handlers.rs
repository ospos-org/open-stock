use std::time::Instant;

use rocket::{http::Status, get};
use rocket::{routes, post};
use rocket::serde::json::Json;
use sea_orm_rocket::{Connection};

use crate::pool::Db;
use super::{Transaction, TransactionInput};


pub fn routes() -> Vec<rocket::Route> {
    routes![get, get_by_name, create]
}

#[get("/<id>")]
pub async fn get(conn: Connection<'_, Db>, id: String) -> Result<Json<Transaction>, Status> {
    let db = conn.into_inner();

    let transaction = Transaction::fetch_by_id(&id, db).await.unwrap();
    Ok(Json(transaction))
}

#[get("/ref/<name>")]
pub async fn get_by_name(conn: Connection<'_, Db>, name: &str) -> Result<Json<Vec<Transaction>>, Status> {
    let db = conn.into_inner();

    let transaction = Transaction::fetch_by_ref(name, db).await.unwrap();
    Ok(Json(transaction))
}


#[post("/", data = "<input_data>")]
pub async fn create(conn: Connection<'_, Db>, input_data: Json<TransactionInput>) -> Result<Json<Transaction>, Status> {
    let new_transaction = input_data.clone().into_inner();
    let db = conn.into_inner();

    match Transaction::insert(new_transaction, db).await {
        Ok(data) => {
            match Transaction::fetch_by_id(&data.last_insert_id, db).await {
                Ok(res) => {
                    Ok(Json(res))
                },  
                Err(_) => Err(Status::InternalServerError)
            }
        },
        Err(_) => Err(Status::InternalServerError),
    }
}