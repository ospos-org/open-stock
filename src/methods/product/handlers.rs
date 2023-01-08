use rocket::http::CookieJar;
use rocket::{http::Status, get};
use rocket::{routes, post, patch};
use rocket::serde::json::Json;
use sea_orm_rocket::{Connection};
use crate::check_permissions;
use crate::methods::{cookie_status_wrapper, Action};
use crate::pool::Db;

use super::Product;

pub fn routes() -> Vec<rocket::Route> {
    routes![get, get_by_name, get_by_name_exact, create, update, generate, search_query]
}

#[get("/<id>")]
pub async fn get(conn: Connection<'_, Db>, id: i32, cookies: &CookieJar<'_>) -> Result<Json<Product>, Status> {
    let db = conn.into_inner();

    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session.clone(), Action::FetchProduct);

    let product = Product::fetch_by_id(&id.to_string(), db).await.unwrap();
    Ok(Json(product))
}

#[get("/name/<name>")]
pub async fn get_by_name(conn: Connection<'_, Db>, name: &str, cookies: &CookieJar<'_>) -> Result<Json<Vec<Product>>, Status> {
    let db = conn.into_inner();
    
    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session.clone(), Action::FetchProduct);

    let product = Product::fetch_by_name(name, db).await.unwrap();
    Ok(Json(product))
}

/// References exact name
#[get("/!name/<name>")]
pub async fn get_by_name_exact(conn: Connection<'_, Db>, name: &str, cookies: &CookieJar<'_>) -> Result<Json<Vec<Product>>, Status> {
    let db = conn.into_inner();
    
    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session.clone(), Action::FetchProduct);

    let product = Product::fetch_by_name_exact(name, db).await.unwrap();
    Ok(Json(product))
}

/// Will search by both name, phone and email.
#[get("/search/<query>")]
pub async fn search_query(conn: Connection<'_, Db>, query: &str, cookies: &CookieJar<'_>) -> Result<Json<Vec<Product>>, Status> {
    let db = conn.into_inner();

    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session.clone(), Action::FetchProduct);

    let employee = Product::search(query, db).await.unwrap();
    Ok(Json(employee))
}

#[post("/<id>", data = "<input_data>")]
async fn update(
    conn: Connection<'_, Db>,
    id: &str,
    input_data: Json<Product>, 
    cookies: &CookieJar<'_>
) -> Result<Json<Product>, Status> {
    let input_data = input_data.clone().into_inner();
    let db = conn.into_inner();
    
    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session.clone(), Action::ModifyProduct);

    match Product::update(input_data, id, db).await {
        Ok(res) => {
            Ok(Json(res))
        },
        Err(_) => Err(Status::BadRequest),
    }
}

#[post("/", data = "<input_data>")]
pub async fn create(conn: Connection<'_, Db>, input_data: Json<Product>, cookies: &CookieJar<'_>) -> Result<Json<Product>, Status> {
    let new_transaction = input_data.clone().into_inner();
    let db = conn.into_inner();
    
    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session.clone(), Action::CreateProduct);
    
    match Product::insert(new_transaction, db).await {
        Ok(data) => {
            match Product::fetch_by_id(&data.last_insert_id, db).await {
                Ok(res) => {
                    Ok(Json(res))
                },  
                Err(reason) => {
                    println!("[dberr]: {}", reason);
                    Err(Status::InternalServerError)
                }
            }
        },
        Err(reason) => {
            println!("[dberr]: {}", reason);
            Err(Status::InternalServerError)
        },
    }
}

#[patch("/generate")]
async fn generate(
    conn: Connection<'_, Db>,
    cookies: &CookieJar<'_>
) -> Result<Json<Product>, Status> {
    let db = conn.into_inner();

    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session.clone(), Action::GenerateTemplateContent);

    match Product::generate(db).await {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(Status::BadRequest)
    }
}