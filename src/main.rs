use methods::Product;
use pool::Db;
use rocket::{*, http::{Status}, serde::json::Json};
use sea_orm_rocket::{Connection, Database};

mod methods;
mod entities;
mod pool;

#[get("/product/<id>")]
async fn get_product(conn: Connection<'_, Db>, id: i32) -> Result<Json<Product>, Status> {
    let db = conn.into_inner();

    let product = Product::fetch_by_id(&id.to_string(), db).await.unwrap();
    Ok(Json(product))
}

#[get("/")]
async fn index() -> &'static str {
    "Hello, bakeries!"
}

#[launch] // The "main" function of the program
fn rocket() -> _ {
    rocket::build()
        .attach(Db::init())    
        .mount("/", routes![index])
}