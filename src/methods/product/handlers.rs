use okapi::openapi3::OpenApi;
use crate::check_permissions;
use crate::methods::{cookie_status_wrapper, Action, Error, ErrorResponse};
use crate::pool::Db;
use rocket::get;
use rocket::http::CookieJar;
use rocket::serde::json::Json;
use rocket::{post};
use rocket_okapi::{openapi, openapi_get_routes_spec};
use rocket_db_pools::Connection;
use rocket_okapi::settings::OpenApiSettings;

use super::{Product, ProductWPromotion, Promotion, PromotionInput};

pub fn documented_routes(settings: &OpenApiSettings) -> (Vec<rocket::Route>, OpenApi) {
    openapi_get_routes_spec![
        settings:
        get,
        get_with_associated_promotions,
        get_by_name,
        get_by_name_exact,
        create,
        update,
        generate,
        search_query,
        get_promotion,
        get_promotion_by_query,
        create_promotion,
        update_promotion,
        generate_promotion,
        search_with_associated_promotions
    ]
}

#[openapi(tag = "Product")]
#[get("/<id>")]
pub async fn get(
    conn: Connection<Db>,
    id: i32,
    cookies: &CookieJar<'_>,
) -> Result<Json<Product>, Error> {
    let db = conn.into_inner();

    let session = cookie_status_wrapper(&db, cookies).await?;
    check_permissions!(session.clone(), Action::FetchProduct);

    match Product::fetch_by_id(&id.to_string(), session, &db).await {
        Ok(product) => Ok(Json(product)),
        Err(reason) => Err(ErrorResponse::db_err(reason)),
    }
}

#[openapi(tag = "Product")]
#[get("/with_promotions/<id>")]
pub async fn get_with_associated_promotions(
    conn: Connection<Db>,
    id: i32,
    cookies: &CookieJar<'_>,
) -> Result<Json<ProductWPromotion>, Error> {
    let db = conn.into_inner();

    let session = cookie_status_wrapper(&db, cookies).await?;
    check_permissions!(session.clone(), Action::FetchProduct);

    match Product::fetch_by_id_with_promotion(&id.to_string(), session, &db).await {
        Ok(product) => Ok(Json(product)),
        Err(reason) => Err(ErrorResponse::db_err(reason)),
    }
}

#[openapi(tag = "Product")]
#[get("/name/<name>")]
pub async fn get_by_name(
    conn: Connection<Db>,
    name: &str,
    cookies: &CookieJar<'_>,
) -> Result<Json<Vec<Product>>, Error> {
    let db = conn.into_inner();

    let session = cookie_status_wrapper(&db, cookies).await?;
    check_permissions!(session.clone(), Action::FetchProduct);

    match Product::fetch_by_name(name, session, &db).await {
        Ok(products) => Ok(Json(products)),
        Err(reason) => Err(ErrorResponse::db_err(reason)),
    }
}

/// References exact name
#[openapi(tag = "Product")]
#[get("/!name/<name>")]
pub async fn get_by_name_exact(
    conn: Connection<Db>,
    name: &str,
    cookies: &CookieJar<'_>,
) -> Result<Json<Vec<Product>>, Error> {
    let db = conn.into_inner();

    let session = cookie_status_wrapper(&db, cookies).await?;
    check_permissions!(session.clone(), Action::FetchProduct);

    match Product::fetch_by_name_exact(name, session, &db).await {
        Ok(products) => Ok(Json(products)),
        Err(reason) => Err(ErrorResponse::db_err(reason)),
    }
}

/// Will search by both name, phone and email.
#[openapi(tag = "Product")]
#[get("/search/<query>")]
pub async fn search_query(
    conn: Connection<Db>,
    query: &str,
    cookies: &CookieJar<'_>,
) -> Result<Json<Vec<Product>>, Error> {
    let db = conn.into_inner();

    let session = cookie_status_wrapper(&db, cookies).await?;
    check_permissions!(session.clone(), Action::FetchProduct);

    match Product::search(query, session, &db).await {
        Ok(products) => Ok(Json(products)),
        Err(reason) => Err(ErrorResponse::db_err(reason)),
    }
}

#[openapi(tag = "Product")]
#[get("/search/with_promotions/<query>")]
pub async fn search_with_associated_promotions(
    conn: Connection<Db>,
    query: &str,
    cookies: &CookieJar<'_>,
) -> Result<Json<Vec<ProductWPromotion>>, Error> {
    let db = conn.into_inner();

    let session = cookie_status_wrapper(&db, cookies).await?;
    check_permissions!(session.clone(), Action::FetchProduct);

    match Product::search_with_promotion(query, session, &db).await {
        Ok(products) => Ok(Json(products)),
        Err(reason) => Err(ErrorResponse::db_err(reason)),
    }
}

#[openapi(tag = "Product")]
#[post("/<id>", data = "<input_data>")]
async fn update(
    conn: Connection<Db>,
    id: &str,
    input_data: Json<Product>,
    cookies: &CookieJar<'_>,
) -> Result<Json<Product>, Error> {
    let input_data = input_data.clone().into_inner();
    let db = conn.into_inner();

    let session = cookie_status_wrapper(&db, cookies).await?;
    check_permissions!(session.clone(), Action::ModifyProduct);

    match Product::update(input_data, session, id, &db).await {
        Ok(res) => Ok(Json(res)),
        Err(reason) => Err(ErrorResponse::db_err(reason)),
    }
}

#[openapi(tag = "Product")]
#[post("/", data = "<input_data>")]
pub async fn create(
    conn: Connection<Db>,
    input_data: Json<Product>,
    cookies: &CookieJar<'_>,
) -> Result<Json<Product>, Error> {
    let new_transaction = input_data.clone().into_inner();
    let db = conn.into_inner();

    let session = cookie_status_wrapper(&db, cookies).await?;
    check_permissions!(session.clone(), Action::CreateProduct);

    match Product::insert(new_transaction, session.clone(), &db).await {
        Ok(data) =>
            match Product::fetch_by_id(
                &data.last_insert_id, session, &db
            ).await {
                Ok(res) => Ok(Json(res)),
                Err(reason) => {
                    println!("[dberr]: {}", reason);
                    Err(ErrorResponse::db_err(reason))
                }
            },
        Err(reason) => {
            println!("[dberr]: {}", reason);
            Err(ErrorResponse::db_err(reason))
        }
    }
}

#[openapi(tag = "Product")]
#[post("/generate")]
async fn generate(
    conn: Connection<Db>,
    cookies: &CookieJar<'_>,
) -> Result<Json<Vec<Product>>, Error> {
    let db = conn.into_inner();

    let session = cookie_status_wrapper(&db, cookies).await?;
    check_permissions!(session.clone(), Action::GenerateTemplateContent);

    match Product::generate(session, &db).await {
        Ok(res) => Ok(Json(res)),
        Err(err) => Err(ErrorResponse::db_err(err)),
    }
}

#[openapi(tag = "Product")]
#[get("/promotion/<id>")]
pub async fn get_promotion(
    conn: Connection<Db>,
    id: i32,
    cookies: &CookieJar<'_>,
) -> Result<Json<Promotion>, Error> {
    let db = conn.into_inner();

    let session = cookie_status_wrapper(&db, cookies).await?;
    check_permissions!(session.clone(), Action::FetchProduct);

    let product = Promotion::fetch_by_id(&id.to_string(), session, &db)
        .await
        .unwrap();
    Ok(Json(product))
}

#[openapi(tag = "Product")]
#[get("/promotion/search/<query>")]
pub async fn get_promotion_by_query(
    conn: Connection<Db>,
    query: &str,
    cookies: &CookieJar<'_>,
) -> Result<Json<Vec<Promotion>>, Error> {
    let db = conn.into_inner();

    let session = cookie_status_wrapper(&db, cookies).await?;
    check_permissions!(session.clone(), Action::FetchProduct);

    let product = Promotion::fetch_by_query(query, session, &db).await.unwrap();
    Ok(Json(product))
}

#[openapi(tag = "Product")]
#[post("/promotion/<id>", data = "<input_data>")]
async fn update_promotion(
    conn: Connection<Db>,
    id: &str,
    input_data: Json<PromotionInput>,
    cookies: &CookieJar<'_>,
) -> Result<Json<Promotion>, Error> {
    let input_data = input_data.clone().into_inner();
    let db = conn.into_inner();

    let session = cookie_status_wrapper(&db, cookies).await?;
    check_permissions!(session.clone(), Action::ModifyProduct);

    match Promotion::update(input_data, session, id, &db).await {
        Ok(res) => Ok(Json(res)),
        Err(reason) => Err(ErrorResponse::db_err(reason)),
    }
}

#[openapi(tag = "Product")]
#[post("/promotion", data = "<input_data>")]
pub async fn create_promotion(
    conn: Connection<Db>,
    input_data: Json<PromotionInput>,
    cookies: &CookieJar<'_>,
) -> Result<Json<Promotion>, Error> {
    let new_promotion = input_data.clone().into_inner();
    let db = conn.into_inner();

    let session = cookie_status_wrapper(&db, cookies).await?;
    check_permissions!(session.clone(), Action::CreateProduct);

    match Promotion::insert(new_promotion, session.clone(), &db).await {
        Ok(data) =>
            match Promotion::fetch_by_id(
                &data.last_insert_id, session, &db
            ).await {
                Ok(res) => Ok(Json(res)),
                Err(reason) => {
                    println!("[dberr]: {}", reason);
                    Err(ErrorResponse::db_err(reason))
                }
            },
        Err(reason) => {
            println!("[dberr]: {}", reason);
            Err(ErrorResponse::db_err(reason))
        }
    }
}

#[openapi(tag = "Product")]
#[post("/generate/promotion")]
async fn generate_promotion(
    conn: Connection<Db>,
    cookies: &CookieJar<'_>,
) -> Result<Json<Vec<Promotion>>, Error> {
    let db = conn.into_inner();

    let session = cookie_status_wrapper(&db, cookies).await?;
    check_permissions!(session.clone(), Action::GenerateTemplateContent);

    match Promotion::generate(session, &db).await {
        Ok(res) => Ok(Json(res)),
        Err(err) => Err(ErrorResponse::db_err(err)),
    }
}
