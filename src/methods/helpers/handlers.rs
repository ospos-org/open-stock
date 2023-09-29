use std::env;

use chrono::{Days, Utc};
use geo::point;
use rocket::{get, http::CookieJar, post, routes, serde::json::Json};
use sea_orm_rocket::Connection;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{check_permissions, create_cookie, example_employee, methods::{
    cookie_status_wrapper, Action, Address, Customer, Employee, Error, ErrorResponse, Product,
    Promotion, Session, Store, Transaction,
}, pool::Db, All, ContactInformation, Email, EmployeeAuth, EmployeeInput, Kiosk, MobileNumber, NewTenantInput, NewTenantResponse, Tenant, TenantSettings, Access};
use geo::VincentyDistance;
use photon_geocoding::{
    filter::{ForwardFilter, PhotonLayer},
    LatLon, PhotonApiClient, PhotonFeature,
};

pub fn routes() -> Vec<rocket::Route> {
    routes![
        generate_template,
        address_to_geolocation,
        distance_to_stores,
        suggest_addr,
        new_tenant,
        distance_to_stores_from_store
    ]
}

/// This route does not require authentication, but is not enabled in release mode.
#[post("/generate")]
pub async fn generate_template(conn: Connection<'_, Db>) -> Result<Json<All>, Error> {
    if env::var("DEMO").is_err() || env::var("DEMO").unwrap() == "0" {
        return Err(Error::DemoDisabled(
            "OpenStock is not in DEMO mode.".to_string(),
        ));
    }

    let db = conn.into_inner();
    let tenant_id = "DEFAULT_TENANT";
    let tenant_id2 = "ALTERNATE_TENANT";
    let default_employee = example_employee();

    let session = Session {
        id: String::new(),
        key: String::new(),
        employee: default_employee.clone().into(),
        expiry: Utc::now().checked_add_days(Days::new(1)).unwrap(),
        tenant_id: tenant_id.to_string().clone(),
    };

    let session2 = Session {
        id: String::new(),
        key: String::new(),
        employee: default_employee.into(),
        expiry: Utc::now().checked_add_days(Days::new(1)).unwrap(),
        tenant_id: tenant_id2.to_string().clone(),
    };

    let tenant = Tenant::generate(db, tenant_id).await.unwrap();
    let tenant2 = Tenant::generate(db, tenant_id2).await.unwrap();

    let employee = Employee::generate(db, session.clone()).await.unwrap();
    let _employee2 = Employee::generate(db, session2.clone()).await.unwrap();

    let stores = Store::generate(session.clone(), db).await.unwrap();
    let products = Product::generate(session.clone(), db).await.unwrap();
    let customer = Customer::generate(session.clone(), db).await.unwrap();

    let kiosk = Kiosk::generate("adbd48ab-f4ca-4204-9c88-3516f3133621", session.clone(), db)
        .await
        .unwrap();

    let _kiosk2 = Kiosk::generate("adbd48ab-f4ca-4204-9c88-3516f3133622", session2.clone(), db)
        .await
        .unwrap();

    let transaction = Transaction::generate(
        db,
        &customer.id,
        Session {
            id: Uuid::new_v4().to_string(),
            key: String::new(),
            employee: employee.clone(),
            expiry: Utc::now(),
            tenant_id: tenant_id.to_string(),
        },
    )
    .await
    .unwrap();
    let promotions = Promotion::generate(session, db).await.unwrap();

    Ok(rocket::serde::json::Json(All {
        employee,
        tenants: vec![tenant, tenant2],
        stores,
        products,
        customer,
        transaction,
        promotions,
        kiosk,
    }))
}

/// Unprotected route, does not require a cookie
#[post("/new", data = "<tenant_input>")]
pub async fn new_tenant(
    conn: Connection<'_, Db>,
    tenant_input: Json<NewTenantInput>,
    cookies: &CookieJar<'_>,
) -> Result<Json<NewTenantResponse>, Error> {
    let db = conn.into_inner();
    let data = tenant_input.into_inner();

    // Create new Tenant
    let tenant_id = Uuid::new_v4().to_string();
    let tenant = Tenant {
        tenant_id: tenant_id.clone(),
        settings: TenantSettings::default(),
        registration_date: Utc::now(),
    };

    Tenant::insert(tenant, db)
        .await
        .map_err(ErrorResponse::db_err)?;

    // Create Primary Employee
    let employee = EmployeeInput {
        name: crate::Name::from_string(data.clone().name),
        level: vec![Access { action: Action::AccessAdminPanel, authority: 1 }],
        rid: 0000,
        password: "...".to_string(),
        clock_history: vec![],
        contact: ContactInformation {
            name: data.clone().name,
            mobile: MobileNumber {
                number: "".to_string(),
                valid: false,
            },
            email: Email::from(data.clone().email),
            landline: "".to_string(),
            address: convert_addr_to_geo(&data.clone().address)?,
        },
    };

    let employee_id = Uuid::new_v4().to_string();

    // Load a temporary session
    let session = Session::ingestion(
        employee.clone(),
        tenant_id.clone(),
        Some(employee_id.clone())
    );

    Employee::insert(
        employee, db, session.clone(),
        None, Some(employee_id))
        .await
        .map_err(ErrorResponse::db_err)?;

    Ok(Json(NewTenantResponse {
        tenant_id,
        api_key: session.key,
    }))
}

#[post("/address", data = "<address>")]
pub async fn address_to_geolocation(
    conn: Connection<'_, Db>,
    address: &str,
    cookies: &CookieJar<'_>,
) -> Result<Json<Address>, Error> {
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
                street: format!(
                    "{} {}",
                    loc.house_number.clone().unwrap_or("0".to_string()),
                    loc.street.clone().unwrap_or("".to_string())
                ),
                street2: loc.district.clone().unwrap_or("".to_string()),
                city: loc.city.clone().unwrap_or("".to_string()),
                country: loc.country.clone().unwrap_or("".to_string()),
                po_code: loc.postcode.clone().unwrap_or("".to_string()),
                lat: loc.coords.lat,
                lon: loc.coords.lon,
            })
        }
        None => Err(ErrorResponse::create_error(
            "Unable to search for location.",
        )),
    }
}

pub fn convert_addresses_to_geo(address: &str, origin: LatLon) -> Result<Vec<Address>, Error> {
    let api: PhotonApiClient = PhotonApiClient::default();
    let result: Vec<PhotonFeature> = api
        .forward_search(
            address,
            Some(
                ForwardFilter::new()
                    .location_bias(origin, Some(16), Some(0.3))
                    .limit(5)
                    .layer(vec![PhotonLayer::House]),
            ),
        )
        .unwrap();

    let mapped = result
        .iter()
        .map(|loc| Address {
            street: format!(
                "{} {}",
                loc.house_number.clone().unwrap_or("0".to_string()),
                loc.street.clone().unwrap_or("".to_string())
            ),
            street2: loc.district.clone().unwrap_or("".to_string()),
            city: loc.city.clone().unwrap_or("".to_string()),
            country: loc.country.clone().unwrap_or("".to_string()),
            po_code: loc.postcode.clone().unwrap_or("".to_string()),
            lat: loc.coords.lat,
            lon: loc.coords.lon,
        })
        .collect();

    Ok(mapped)
}

#[post("/suggest", data = "<address>")]
pub async fn suggest_addr(
    conn: Connection<'_, Db>,
    address: &str,
    cookies: &CookieJar<'_>,
) -> Result<Json<Vec<Address>>, Error> {
    let db = conn.into_inner();
    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session.clone(), Action::FetchGeoLocation);

    match convert_addresses_to_geo(
        address,
        LatLon {
            lat: session.employee.contact.address.lat,
            lon: session.employee.contact.address.lon,
        },
    ) {
        Ok(val) => Ok(Json(val)),
        Err(status) => Err(status),
    }
}

#[derive(Serialize, Deserialize)]
pub struct Distance {
    store_id: String,
    store_code: String,
    distance: f64,
}

#[get("/distance/<id>")]
pub async fn distance_to_stores(
    conn: Connection<'_, Db>,
    id: &str,
    cookies: &CookieJar<'_>,
) -> Result<Json<Vec<Distance>>, Error> {
    let db = conn.into_inner();
    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session.clone(), Action::FetchGeoLocation);

    let customer = match Customer::fetch_by_id(id, session.clone(), db).await {
        Ok(c) => c,
        Err(reason) => return Err(ErrorResponse::db_err(reason)),
    };

    let stores = match Store::fetch_all(session, db).await {
        Ok(s) => s,
        Err(reason) => return Err(ErrorResponse::db_err(reason)),
    };

    let cust = point!(x: customer.contact.address.lat, y: customer.contact.address.lon);

    Ok(Json(
        stores
            .into_iter()
            .map(|store| {
                let stor = point!(x: store.contact.address.lat, y: store.contact.address.lon);

                Distance {
                    /// Defaults to the diameter of the earth, i.e. longest distance between two points (minimizes priority if incorrect data is provided)
                    distance: stor.vincenty_distance(&cust).unwrap_or(12756000.01),
                    store_id: store.id,
                    store_code: store.code,
                }
            })
            .collect(),
    ))
}

#[get("/distance/store/<store_id>")]
pub async fn distance_to_stores_from_store(
    conn: Connection<'_, Db>,
    store_id: &str,
    cookies: &CookieJar<'_>,
) -> Result<Json<Vec<Distance>>, Error> {
    let db = conn.into_inner();
    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session.clone(), Action::FetchGeoLocation);

    let store_ = match Store::fetch_by_id(store_id, session.clone(), db).await {
        Ok(c) => c,
        Err(reason) => return Err(ErrorResponse::db_err(reason)),
    };

    let stores = match Store::fetch_all(session, db).await {
        Ok(s) => s,
        Err(reason) => return Err(ErrorResponse::db_err(reason)),
    };

    let cust = point!(x: store_.contact.address.lat, y: store_.contact.address.lon);

    Ok(Json(
        stores
            .into_iter()
            .map(|store| {
                let stor = point!(x: store.contact.address.lat, y: store.contact.address.lon);

                Distance {
                    /// Defaults to the diameter of the earth, i.e. longest distance between two points (minimizes priority if incorrect data is provided)
                    distance: stor.vincenty_distance(&cust).unwrap_or(12756000.01),
                    store_id: store.id,
                    store_code: store.code,
                }
            })
            .collect(),
    ))
}
