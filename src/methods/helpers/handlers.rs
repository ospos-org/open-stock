use crate::catchers::Validated;
use crate::session::ActiveModel;
use crate::ContactInformationInput;
use crate::{
    all_actions, check_permissions, create_cookie, example_employee,
    methods::{
        cookie_status_wrapper, Action, Address, Customer, Employee, Error, ErrorResponse, Product,
        Promotion, Session, Store, Transaction,
    },
    pool::Db,
    session, AccountType, All, Distance, EmployeeInput, Kiosk, NewTenantInput, NewTenantResponse,
    SessionRaw, SessionVariant, Tenant, TenantSettings,
};
use chrono::{Days, Duration, Utc};
use geo::point;
use geo::VincentyDistance;
use okapi::openapi3::OpenApi;
use photon_geocoding::{
    filter::{ForwardFilter, PhotonLayer},
    LatLon, PhotonApiClient, PhotonFeature,
};
use rocket::{get, http::CookieJar, post, serde::json::Json};
use rocket_db_pools::Connection;
use rocket_okapi::settings::OpenApiSettings;
use rocket_okapi::{openapi, openapi_get_routes_spec};
use sea_orm::{ActiveModelTrait, ColumnTrait, DbErr, EntityTrait, QueryFilter, QuerySelect, Set};
use serde_json::json;
use std::env;
use uuid::Uuid;

pub fn documented_routes(_settings: &OpenApiSettings) -> (Vec<rocket::Route>, OpenApi) {
    openapi_get_routes_spec![
        generate_template,
        address_to_geolocation,
        distance_to_stores,
        suggest_addr,
        new_tenant,
        distance_to_stores_from_store,
        assign_session_cookie,
        refresh_token_create,
        refresh_token_refresh
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
        variant: SessionVariant::AccessToken,
    };

    let session2 = Session {
        id: String::new(),
        key: String::new(),
        employee: default_employee.into(),
        expiry: Utc::now().checked_add_days(Days::new(1)).unwrap(),
        tenant_id: tenant_id2.to_string().clone(),
        variant: SessionVariant::AccessToken,
    };

    // Add Tenants
    let tenant = Tenant::generate(&db, tenant_id)
        .await
        .map_err(ErrorResponse::db_err)?;
    let tenant2 = Tenant::generate(&db, tenant_id2)
        .await
        .map_err(ErrorResponse::db_err)?;

    // Add Employees
    let employee = Employee::generate(&db, session.clone())
        .await
        .map_err(ErrorResponse::db_err)?;
    let _employee2 = Employee::generate(&db, session2.clone())
        .await
        .map_err(ErrorResponse::db_err)?;

    // Add other items (aggregated)
    let stores = Store::generate(session.clone(), &db)
        .await
        .map_err(ErrorResponse::db_err)?;
    let products = Product::generate(session.clone(), &db)
        .await
        .map_err(ErrorResponse::db_err)?;
    let customer = Customer::generate(session.clone(), &db)
        .await
        .map_err(ErrorResponse::db_err)?;

    // Add Kiosks
    let kiosk = Kiosk::generate("adbd48ab-f4ca-4204-9c88-3516f3133621", session.clone(), &db)
        .await
        .map_err(ErrorResponse::db_err)?;

    let _kiosk2 = Kiosk::generate(
        "adbd48ab-f4ca-4204-9c88-3516f3133622",
        session2.clone(),
        &db,
    )
    .await
    .map_err(ErrorResponse::db_err)?;

    let transaction = Transaction::generate(
        &db,
        &customer.id,
        Session {
            id: Uuid::new_v4().to_string(),
            key: String::new(),
            employee: employee.clone(),
            expiry: Utc::now(),
            tenant_id: tenant_id.to_string(),
            variant: SessionVariant::AccessToken,
        },
    )
    .await
    .map_err(ErrorResponse::db_err)?;

    let promotions = Promotion::generate(session, &db)
        .await
        .map_err(ErrorResponse::db_err)?;

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
        name: data.clone().name,
        level: all_actions(),
        rid: 0000,
        password: Some(data.clone().password),
        account_type: AccountType::Managerial,
        clock_history: vec![],
        contact: ContactInformationInput {
            name: data.clone().name,
            mobile: "".to_string(),
            email: data.clone().email,
            landline: "".to_string(),
            address: convert_addr_to_geo(&data.clone().address)?,
        },
    };

    let employee_id = Uuid::new_v4().to_string();

    // Load a temporary session
    let session = Session::ingestion(
        employee.clone(),
        tenant_id.clone(),
        Some(employee_id.clone()),
    );

    let employee_insert_result =
        Employee::insert(employee, &db, session.clone(), None, Some(employee_id))
            .await
            .map_err(ErrorResponse::db_err)?;

    match session::Entity::insert::<ActiveModel>(session.clone().into())
        .exec(&db)
        .await
    {
        Ok(_) => Ok(Json(NewTenantResponse {
            tenant_id,
            api_key: session.key,
            employee_id: employee_insert_result.last_insert_id,
        })),
        Err(reason) => Err(ErrorResponse::db_err(reason)),
    }
}

#[openapi(tag = "Helpers")]
#[get("/session/<key>")]
pub async fn assign_session_cookie(
    _conn: Connection<Db>,
    key: &str,
    cookies: &CookieJar<'_>,
) -> Result<Json<()>, Error> {
    let hard_key = key.to_string();
    let cookie = create_cookie(hard_key.clone());

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
                    distance: stor.vincenty_distance(&cust).unwrap_or(12756000.01),
                    store_id: store.id,
                    store_code: store.code,
                }
            })
            .collect(),
    ))
}

/// By fetching this route, as long as the user
/// is authenticated by the access token, we will
/// generate a refresh token for it.
///
///
#[openapi(tag = "Helpers")]
#[get("/refresh_token")]
pub async fn refresh_token_create(
    conn: Connection<Db>,
    cookies: &CookieJar<'_>,
) -> Result<Json<String>, Error> {
    let db = conn.into_inner();
    let session = cookie_status_wrapper(&db, cookies).await?;

    let token_key = Uuid::new_v4().to_string();
    let token_id = Uuid::new_v4().to_string();

    match session::Entity::insert(session::ActiveModel {
        id: Set(token_id),
        key: Set(token_key.clone()),
        variant: Set(json!(SessionVariant::RefreshToken(session.id))),
        employee_id: Set(session.employee.id),
        tenant_id: Set(session.tenant_id),
        expiry: Set(Utc::now()
            .checked_add_signed(Duration::days(7))
            .unwrap()
            .naive_utc()),
    })
    .exec(&db)
    .await
    {
        // Note; we do not assign a cookie.
        // This is a refresh token, not an access token.
        Ok(_) => Ok(Json(token_key)),
        Err(reason) => Err(ErrorResponse::db_err(reason)),
    }
}

/// Sending a refresh token through to this route,
/// will create a new access token and assign it
/// as server-assigned current cookie.
///
///
#[openapi(tag = "Helpers")]
#[get("/refresh_token/<token>")]
pub async fn refresh_token_refresh(
    conn: Connection<Db>,
    token: &str,
    cookies: &CookieJar<'_>,
) -> Result<Json<String>, Error> {
    let db = conn.into_inner();

    let found_token = crate::entities::session::Entity::find()
        .having(session::Column::Key.eq(token))
        .one(&db)
        .await?;

    match found_token {
        Some(e) => {
            let decoded_token: SessionRaw = e.into();
            println!("GOT TOKEN: {:?}", decoded_token);

            match decoded_token.variant {
                SessionVariant::AccessToken => Err(ErrorResponse::create_error(
                    "Expected a Refresh Token, got an Access Token",
                )),
                SessionVariant::RefreshToken(access_reference) => {
                    let access_token = crate::entities::session::Entity::find()
                        .having(session::Column::Key.eq(access_reference.clone()))
                        .one(&db)
                        .await?;

                    match access_token {
                        Some(token) => {
                            println!("THEN GOT ACCESS TOKEN: {:?}", token);

                            let decoded_token_2: SessionRaw = token.into();
                            let api_key = Uuid::new_v4().to_string();

                            // Return an updated access token (we update
                            // to optimize the avoidance of a dangling token)
                            match session::Entity::update(session::ActiveModel {
                                id: Set(decoded_token_2.id.to_string()),
                                key: Set(api_key.clone()),
                                variant: Set(json!(SessionVariant::AccessToken)),
                                ..Default::default()
                            })
                            .exec(&db)
                            .await
                            {
                                Ok(_) => {
                                    // Assign the cookie
                                    cookies.add(create_cookie(api_key.clone()));

                                    Ok(Json(api_key))
                                }
                                Err(reason) => Err(ErrorResponse::db_err(reason)),
                            }
                        }
                        None => {
                            println!("ACCESS TOKEN EXPIRED! LET'S GENERATE ONE! {:?}", token);

                            let new_access_key = Uuid::new_v4().to_string();

                            let exp = Utc::now()
                                .checked_add_signed(Duration::minutes(10))
                                .unwrap();

                            let access_token_to_insert = session::ActiveModel {
                                id: Set(access_reference),
                                key: Set(new_access_key.clone()),
                                employee_id: Set(decoded_token.employee_id),
                                expiry: Set(exp.naive_utc()),
                                tenant_id: Set(decoded_token.tenant_id),
                                variant: Set(json!(SessionVariant::AccessToken)),
                            };

                            match session::Entity::insert(access_token_to_insert)
                                .exec(&db)
                                .await
                            {
                                Ok(_) => {
                                    // Assign the cookie
                                    cookies.add(create_cookie(new_access_key.clone()));

                                    Ok(Json(new_access_key))
                                }
                                Err(reason) => Err(ErrorResponse::db_err(reason)),
                            }
                        }
                    }
                }
            }
        }
        None => Err(ErrorResponse::db_err(DbErr::RecordNotFound(
            token.to_string(),
        ))),
    }
}
