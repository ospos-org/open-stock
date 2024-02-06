use okapi::openapi3::OpenApi;
use crate::{check_permissions, Session};
use crate::methods::{Action, ContactInformation, CustomerWithTransactionsOut, Error, Transaction};
use crate::pool::{InternalDb};
use rocket::get;
use rocket::serde::json::Json;
use rocket::{post};
use rocket_okapi::{openapi, openapi_get_routes_spec};
use rocket_okapi::settings::OpenApiSettings;
use crate::catchers::Validated;
use crate::guards::Convert;
use super::{Customer, CustomerInput};

pub fn documented_routes(settings: &OpenApiSettings) -> (Vec<rocket::Route>, OpenApi) {
    openapi_get_routes_spec![
        settings:
        get,
        delete,
        get_by_name,
        get_by_phone,
        get_by_addr,
        get_recent,
        create,
        update,
        generate,
        search_query,
        update_contact_info,
        update_by_input,
        find_related_transactions
    ]
}

#[openapi(tag = "Customer")]
#[get("/<id>")]
pub async fn get(
    db: InternalDb,
    id: &str,
    session: Session,
) -> Convert<Customer> {
    check_permissions!(session.clone(), Action::FetchCustomer);
    Customer::fetch_by_id(id, session, &db.0).await.into()
}

#[openapi(tag = "Customer")]
#[post("/delete/<id>")]
pub async fn delete(
    db: InternalDb,
    id: &str,
    session: Session,
) -> Result<(), Error> {
    check_permissions!(session.clone(), Action::AccessAdminPanel);
    Customer::delete(id, session, &db.0).await.map(|_| ())
}

#[openapi(tag = "Customer")]
#[get("/recent")]
pub async fn get_recent(
    db: InternalDb,
    session: Session,
) -> Convert<Vec<Customer>> {
    check_permissions!(session.clone(), Action::FetchCustomer);
    Customer::fetch_recent(session, &db.0).await.into()
}

#[openapi(tag = "Customer")]
#[get("/name/<name>")]
pub async fn get_by_name(
    db: InternalDb,
    session: Session,
    name: &str,
) -> Convert<Vec<Customer>> {
    check_permissions!(session.clone(), Action::FetchCustomer);
    Customer::fetch_by_name(name, session, &db.0).await.into()
}

/// Will search by both name, phone and email.
#[openapi(tag = "Customer")]
#[get("/search/<query>")]
pub async fn search_query(
    db: InternalDb,
    session: Session,
    query: &str,
) -> Convert<Vec<CustomerWithTransactionsOut>> {
    check_permissions!(session.clone(), Action::FetchCustomer);
    Customer::search(query, session, &db.0).await.into()
}

#[openapi(tag = "Customer")]
#[get("/transactions/<id>")]
pub async fn find_related_transactions(
    db: InternalDb,
    session: Session,
    id: &str,
) -> Convert<Vec<Transaction>> {
    check_permissions!(session.clone(), Action::FetchCustomer);
    Transaction::fetch_by_client_id(id, session, &db.0).await.into()
}

#[openapi(tag = "Customer")]
#[get("/phone/<phone>")]
pub async fn get_by_phone(
    db: InternalDb,
    session: Session,
    phone: &str,
) -> Convert<Vec<Customer>> {
    check_permissions!(session.clone(), Action::FetchCustomer);
    Customer::fetch_by_phone(phone, session, &db.0).await.into()
}

#[openapi(tag = "Customer")]
#[get("/addr/<addr>")]
pub async fn get_by_addr(
    db: InternalDb,
    session: Session,
    addr: &str,
) -> Convert<Vec<Customer>> {
    check_permissions!(session.clone(), Action::FetchCustomer);
    Customer::fetch_by_addr(addr, session, &db.0).await.into()
}

#[openapi(tag = "Customer")]
#[post("/generate")]
async fn generate(
    db: InternalDb,
    session: Session,
) -> Convert<Customer> {
    check_permissions!(session.clone(), Action::GenerateTemplateContent);
    Customer::generate(session, &db.0).await.into()
}

#[openapi(tag = "Customer")]
#[post("/<id>", data = "<input_data>")]
async fn update(
    db: InternalDb,
    session: Session,
    input_data: Validated<Json<Customer>>,
    id: &str,
) -> Convert<Customer> {
    check_permissions!(session.clone(), Action::ModifyCustomer);
    Customer::update(input_data.data(), session, id, &db.0).await.into()
}

#[openapi(tag = "Customer")]
#[post("/input/<id>", data = "<input_data>")]
async fn update_by_input(
    db: InternalDb,
    id: &str,
    session: Session,
    input_data: Validated<Json<CustomerInput>>,
) -> Convert<Customer> {
    check_permissions!(session.clone(), Action::ModifyCustomer);
    Customer::update_by_input(input_data.data(), session, id, &db.0).await.into()
}

#[openapi(tag = "Customer")]
#[post("/contact/<id>", data = "<input_data>")]
async fn update_contact_info(
    db: InternalDb,
    session: Session,
    input_data: Validated<Json<ContactInformation>>,
    id: &str,
) -> Convert<Customer> {
    check_permissions!(session.clone(), Action::ModifyCustomer);
    Customer::update_contact_information(input_data.data(), id, session, &db.0).await.into()
}

#[openapi(tag = "Customer")]
#[post("/", data = "<input_data>")]
pub async fn create(
    db: InternalDb,
    session: Session,
    input_data: Validated<Json<CustomerInput>>,
) -> Result<Json<Customer>, Error> {
    check_permissions!(session.clone(), Action::CreateCustomer);

    let data = Customer::insert(input_data.data(), session.clone(), &db.0).await?;
    let converted: Convert<Customer> = Customer::fetch_by_id(&data.last_insert_id, session, &db.0).await.into();
    converted.0
}
