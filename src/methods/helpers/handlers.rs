use rocket::{routes, patch, http::{Status}, serde::json::Json};
use sea_orm_rocket::{Connection};
use serde::{Deserialize, Serialize};

use crate::{pool::Db, methods::{Employee, Store, Product, Customer}};

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

/// This route does not require authentication, but is not enabled in release mode.
#[patch("/generate")]
async fn generate_template(conn: Connection<'_, Db>) -> Result<Json<All>, Status> {
    let db = conn.into_inner();

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