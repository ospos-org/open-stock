#![allow(ambiguous_glob_reexports)]

#[cfg(feature = "sql")]
use pool::Db;
#[cfg(feature = "sql")]
use rocket::{
    fairing::{Fairing, Info, Kind},
    http::Header,
    *,
};
use rocket_db_pools::Database;
use rocket_okapi::mount_endpoints_and_merged_docs;
use rocket_okapi::swagger_ui::{make_swagger_ui, SwaggerUIConfig};
#[cfg(feature = "process")]

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

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        let access_origin = dotenv::var("ACCESS_ORIGIN").unwrap();

        // Permit `localhost:3000` when DEMO mode is enabled.
        // `request.host().unwrap().domain().eq( &access_origin)`

        response.set_header(Header::new("Access-Control-Allow-Origin", access_origin));
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

    // All non-documented items attached here.
    let mut launcher = build()
        .attach(Db::init())
        .attach(CORS)
        .mount(
            "/docs",
            make_swagger_ui(&SwaggerUIConfig {
                url: "../openapi.json".to_owned(),
                ..Default::default()
            }),
        );

    let openapi_settings = rocket_okapi::settings::OpenApiSettings::default();

    mount_endpoints_and_merged_docs! {
        launcher, "/".to_owned(), openapi_settings,
        "/api/store" => methods::store::handlers::documented_routes(&openapi_settings),
        "/api/kiosk" => methods::kiosk::handlers::documented_routes(&openapi_settings),
        "/api/ingress" => methods::ingress::handlers::documented_routes(&openapi_settings),
        "/api/product" => methods::product::handlers::documented_routes(&openapi_settings),
        "/api/customer" => methods::customer::handlers::documented_routes(&openapi_settings),
        "/api/employee" => methods::employee::handlers::documented_routes(&openapi_settings),
        "/api/supplier" => methods::supplier::handlers::documented_routes(&openapi_settings),
        "/api/helpers" => methods::transaction::handlers::documented_routes(&openapi_settings),
        "/api/transaction" => methods::transaction::handlers::documented_routes(&openapi_settings),
    }

    launcher
}

#[cfg(not(feature = "process"))]
fn main() {}
