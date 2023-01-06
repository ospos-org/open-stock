use rocket::{routes, patch, get, http::{CookieJar, Status}, serde::json::Json, put, post};
use sea_orm_rocket::{Connection};
use serde::{Deserialize, Serialize};

use crate::{pool::Db, methods::{cookie_status_wrapper, Action, Employee, Store, Product, Customer}};

pub fn routes() -> Vec<rocket::Route> {
    routes![generate_template]
}

#[derive(Serialize, Deserialize)]
struct All {
    employee: Employee,
    store: Store,
    product: Product,
    customer: Customer
}

#[patch("/generate")]
async fn generate_template(conn: Connection<'_, Db>, cookies: &CookieJar<'_>) -> Result<Json<All>, Status> {
    let db = conn.into_inner();
    let _session = cookie_status_wrapper(db, cookies).await?;

    let employee = Employee::generate(db).await.unwrap();
    let store = Store::generate(db).await.unwrap();
    let product = Product::generate(db).await.unwrap();
    let customer = Customer::generate(db).await.unwrap();

    Ok(rocket::serde::json::Json(All {
        employee,
        store,
        product,
        customer
    }))
}