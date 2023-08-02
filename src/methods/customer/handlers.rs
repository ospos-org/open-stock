use crate::check_permissions;
use crate::methods::{
    cookie_status_wrapper, Action, ContactInformation, CustomerWithTransactionsOut, Error,
    ErrorResponse, Transaction,
};
use crate::pool::Db;
use rocket::get;
use rocket::http::CookieJar;
use rocket::serde::json::Json;
use rocket::{post, routes};
use sea_orm_rocket::Connection;

use super::{Customer, CustomerInput};

pub fn routes() -> Vec<rocket::Route> {
    routes![
        get,
        get_by_name,
        get_by_phone,
        get_by_addr,
        create,
        update,
        generate,
        search_query,
        update_contact_info,
        find_related_transactions
    ]
}

#[get("/<id>")]
pub async fn get(
    conn: Connection<'_, Db>,
    id: &str,
    cookies: &CookieJar<'_>,
) -> Result<Json<Customer>, Error> {
    let db = conn.into_inner();

    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session.clone(), Action::FetchCustomer);

    match Customer::fetch_by_id(id, session, db).await {
        Ok(customers) => Ok(Json(customers)),
        Err(err) => Err(ErrorResponse::db_err(err)),
    }
}

#[get("/name/<name>")]
pub async fn get_by_name(
    conn: Connection<'_, Db>,
    name: &str,
    cookies: &CookieJar<'_>,
) -> Result<Json<Vec<Customer>>, Error> {
    let db = conn.into_inner();

    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session.clone(), Action::FetchCustomer);

    match Customer::fetch_by_name(name, session, db).await {
        Ok(customers) => Ok(Json(customers)),
        Err(err) => Err(ErrorResponse::db_err(err)),
    }
}

/// Will search by both name, phone and email.
#[get("/search/<query>")]
pub async fn search_query(
    conn: Connection<'_, Db>,
    query: &str,
    cookies: &CookieJar<'_>,
) -> Result<Json<Vec<CustomerWithTransactionsOut>>, Error> {
    let db = conn.into_inner();

    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session.clone(), Action::FetchCustomer);

    match Customer::search(query, session, db).await {
        Ok(customers) => Ok(Json(customers)),
        Err(err) => Err(ErrorResponse::db_err(err)),
    }
}

#[get("/transactions/<id>")]
pub async fn find_related_transactions(
    conn: Connection<'_, Db>,
    id: &str,
    cookies: &CookieJar<'_>,
) -> Result<Json<Vec<Transaction>>, Error> {
    let db = conn.into_inner();

    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session.clone(), Action::FetchCustomer);

    match Transaction::fetch_by_client_id(id, session, db).await {
        Ok(transactions) => Ok(Json(transactions)),
        Err(err) => Err(ErrorResponse::db_err(err)),
    }
}

#[get("/phone/<phone>")]
pub async fn get_by_phone(
    conn: Connection<'_, Db>,
    phone: &str,
    cookies: &CookieJar<'_>,
) -> Result<Json<Vec<Customer>>, Error> {
    let db = conn.into_inner();
    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session.clone(), Action::FetchCustomer);

    match Customer::fetch_by_phone(phone, session, db).await {
        Ok(customer) => Ok(Json(customer)),
        Err(err) => Err(ErrorResponse::db_err(err)),
    }
}

#[get("/addr/<addr>")]
pub async fn get_by_addr(
    conn: Connection<'_, Db>,
    addr: &str,
    cookies: &CookieJar<'_>,
) -> Result<Json<Vec<Customer>>, Error> {
    let db = conn.into_inner();

    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session.clone(), Action::FetchCustomer);

    match Customer::fetch_by_addr(addr, session, db).await {
        Ok(customers) => Ok(Json(customers)),
        Err(err) => Err(ErrorResponse::db_err(err)),
    }
}

#[post("/generate")]
async fn generate(
    conn: Connection<'_, Db>,
    cookies: &CookieJar<'_>,
) -> Result<Json<Customer>, Error> {
    let db = conn.into_inner();

    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session.clone(), Action::GenerateTemplateContent);

    match Customer::generate(session, db).await {
        Ok(res) => Ok(Json(res)),
        Err(err) => Err(ErrorResponse::db_err(err)),
    }
}

#[post("/<id>", data = "<input_data>")]
async fn update(
    conn: Connection<'_, Db>,
    id: &str,
    cookies: &CookieJar<'_>,
    input_data: Json<CustomerInput>,
) -> Result<Json<Customer>, Error> {
    let input_data = input_data.clone().into_inner();
    let db = conn.into_inner();

    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session.clone(), Action::ModifyCustomer);

    match Customer::update(input_data, session, id, db).await {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(ErrorResponse::input_error()),
    }
}

#[post("/contact/<id>", data = "<input_data>")]
async fn update_contact_info(
    conn: Connection<'_, Db>,
    id: &str,
    cookies: &CookieJar<'_>,
    input_data: Json<ContactInformation>,
) -> Result<Json<Customer>, Error> {
    let input_data = input_data.clone().into_inner();
    let db = conn.into_inner();
    let session = cookie_status_wrapper(db, cookies).await?;

    check_permissions!(session.clone(), Action::ModifyCustomer);

    match Customer::update_contact_information(input_data, id, session, db).await {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(ErrorResponse::input_error()),
    }
}

#[post("/", data = "<input_data>")]
pub async fn create(
    conn: Connection<'_, Db>,
    cookies: &CookieJar<'_>,
    input_data: Json<CustomerInput>,
) -> Result<Json<Customer>, Error> {
    let new_transaction = input_data.clone().into_inner();
    let db = conn.into_inner();

    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session.clone(), Action::CreateCustomer);

    match Customer::insert(new_transaction, session.clone(), db).await {
        Ok(data) => match Customer::fetch_by_id(&data.last_insert_id, session, db).await {
            Ok(res) => Ok(Json(res)),
            Err(reason) => {
                println!("[dberr]: {}", reason);
                Err(ErrorResponse::create_error(&format!(
                    "Fetch for customer failed, reason: {}",
                    reason
                )))
            }
        },
        Err(reason) => {
            println!("[dberr]: {}", reason);
            Err(ErrorResponse::input_error())
        }
    }
}
