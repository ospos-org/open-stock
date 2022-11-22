use pool::Db;
use rocket::{*};
use sea_orm_rocket::{Database};

mod methods;
mod entities;
mod pool;

#[get("/")]
async fn index() -> &'static str {
    "Hello, bakeries!"
}

#[launch] // The "main" function of the program
fn rocket() -> _ {
    rocket::build()
        .attach(Db::init())  
        .mount("/product", methods::product::handlers::routes())  
        .mount("/", routes![index])
}