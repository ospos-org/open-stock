use pool::Db;
use rocket::{*, fairing::{Fairing, Info, Kind}, http::Header};
use sea_orm_rocket::Database;

pub mod methods;
pub mod entities;
pub mod pool;

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
        let access_origin = dotenv::var("ACCESS_ORIGIN").unwrap();

        response.set_header(Header::new("Access-Control-Allow-Origin", access_origin));
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
        .mount("/api/product", methods::product::handlers::routes())
        .mount("/api/customer", methods::customer::handlers::routes())
        .mount("/api/employee", methods::employee::handlers::routes())
        .mount("/api/transaction", methods::transaction::handlers::routes())
        .mount("/api/supplier", methods::supplier::handlers::routes())
        .mount("/api/store", methods::store::handlers::routes())
        .mount("/api/helpers", methods::helpers::handlers::routes())
}