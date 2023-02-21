use pool::Db;
use rocket::{*, fairing::{Fairing, Info, Kind}, http::Header};
use sea_orm_rocket::{Database};

pub(crate) mod methods;
pub(crate) mod entities;
pub(crate) mod pool;

extern crate argon2;
extern crate futures_cpupool;

pub struct CORS;

#[rocket::async_trait]
impl Fairing for CORS {
    fn info(&self) -> Info {
        Info {
            name: "Add CORS headers to responses",
            kind: Kind::Response
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "https://open-retail.bennjii.dev"));
        response.set_header(Header::new("Access-Control-Allow-Methods", "POST, GET, PATCH, OPTIONS"));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}

#[launch] // The "main" function of the program
fn rocket() -> _ {
    dotenv::dotenv().ok();

    rocket::build()
        .attach(Db::init())
        .attach(CORS)
        .mount("/product", methods::product::handlers::routes())  
        .mount("/customer", methods::customer::handlers::routes()) 
        .mount("/employee", methods::employee::handlers::routes())  
        .mount("/transaction", methods::transaction::handlers::routes())  
        .mount("/supplier", methods::supplier::handlers::routes())  
        .mount("/store", methods::store::handlers::routes())
        .mount("/helpers", methods::helpers::handlers::routes())
}