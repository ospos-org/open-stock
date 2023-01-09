use geo::{point};
use rocket::{routes, patch, http::{Status, CookieJar}, serde::json::{Json}, post, get};
use sea_orm_rocket::{Connection};
use serde::{Deserialize, Serialize};

use crate::{pool::Db, methods::{Employee, Store, Product, Customer, cookie_status_wrapper, Action}, check_permissions};
use photon_geocoding::{PhotonApiClient, PhotonFeature};
use geo::VincentyDistance;

pub fn routes() -> Vec<rocket::Route> {
    routes![generate_template, address_to_geolocation, distance_to_stores]
}

#[derive(Serialize, Deserialize)]
pub struct All {
    employee: Employee,
    stores: Vec<Store>,
    product: Product,
    customer: Customer
}

/// This route does not require authentication, but is not enabled in release mode.
#[patch("/generate")]
pub async fn generate_template(conn: Connection<'_, Db>) -> Result<Json<All>, Status> {
    let db = conn.into_inner();

    let employee = Employee::generate(db).await.unwrap();
    let stores = Store::generate(db).await.unwrap();
    let product = Product::generate(db).await.unwrap();
    let customer = Customer::generate(db).await.unwrap();

    Ok(rocket::serde::json::Json(All {
        employee,
        stores,
        product,
        customer
    }))
}

#[post("/address", data="<address>")]
pub async fn address_to_geolocation(conn: Connection<'_, Db>, address: &str, cookies: &CookieJar<'_>) -> Result<Json<(f64, f64)>, Status> {
    let db = conn.into_inner();
    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session, Action::FetchGeoLocation);

    match convert_addr_to_geo(address) {
        Ok(val) => Ok(Json(val)),
        Err(status) => Err(status),
    }
}

pub fn convert_addr_to_geo(address: &str) -> Result<(f64, f64), Status> {
    let api: PhotonApiClient = PhotonApiClient::default();
    let mut result: Vec<PhotonFeature> = api.forward_search(address, None).unwrap();
    match result.get_mut(0) {
        Some(loc) => {
            Ok((loc.coords.lat, loc.coords.lon))
        },
        None => Err(Status::UnprocessableEntity)
    }
}

#[derive(Serialize, Deserialize)]
pub struct Distance {
    store_id: String,
    store_code: String,
    distance: f64
}

#[get("/distance/<id>")]
pub async fn distance_to_stores(conn: Connection<'_, Db>, id: &str, cookies: &CookieJar<'_>) -> Result<Json<Vec<Distance>>, Status> {
    let db = conn.into_inner();
    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session, Action::FetchGeoLocation);

    let customer = match Customer::fetch_by_id(id, db).await {
        Ok(c) => c,
        Err(_) => return Err(Status::InternalServerError),
    };

    let stores = match Store::fetch_all(db).await {
        Ok(s) => s,
        Err(_) => return Err(Status::InternalServerError),
    };

    let cust = point!(x: customer.contact.address.lat, y: customer.contact.address.lon);

    Ok(Json(stores.into_iter().map(|store| {
        let stor = point!(x: store.contact.address.lat, y: store.contact.address.lon);

        Distance {
            /// Defaults to the diameter of the earth, i.e. longest distance between two points (minimizes priority if incorrect data is provided)
            distance: stor.vincenty_distance(&cust).unwrap_or(12756000.01),
            store_id: store.id,
            store_code: store.code
        }
    }).collect()))
}