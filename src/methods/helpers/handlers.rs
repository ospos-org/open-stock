use std::env;

use std::time::Duration;
use chrono::{Days, Utc};
use geo::point;
use rocket::{get, http::CookieJar, post, serde::json::Json};
use uuid::Uuid;

use crate::{check_permissions, example_employee, methods::{
    cookie_status_wrapper, Action, Address, Customer, Employee, Error, ErrorResponse, Product,
    Promotion, Session, Store, Transaction,
}, pool::Db, All, ContactInformation, Email, EmployeeInput, Kiosk, MobileNumber, NewTenantInput, NewTenantResponse, Tenant, TenantSettings, session, all_actions, AccountType, Distance};
use geo::VincentyDistance;
use okapi::openapi3::OpenApi;
use photon_geocoding::{
    filter::{ForwardFilter, PhotonLayer},
    LatLon, PhotonApiClient, PhotonFeature,
};
use rocket::http::{Cookie, SameSite};
use rocket::time::OffsetDateTime;
use rocket_db_pools::Connection;
use rocket_okapi::{openapi, openapi_get_routes_spec};
use rocket_okapi::settings::OpenApiSettings;
use sea_orm::EntityTrait;
use crate::catchers::Validated;
use crate::session::ActiveModel;

pub fn documented_routes(_settings: &OpenApiSettings) -> (Vec<rocket::Route>, OpenApi) {
    openapi_get_routes_spec![
        generate_template,
        address_to_geolocation,
        distance_to_stores,
        suggest_addr,
        new_tenant,
        distance_to_stores_from_store,
        assign_session_cookie
    ]
}

/// This route does not require authentication, but is not enabled in release mode.
#[openapi(tag = "Helpers")]
#[post("/generate")]
pub async fn generate_template(conn: Connection<Db>) -> Result<Json<All>, Error> {
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

    // Add Tenants
    let tenant = Tenant::generate(&db, tenant_id).await
        .map_err(|e| ErrorResponse::db_err(e))?;
    let tenant2 = Tenant::generate(&db, tenant_id2).await
        .map_err(|e| ErrorResponse::db_err(e))?;

    // Add Employees
    let employee = Employee::generate(&db, session.clone()).await
        .map_err(|e| ErrorResponse::db_err(e))?;
    let _employee2 = Employee::generate(&db, session2.clone()).await
        .map_err(|e| ErrorResponse::db_err(e))?;

    // Add other items (aggregated)
    let stores = Store::generate(session.clone(), &db).await
        .map_err(|e| ErrorResponse::db_err(e))?;
    let products = Product::generate(session.clone(), &db).await
        .map_err(|e| ErrorResponse::db_err(e))?;
    let customer = Customer::generate(session.clone(), &db).await
        .map_err(|e| ErrorResponse::db_err(e))?;

    // Add Kiosks
    let kiosk = Kiosk::generate("adbd48ab-f4ca-4204-9c88-3516f3133621", session.clone(), &db).await
        .map_err(|e| ErrorResponse::db_err(e))?;

    let _kiosk2 = Kiosk::generate("adbd48ab-f4ca-4204-9c88-3516f3133622", session2.clone(), &db).await
        .map_err(|e| ErrorResponse::db_err(e))?;

    let transaction = Transaction::generate(
        &db,
        &customer.id,
        Session {
            id: Uuid::new_v4().to_string(),
            key: String::new(),
            employee: employee.clone(),
            expiry: Utc::now(),
            tenant_id: tenant_id.to_string(),
        },
    ).await.map_err(|e| ErrorResponse::db_err(e))?;

    let promotions = Promotion::generate(session, &db).await.map_err(|e| ErrorResponse::db_err(e))?;

    Ok(Json(All {
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
#[openapi(tag = "Helpers")]
#[post("/new", data = "<tenant_input>")]
pub async fn new_tenant(
    conn: Connection<Db>,
    tenant_input: Validated<Json<NewTenantInput>>,
    _cookies: &CookieJar<'_>,
) -> Result<Json<NewTenantResponse>, Error> {
    let db = conn.into_inner();
    let data = tenant_input.0.into_inner();

    // Create new Tenant
    let tenant_id = Uuid::new_v4().to_string();
    let tenant = Tenant {
        tenant_id: tenant_id.clone(),
        settings: TenantSettings::default(),
        registration_date: Utc::now(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    Tenant::insert(tenant, &db)
        .await
        .map_err(ErrorResponse::db_err)?;

    // Create Primary Employee
    let employee = EmployeeInput {
        name: crate::Name::from_string(data.clone().name),
        level: all_actions(),
        rid: 0000,
        password: data.clone().password,
        account_type: AccountType::Managerial,
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

    let employee_insert_result = Employee::insert(
        employee, &db, session.clone(),
        None, Some(employee_id))
        .await
        .map_err(ErrorResponse::db_err)?;

    match session::Entity::insert::<ActiveModel>(session.clone().into())
        .exec(&db)
        .await
    {
        Ok(_) => {
            Ok(Json(NewTenantResponse {
                tenant_id,
                api_key: session.key,
                employee_id: employee_insert_result.last_insert_id,
            }))
        }
        Err(reason) => Err(ErrorResponse::db_err(reason)),
    }
}

#[openapi(tag = "Helpers")]
#[get("/session/<key>")]
pub async fn assign_session_cookie(
    _conn: Connection<Db>,
    key: &str,
    cookies: &CookieJar<'_>
) -> Result<Json<()>, Error> {
    let now = OffsetDateTime::now_utc();
    let expiry = now + Duration::from_secs(10 * 60);

    let hard_key = key.to_string();

    let cookie = Cookie::build("key", hard_key.clone())
        .expires(expiry)
        .path("/")
        .secure(true)
        .same_site(SameSite::None)
        .http_only(true)
        .finish();

    cookies.add(cookie);

    Ok(Json(()))
}

#[openapi(tag = "Helpers")]
#[post("/address", data = "<address>")]
pub async fn address_to_geolocation(
    conn: Connection<Db>,
    address: &str,
    cookies: &CookieJar<'_>,
) -> Result<Json<Address>, Error> {
    let db = conn.into_inner();
    let session = cookie_status_wrapper(&db, cookies).await?;
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

#[openapi(tag = "Helpers")]
#[post("/suggest", data = "<address>")]
pub async fn suggest_addr(
    conn: Connection<Db>,
    address: &str,
    cookies: &CookieJar<'_>,
) -> Result<Json<Vec<Address>>, Error> {
    let db = conn.into_inner();
    let session = cookie_status_wrapper(&db, cookies).await?;
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

#[openapi(tag = "Helpers")]
#[get("/distance/<id>")]
pub async fn distance_to_stores(
    conn: Connection<Db>,
    id: &str,
    cookies: &CookieJar<'_>,
) -> Result<Json<Vec<Distance>>, Error> {
    let db = conn.into_inner();
    let session = cookie_status_wrapper(&db, cookies).await?;
    check_permissions!(session.clone(), Action::FetchGeoLocation);

    let customer = match Customer::fetch_by_id(id, session.clone(), &db).await {
        Ok(c) => c,
        Err(reason) => return Err(ErrorResponse::db_err(reason)),
    };

    let stores = match Store::fetch_all(session, &db).await {
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
                    /// Defaults to the diameter of the earth, i.e. longest distance between two
                    /// points (minimizes priority if incorrect data is provided)
                    distance: stor.vincenty_distance(&cust).unwrap_or(12756000.01),
                    store_id: store.id,
                    store_code: store.code,
                }
            })
            .collect(),
    ))
}

#[openapi(tag = "Helpers")]
#[get("/distance/store/<store_id>")]
pub async fn distance_to_stores_from_store(
    conn: Connection<Db>,
    store_id: &str,
    cookies: &CookieJar<'_>,
) -> Result<Json<Vec<Distance>>, Error> {
    let db = conn.into_inner();
    let session = cookie_status_wrapper(&db, cookies).await?;
    check_permissions!(session.clone(), Action::FetchGeoLocation);

    let store_ = match Store::fetch_by_id(store_id, session.clone(), &db).await {
        Ok(c) => c,
        Err(reason) => return Err(ErrorResponse::db_err(reason)),
    };

    let stores = match Store::fetch_all(session, &db).await {
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
                    /// Defaults to the diameter of the earth, i.e. longest distance between two
                    /// points (minimizes priority if incorrect data is provided)
                    distance: stor.vincenty_distance(&cust).unwrap_or(12756000.01),
                    store_id: store.id,
                    store_code: store.code,
                }
            })
            .collect(),
    ))
}
