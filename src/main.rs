#![allow(ambiguous_glob_reexports)]

use std::env;
#[cfg(feature = "sql")]
use pool::Db;
#[cfg(feature = "sql")]
use rocket::{
    fairing::{Fairing, Info, Kind},
    http::Header,
    *,
};
use rocket::http::uri::Host;
#[cfg(feature = "process")]
use sea_orm_rocket::Database;

#[cfg(feature = "process")]
pub mod entities;
pub mod methods;
#[cfg(feature = "sql")]
pub mod migrator;
#[cfg(feature = "process")]
pub mod pool;

#[cfg(feature = "sql")]
pub use entities::*;
pub use methods::*;
#[cfg(feature = "sql")]
pub use migrator::*;

#[cfg(feature = "sql")]
extern crate argon2;
#[cfg(feature = "sql")]
extern crate futures_cpupool;

pub struct CORS;

#[cfg(feature = "sql")]
#[rocket::async_trait]
impl Fairing for CORS {
    fn info(&self) -> Info {
        Info {
            name: "Add CORS headers to responses",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, request: &'r Request<'_>, response: &mut Response<'r>) {
        let access_origin = dotenv::var("ACCESS_ORIGIN").unwrap();

        // Permit `localhost:3000` when DEMO mode is enabled.
        if request.host().unwrap().domain().eq( &access_origin) {
            response.set_header(Header::new("Access-Control-Allow-Origin", access_origin));
        } else if request.host().unwrap().domain().eq("localhost") && !(
            env::var("DEMO").is_err() || env::var("DEMO").unwrap() == "0"
        ) {
            response.set_header(Header::new("Access-Control-Allow-Origin", "localhost:3000"));
        }

        response.set_header(Header::new(
            "Access-Control-Allow-Methods",
            "POST, GET, PATCH, OPTIONS",
        ));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}

#[cfg(feature = "process")]
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
        .mount("/api/ingress", methods::ingress::handlers::routes())
        .mount("/api/supplier", methods::supplier::handlers::routes())
        .mount("/api/store", methods::store::handlers::routes())
        .mount("/api/helpers", methods::helpers::handlers::routes())
}

#[cfg(not(feature = "process"))]
fn main() {}
