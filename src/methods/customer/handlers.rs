use rocket::http::CookieJar;
use rocket::{get};
use rocket::{routes, post};
use rocket::serde::json::Json;
use sea_orm_rocket::{Connection};
use crate::check_permissions;
use crate::methods::{ContactInformation, cookie_status_wrapper, Action, Error, ErrorResponse, CustomerWithTransactionsOut, Transaction};
use crate::pool::Db;

use super::{Customer, CustomerInput};

pub fn routes() -> Vec<rocket::Route> {
    routes![get, get_by_name, get_by_phone, get_by_addr, create, update, generate, search_query, update_contact_info, find_related_transactions]
}

#[get("/<id>")]
pub async fn get(conn: Connection<'_, Db>, id: &str, cookies: &CookieJar<'_>) -> Result<Json<Customer>, Error> {
    let db = conn.into_inner();
    
    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session, Action::FetchCustomer);

    match Customer::fetch_by_id(&id, db).await {
        Ok(customers) => Ok(Json(customers)),
        Err(err) => Err(ErrorResponse::db_err(err)),
    }
}

#[get("/name/<name>")]
pub async fn get_by_name(conn: Connection<'_, Db>, name: &str, cookies: &CookieJar<'_>) -> Result<Json<Vec<Customer>>, Error> {
    let db = conn.into_inner();
    
    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session, Action::FetchCustomer);

    match Customer::fetch_by_name(name, db).await {
        Ok(customers) => Ok(Json(customers)),
        Err(err) => Err(ErrorResponse::db_err(err)),
    }
}

/// Will search by both name, phone and email.
#[get("/search/<query>")]
pub async fn search_query(conn: Connection<'_, Db>, query: &str, cookies: &CookieJar<'_>) -> Result<Json<Vec<CustomerWithTransactionsOut>>, Error> {
    let db = conn.into_inner();
    
    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session, Action::FetchCustomer);

    match Customer::search(query, db).await {
        Ok(customers) => Ok(Json(customers)),
        Err(err) => Err(ErrorResponse::db_err(err)),
    }
}

#[get("/transactions/<id>")]
pub async fn find_related_transactions(conn: Connection<'_, Db>, id: &str, cookies: &CookieJar<'_>) -> Result<Json<Vec<Transaction>>, Error> {
    let db = conn.into_inner();
    
    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session, Action::FetchCustomer);

    match Transaction::fetch_by_client_id(id, db).await {
        Ok(transactions) => Ok(Json(transactions)),
        Err(err) => Err(ErrorResponse::db_err(err)),
    }
}

#[get("/phone/<phone>")]
pub async fn get_by_phone(conn: Connection<'_, Db>, phone: &str, cookies: &CookieJar<'_>) -> Result<Json<Vec<Customer>>, Error> {
    let db = conn.into_inner();
    let new_phone = phone.clone();
    
    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session, Action::FetchCustomer);

    match Customer::fetch_by_phone(new_phone, db).await {
        Ok(customer) => Ok(Json(customer)),
        Err(err) => Err(ErrorResponse::db_err(err)),
    }
}

#[get("/addr/<addr>")]
pub async fn get_by_addr(conn: Connection<'_, Db>, addr: &str, cookies: &CookieJar<'_>) -> Result<Json<Vec<Customer>>, Error> {
    let db = conn.into_inner();
    let new_addr = addr.clone();

    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session, Action::FetchCustomer);

    match Customer::fetch_by_addr(new_addr, db).await {
        Ok(customers) => Ok(Json(customers)),
        Err(err) => Err(ErrorResponse::db_err(err)),
    }
}

#[post("/generate")]
async fn generate(
    conn: Connection<'_, Db>, 
    cookies: &CookieJar<'_>
) -> Result<Json<Customer>, Error> {
    let db = conn.into_inner();
    
    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session, Action::GenerateTemplateContent);

    match Customer::generate(db).await {
        Ok(res) => Ok(Json(res)),
        Err(err) => Err(ErrorResponse::db_err(err))
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
    check_permissions!(session, Action::ModifyCustomer);

    match Customer::update(input_data, id, db).await {
        Ok(res) => {
            Ok(Json(res))
        },
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

    check_permissions!(session, Action::ModifyCustomer);

    match Customer::update_contact_information(input_data, id, db).await {
        Ok(res) => {
            Ok(Json(res))
        },
        Err(_) => Err(ErrorResponse::input_error()),
    }
}

#[post("/", data = "<input_data>")]
pub async fn create(conn: Connection<'_, Db>, cookies: &CookieJar<'_>, input_data: Json<CustomerInput>) -> Result<Json<Customer>, Error> {
    let new_transaction = input_data.clone().into_inner();
    let db = conn.into_inner();
    
    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session, Action::CreateCustomer);

    match Customer::insert(new_transaction, db).await {
        Ok(data) => {
            match Customer::fetch_by_id(&data.last_insert_id, db).await {
                Ok(res) => {
                    Ok(Json(res))
                },  
                Err(reason) => {
                    println!("[dberr]: {}", reason);
                    Err(ErrorResponse::create_error(&format!("Fetch for customer failed, reason: {}", reason)))
                }
            }
        },
        Err(reason) => {
            println!("[dberr]: {}", reason);
            Err(ErrorResponse::input_error())
        },
    }
}