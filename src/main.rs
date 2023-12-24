#![allow(ambiguous_glob_reexports)]

#[cfg(feature = "sql")]
use pool::Db;
#[cfg(feature = "sql")]
use rocket::{
    fairing::{Fairing, Info, Kind},
    http::Header,
    *,
};
use rocket::http::{Method, Status};
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
use open_stock::{catchers};

#[cfg(feature = "sql")]
extern crate argon2;

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

        response.set_header(Header::new("Access-Control-Allow-Origin", access_origin));
        response.set_header(Header::new(
            "Access-Control-Allow-Methods",
            "POST, GET, PATCH, OPTIONS",
        ));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_header(Header::new("Access-Control-Expose-Headers", "*"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));

        if request.method() == Method::Options {
            response.set_status(Status::Ok);
            response.set_header(Header::new("Access-Control-Allow-Headers", "Content-Type, Referer, *"));
        }
    }
}

#[cfg(feature = "process")]
#[launch] // The "main" function of the program
fn rocket() -> _ {
    dotenv::dotenv().ok();

    // All non-documented items attached here.
    let mut launcher = build()
        .register("/", catchers![
            catchers::not_authorized,
            catchers::internal_server_error,
            catchers::not_found,
            catchers::unprocessable_entry,
            catchers::general_catcher,
        ])
        .attach(Db::init())
        .attach(CORS)
        .mount(
            "/docs",
            make_swagger_ui(&SwaggerUIConfig {
                url: "../api/openapi.json".to_owned(),
                ..Default::default()
            }),
        );

    let openapi_settings = rocket_okapi::settings::OpenApiSettings::new();

    mount_endpoints_and_merged_docs! {
        launcher, "/api".to_owned(), openapi_settings,
        "/store" => methods::store::handlers::documented_routes(&openapi_settings),
        "/kiosk" => methods::kiosk::handlers::documented_routes(&openapi_settings),
        "/ingress" => methods::ingress::handlers::documented_routes(&openapi_settings),
        "/product" => methods::product::handlers::documented_routes(&openapi_settings),
        "/customer" => methods::customer::handlers::documented_routes(&openapi_settings),
        "/employee" => methods::employee::handlers::documented_routes(&openapi_settings),
        "/supplier" => methods::supplier::handlers::documented_routes(&openapi_settings),
        "/helpers" => methods::helpers::handlers::documented_routes(&openapi_settings),
        "/transaction" => methods::transaction::handlers::documented_routes(&openapi_settings),
    }

    launcher
}

#[cfg(not(feature = "process"))]
fn main() {}
