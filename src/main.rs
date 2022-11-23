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
        .mount("/customer", methods::customer::handlers::routes()) 
        .mount("/employee", methods::employee::handlers::routes())  
        .mount("/transaction", methods::transaction::handlers::routes())  
        .mount("/", routes![index])
}