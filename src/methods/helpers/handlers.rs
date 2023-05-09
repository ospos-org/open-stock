use std::env;

use chrono::Utc;
use geo::{point};
use rocket::{routes, http::{CookieJar}, serde::json::{Json}, post, get};
use sea_orm_rocket::{Connection};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{pool::Db, methods::{Employee, Store, Product, Customer, cookie_status_wrapper, Action, Address, Transaction, Session, Promotion, Error, ErrorResponse}, check_permissions};
use photon_geocoding::{PhotonApiClient, PhotonFeature, filter::{ForwardFilter, PhotonLayer}, LatLon};
use geo::VincentyDistance;

pub fn routes() -> Vec<rocket::Route> {
    routes![generate_template, address_to_geolocation, distance_to_stores, suggest_addr, distance_to_stores_from_store]
}

#[derive(Serialize, Deserialize)]
pub struct All {
    employee: Employee,
    stores: Vec<Store>,
    products: Vec<Product>,
    customer: Customer,
    transaction: Transaction,
    promotions: Vec<Promotion>
}

/// This route does not require authentication, but is not enabled in release mode.
#[post("/generate")]
pub async fn generate_template(conn: Connection<'_, Db>) -> Result<Json<All>, Error> {
    let _ = match env::var("DEMO") {
        Ok(url) => {
            match url.as_str() {
                "0" => return Err(Error::DemoDisabled(format!("OpenStock is not in DEMO mode."))),
                "1" => true,
                _ => return Err(Error::DemoDisabled(format!("OpenStock is not in DEMO mode.")))
            }
        },
        Err(_) => {
            return Err(Error::DemoDisabled(format!("OpenStock is not in DEMO mode.")))
        },
    };

    let db = conn.into_inner();

    let employee = Employee::generate(db).await.unwrap();
    let stores = Store::generate(db).await.unwrap();
    let products = Product::generate(db).await.unwrap();
    let customer = Customer::generate(db).await.unwrap();
    let transaction = Transaction::generate(db, &customer.id, Session { id: Uuid::new_v4().to_string(), key: format!(""), employee: employee.clone(), expiry: Utc::now() }).await.unwrap();
    let promotions = Promotion::generate(db).await.unwrap();

    Ok(rocket::serde::json::Json(All {
        employee,
        stores,
        products,
        customer,
        transaction,
        promotions
    }))
}

#[post("/address", data="<address>")]
pub async fn address_to_geolocation(conn: Connection<'_, Db>, address: &str, cookies: &CookieJar<'_>) -> Result<Json<Address>, Error> {
    let db = conn.into_inner();
    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session, Action::FetchGeoLocation);

    match convert_addr_to_geo(address) {
        Ok(val) => Ok(Json(val)),
        Err(status) => Err(status),
    }
}

pub fn convert_addr_to_geo(address: &str) -> Result<Address, Error> {
    let api: PhotonApiClient = PhotonApiClient::default();
    let result: Vec<PhotonFeature> = api.forward_search(address, None).unwrap();

    match result.get(0) {
        Some(loc) => {
            println!("{:?}", loc);

            Ok(Address { 
                street: format!("{} {}", loc.house_number.clone().unwrap_or("0".to_string()), loc.street.clone().unwrap_or("".to_string())), 
                street2: format!("{}", loc.district.clone().unwrap_or("".to_string())), 
                city: loc.city.clone().unwrap_or("".to_string()), 
                country: loc.country.clone().unwrap_or("".to_string()), 
                po_code: loc.postcode.clone().unwrap_or("".to_string()), 
                lat: loc.coords.lat, 
                lon: loc.coords.lon 
            })
        },
        None => Err(ErrorResponse::create_error("Unable to search for location."))
    }
}

pub fn convert_addresses_to_geo(address: &str, origin: LatLon) -> Result<Vec<Address>, Error> {
    let api: PhotonApiClient = PhotonApiClient::default();
    let result: Vec<PhotonFeature> = api.forward_search(address, Some(ForwardFilter::new()
        .location_bias(origin, Some(16), Some(0.3))
        .limit(5)
        .layer(vec![PhotonLayer::House])
    )).unwrap();

    let mapped = result.iter().map(|loc| 
        Address { 
            street: format!("{} {}", loc.house_number.clone().unwrap_or("0".to_string()), loc.street.clone().unwrap_or("".to_string())), 
            street2: format!("{}", loc.district.clone().unwrap_or("".to_string())), 
            city: loc.city.clone().unwrap_or("".to_string()), 
            country: loc.country.clone().unwrap_or("".to_string()), 
            po_code: loc.postcode.clone().unwrap_or("".to_string()), 
            lat: loc.coords.lat, 
            lon: loc.coords.lon 
        }
    ).collect();

    Ok(mapped)
}

#[post("/suggest", data="<address>")]
pub async fn suggest_addr(conn: Connection<'_, Db>, address: &str, cookies: &CookieJar<'_>) -> Result<Json<Vec<Address>>, Error> {
    let db = conn.into_inner();
    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session.clone(), Action::FetchGeoLocation);

    match convert_addresses_to_geo(address, LatLon { lat: session.employee.contact.address.lat.clone(), lon: session.employee.contact.address.lon.clone() }) {
        Ok(val) => Ok(Json(val)),
        Err(status) => Err(status),
    }
}

#[derive(Serialize, Deserialize)]
pub struct Distance {
    store_id: String,
    store_code: String,
    distance: f64
}

#[get("/distance/<id>")]
pub async fn distance_to_stores(conn: Connection<'_, Db>, id: &str, cookies: &CookieJar<'_>) -> Result<Json<Vec<Distance>>, Error> {
    let db = conn.into_inner();
    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session, Action::FetchGeoLocation);

    let customer = match Customer::fetch_by_id(id, db).await {
        Ok(c) => c,
        Err(reason) => return Err(ErrorResponse::db_err(reason)),
    };

    let stores = match Store::fetch_all(db).await {
        Ok(s) => s,
        Err(reason) => return Err(ErrorResponse::db_err(reason)),
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

#[get("/distance/store/<store_id>")]
pub async fn distance_to_stores_from_store(conn: Connection<'_, Db>, store_id: &str, cookies: &CookieJar<'_>) -> Result<Json<Vec<Distance>>, Error> {
    let db = conn.into_inner();
    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session, Action::FetchGeoLocation);

    let store_ = match Store::fetch_by_id(store_id, db).await {
        Ok(c) => c,
        Err(reason) => return Err(ErrorResponse::db_err(reason)),
    };

    let stores = match Store::fetch_all(db).await {
        Ok(s) => s,
        Err(reason) => return Err(ErrorResponse::db_err(reason)),
    };

    let cust = point!(x: store_.contact.address.lat, y: store_.contact.address.lon);

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