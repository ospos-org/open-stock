use okapi::openapi3::OpenApi;
use crate::check_permissions;
use crate::methods::{
    cookie_status_wrapper, Action, ContactInformation, CustomerWithTransactionsOut, Error,
    ErrorResponse, Transaction,
};
use crate::pool::Db;

use rocket::get;
use rocket::http::CookieJar;
use rocket::serde::json::Json;
use rocket::{post};

use rocket_db_pools::Connection;
use rocket_okapi::{openapi, openapi_get_routes_spec};
use rocket_okapi::settings::OpenApiSettings;

use super::{Customer, CustomerInput};

pub fn documented_routes(settings: &OpenApiSettings) -> (Vec<rocket::Route>, OpenApi) {
    openapi_get_routes_spec![
        settings:
        get,
        get_by_name,
        get_by_phone,
        get_by_addr,
        get_recent,
        create,
        update,
        generate,
        search_query,
        update_contact_info,
        find_related_transactions
    ]
}

#[openapi(tag = "Customer")]
#[get("/<id>")]
pub async fn get(
    conn: Connection<Db>,
    id: &str,
    cookies: &CookieJar<'_>,
) -> Result<Json<Customer>, Error> {
    let db = conn.into_inner();

    let session = cookie_status_wrapper(&db, cookies).await?;
    check_permissions!(session.clone(), Action::FetchCustomer);

    match Customer::fetch_by_id(id, session, &db).await {
        Ok(customers) => Ok(Json(customers)),
        Err(err) => Err(ErrorResponse::db_err(err)),
    }
}

#[openapi(tag = "Customer")]
#[get("/recent")]
pub async fn get_recent(
    conn: Connection<Db>,
    cookies: &CookieJar<'_>,
) -> Result<Json<Vec<Customer>>, Error> {
    let db = conn.into_inner();

    let session = cookie_status_wrapper(&db, cookies).await?;
    check_permissions!(session.clone(), Action::FetchCustomer);

    match Customer::fetch_recent(session, &db).await {
        Ok(customers) => Ok(Json(customers)),
        Err(err) => Err(ErrorResponse::db_err(err)),
    }
}

#[openapi(tag = "Customer")]
#[get("/name/<name>")]
pub async fn get_by_name(
    conn: Connection<Db>,
    name: &str,
    cookies: &CookieJar<'_>,
) -> Result<Json<Vec<Customer>>, Error> {
    let db = conn.into_inner();

    let session = cookie_status_wrapper(&db, cookies).await?;
    check_permissions!(session.clone(), Action::FetchCustomer);

    match Customer::fetch_by_name(name, session, &db).await {
        Ok(customers) => Ok(Json(customers)),
        Err(err) => Err(ErrorResponse::db_err(err)),
    }
}

/// Will search by both name, phone and email.
#[openapi(tag = "Customer")]
#[get("/search/<query>")]
pub async fn search_query(
    conn: Connection<Db>,
    query: &str,
    cookies: &CookieJar<'_>,
) -> Result<Json<Vec<CustomerWithTransactionsOut>>, Error> {
    let db = conn.into_inner();

    let session = cookie_status_wrapper(&db, cookies).await?;
    check_permissions!(session.clone(), Action::FetchCustomer);

    match Customer::search(query, session, &db).await {
        Ok(customers) => Ok(Json(customers)),
        Err(err) => Err(ErrorResponse::db_err(err)),
    }
}

#[openapi(tag = "Customer")]
#[get("/transactions/<id>")]
pub async fn find_related_transactions(
    conn: Connection<Db>,
    id: &str,
    cookies: &CookieJar<'_>,
) -> Result<Json<Vec<Transaction>>, Error> {
    let db = conn.into_inner();

    let session = cookie_status_wrapper(&db, cookies).await?;
    check_permissions!(session.clone(), Action::FetchCustomer);

    Ok(Json(
        Transaction::fetch_by_client_id(id, session, &db).await?
    ))
}

#[openapi(tag = "Customer")]
#[get("/phone/<phone>")]
pub async fn get_by_phone(
    conn: Connection<Db>,
    phone: &str,
    cookies: &CookieJar<'_>,
) -> Result<Json<Vec<Customer>>, Error> {
    let db = conn.into_inner();
    let session = cookie_status_wrapper(&db, cookies).await?;
    check_permissions!(session.clone(), Action::FetchCustomer);

    Ok(Json(
        Customer::fetch_by_phone(phone, session, &db).await?
    ))
}

#[openapi(tag = "Customer")]
#[get("/addr/<addr>")]
pub async fn get_by_addr(
    conn: Connection<Db>,
    addr: &str,
    cookies: &CookieJar<'_>,
) -> Result<Json<Vec<Customer>>, Error> {
    let db = conn.into_inner();

    let session = cookie_status_wrapper(&db, cookies).await?;
    check_permissions!(session.clone(), Action::FetchCustomer);

    Ok(Json(
        Customer::fetch_by_addr(addr, session, &db).await?
    ))
}

#[openapi(tag = "Customer")]
#[post("/generate")]
async fn generate(
    conn: Connection<Db>,
    cookies: &CookieJar<'_>,
) -> Result<Json<Customer>, Error> {
    let db = conn.into_inner();

    let session = cookie_status_wrapper(&db, cookies).await?;
    check_permissions!(session.clone(), Action::GenerateTemplateContent);

    Ok(Json(
        Customer::generate(session, &db).await?
    ))
}

#[openapi(tag = "Customer")]
#[post("/<id>", data = "<input_data>")]
async fn update(
    conn: Connection<Db>,
    id: &str,
    cookies: &CookieJar<'_>,
    input_data: Json<Customer>,
) -> Result<Json<Customer>, Error> {
    let input_data = input_data.clone().into_inner();
    let db = conn.into_inner();

    let session = cookie_status_wrapper(&db, cookies).await?;
    check_permissions!(session.clone(), Action::ModifyCustomer);

    Ok(Json(
        Customer::update(input_data, session, id, &db).await?
    ))
}

#[openapi(tag = "Customer")]
#[post("/contact/<id>", data = "<input_data>")]
async fn update_contact_info(
    conn: Connection<Db>,
    id: &str,
    cookies: &CookieJar<'_>,
    input_data: Json<ContactInformation>,
) -> Result<Json<Customer>, Error> {
    let input_data = input_data.clone().into_inner();
    let db = conn.into_inner();
    let session = cookie_status_wrapper(&db, cookies).await?;

    check_permissions!(session.clone(), Action::ModifyCustomer);

    match Customer::update_contact_information(input_data, id, session, &db).await {
        Ok(res) => Ok(Json(res)),
        Err(error) => Err(ErrorResponse::db_err(error)),
    }
}

#[openapi(tag = "Customer")]
#[post("/", data = "<input_data>")]
pub async fn create(
    conn: Connection<Db>,
    cookies: &CookieJar<'_>,
    input_data: Json<CustomerInput>,
) -> Result<Json<Customer>, Error> {
    let new_transaction = input_data.clone().into_inner();
    let db = conn.into_inner();

    let session = cookie_status_wrapper(&db, cookies).await?;
    check_permissions!(session.clone(), Action::CreateCustomer);

    match Customer::insert(new_transaction, session.clone(), &db).await {
        Ok(data) =>
            match Customer::fetch_by_id(
                &data.last_insert_id, session, &db
            ).await {
                Ok(res) => Ok(Json(res)),
                Err(reason) => {
                    println!("[dberr]: {}", reason);
                    Err(ErrorResponse::create_error(&format!(
                        "Fetch for customer failed, reason: {}",
                        reason
                    )))
                }
            },
        Err(error) => Err(ErrorResponse::db_err(error)),
    }
}
