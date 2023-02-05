use rocket::http::CookieJar;
use rocket::{get};
use rocket::{routes, post};
use rocket::serde::json::Json;
use sea_orm_rocket::{Connection};
use crate::check_permissions;
use crate::methods::{cookie_status_wrapper, Action, Error, ErrorResponse};
use crate::pool::Db;

use super::{Product, Promotion, PromotionInput, ProductWPromotion};

pub fn routes() -> Vec<rocket::Route> {
    routes![
        get, get_with_associated_promotions, get_by_name, get_by_name_exact, create, update, generate, search_query,
        get_promotion, get_promotion_by_query, create_promotion, update_promotion, generate_promotion, search_with_associated_promotions
    ]
}

#[get("/<id>")]
pub async fn get(conn: Connection<'_, Db>, id: i32, cookies: &CookieJar<'_>) -> Result<Json<Product>, Error> {
    let db = conn.into_inner();

    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session.clone(), Action::FetchProduct);

    match Product::fetch_by_id(&id.to_string(), db).await {
        Ok(product) => Ok(Json(product)),
        Err(reason) => Err(ErrorResponse::db_err(reason))
    }
}

#[get("/with_promotions/<id>")]
pub async fn get_with_associated_promotions(conn: Connection<'_, Db>, id: i32, cookies: &CookieJar<'_>) -> Result<Json<ProductWPromotion>, Error> {
    let db = conn.into_inner();

    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session.clone(), Action::FetchProduct);

    match Product::fetch_by_id_with_promotion(&id.to_string(), db).await {
        Ok(product) => Ok(Json(product)),
        Err(reason) => Err(ErrorResponse::db_err(reason))
    }
}

#[get("/name/<name>")]
pub async fn get_by_name(conn: Connection<'_, Db>, name: &str, cookies: &CookieJar<'_>) -> Result<Json<Vec<Product>>, Error> {
    let db = conn.into_inner();
    
    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session.clone(), Action::FetchProduct);

    match Product::fetch_by_name(name, db).await {
        Ok(products) => Ok(Json(products)),
        Err(reason) => Err(ErrorResponse::db_err(reason))
    }
}

/// References exact name
#[get("/!name/<name>")]
pub async fn get_by_name_exact(conn: Connection<'_, Db>, name: &str, cookies: &CookieJar<'_>) -> Result<Json<Vec<Product>>, Error> {
    let db = conn.into_inner();
    
    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session.clone(), Action::FetchProduct);

    match Product::fetch_by_name_exact(name, db).await {
        Ok(products) => Ok(Json(products)),
        Err(reason) => Err(ErrorResponse::db_err(reason))
    }
}

/// Will search by both name, phone and email.
#[get("/search/<query>")]
pub async fn search_query(conn: Connection<'_, Db>, query: &str, cookies: &CookieJar<'_>) -> Result<Json<Vec<Product>>, Error> {
    let db = conn.into_inner();

    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session.clone(), Action::FetchProduct);

    match Product::search(query, db).await {
        Ok(products) => Ok(Json(products)),
        Err(reason) => Err(ErrorResponse::db_err(reason))
    }
}

#[get("/search/with_promotions/<query>")]
pub async fn search_with_associated_promotions(conn: Connection<'_, Db>, query: &str, cookies: &CookieJar<'_>) -> Result<Json<Vec<ProductWPromotion>>, Error> {
    let db = conn.into_inner();

    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session.clone(), Action::FetchProduct);

    match Product::search_with_promotion(query, db).await {
        Ok(products) => Ok(Json(products)),
        Err(reason) => Err(ErrorResponse::db_err(reason))
    }
}

#[post("/<id>", data = "<input_data>")]
async fn update(
    conn: Connection<'_, Db>,
    id: &str,
    input_data: Json<Product>, 
    cookies: &CookieJar<'_>
) -> Result<Json<Product>, Error> {
    let input_data = input_data.clone().into_inner();
    let db = conn.into_inner();
    
    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session.clone(), Action::ModifyProduct);

    match Product::update(input_data, id, db).await {
        Ok(res) => {
            Ok(Json(res))
        },
        Err(reason) => Err(ErrorResponse::db_err(reason))
    }
}

#[post("/", data = "<input_data>")]
pub async fn create(conn: Connection<'_, Db>, input_data: Json<Product>, cookies: &CookieJar<'_>) -> Result<Json<Product>, Error> {
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
                    Err(ErrorResponse::db_err(reason))
                }
            }
        },
        Err(reason) => {
            println!("[dberr]: {}", reason);
            Err(ErrorResponse::db_err(reason))
        },
    }
}

#[post("/generate")]
async fn generate(
    conn: Connection<'_, Db>,
    cookies: &CookieJar<'_>
) -> Result<Json<Vec<Product>>, Error> {
    let db = conn.into_inner();

    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session.clone(), Action::GenerateTemplateContent);

    match Product::generate(db).await {
        Ok(res) => Ok(Json(res)),
        Err(err) => Err(ErrorResponse::db_err(err))
    }
}

#[get("/promotion/<id>")]
pub async fn get_promotion(conn: Connection<'_, Db>, id: i32, cookies: &CookieJar<'_>) -> Result<Json<Promotion>, Error> {
    let db = conn.into_inner();

    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session.clone(), Action::FetchProduct);

    let product = Promotion::fetch_by_id(&id.to_string(), db).await.unwrap();
    Ok(Json(product))
}

#[get("/promotion/search/<query>")]
pub async fn get_promotion_by_query(conn: Connection<'_, Db>, query: &str, cookies: &CookieJar<'_>) -> Result<Json<Vec<Promotion>>, Error> {
    let db = conn.into_inner();

    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session.clone(), Action::FetchProduct);

    let product = Promotion::fetch_by_query(&query.to_string(), db).await.unwrap();
    Ok(Json(product))
}


#[post("/promotion/<id>", data = "<input_data>")]
async fn update_promotion(
    conn: Connection<'_, Db>,
    id: &str,
    input_data: Json<PromotionInput>, 
    cookies: &CookieJar<'_>
) -> Result<Json<Promotion>, Error> {
    let input_data = input_data.clone().into_inner();
    let db = conn.into_inner();
    
    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session.clone(), Action::ModifyProduct);

    match Promotion::update(input_data, id, db).await {
        Ok(res) => {
            Ok(Json(res))
        },
        Err(reason) => Err(ErrorResponse::db_err(reason))
    }
}

#[post("/promotion", data = "<input_data>")]
pub async fn create_promotion(conn: Connection<'_, Db>, input_data: Json<PromotionInput>, cookies: &CookieJar<'_>) -> Result<Json<Promotion>, Error> {
    let new_promotion = input_data.clone().into_inner();
    let db = conn.into_inner();
    
    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session.clone(), Action::CreateProduct);
    
    match Promotion::insert(new_promotion, db).await {
        Ok(data) => {
            match Promotion::fetch_by_id(&data.last_insert_id, db).await {
                Ok(res) => {
                    Ok(Json(res))
                },  
                Err(reason) => {
                    println!("[dberr]: {}", reason);
                    Err(ErrorResponse::db_err(reason))
                }
            }
        },
        Err(reason) => {
            println!("[dberr]: {}", reason);
            Err(ErrorResponse::db_err(reason))
        },
    }
}

#[post("/generate/promotion")]
async fn generate_promotion(
    conn: Connection<'_, Db>,
    cookies: &CookieJar<'_>
) -> Result<Json<Vec<Promotion>>, Error> {
    let db = conn.into_inner();

    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session.clone(), Action::GenerateTemplateContent);

    match Promotion::generate(db).await {
        Ok(res) => Ok(Json(res)),
        Err(err) => Err(ErrorResponse::db_err(err))
    }
}
