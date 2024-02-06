use crate::catchers::Validated;
use crate::{check_permissions, Session};
use crate::methods::{Action, Error};
use crate::pool::{InternalDb};
use okapi::openapi3::OpenApi;
use rocket::get;
use rocket::post;
use rocket::serde::json::Json;
use rocket_okapi::settings::OpenApiSettings;
use rocket_okapi::{openapi, openapi_get_routes_spec};
use crate::guards::Convert;
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
    session: Session,
    db: InternalDb,
    id: i32,
) -> Convert<Product> {
    check_permissions!(session.clone(), Action::FetchProduct);
    Product::fetch_by_id(&id.to_string(), session, &db.0).await.into()
}

#[openapi(tag = "Product")]
#[get("/with_promotions/<id>")]
pub async fn get_with_associated_promotions(
    db: InternalDb,
    session: Session,
    id: i32,
) -> Convert<ProductWPromotion> {
    check_permissions!(session.clone(), Action::FetchProduct);
    Product::fetch_by_id_with_promotion(&id.to_string(), session, &db.0).await.into()
}

#[openapi(tag = "Product")]
#[get("/name/<name>")]
pub async fn get_by_name(
    db: InternalDb,
    session: Session,
    name: &str,
) -> Convert<Vec<Product>> {
    check_permissions!(session.clone(), Action::FetchProduct);
    Product::fetch_by_name(name, session, &db.0).await.into()
}

/// References exact name
#[openapi(tag = "Product")]
#[get("/!name/<name>")]
pub async fn get_by_name_exact(
    db: InternalDb,
    session: Session,
    name: &str
) -> Convert<Vec<Product>> {
    check_permissions!(session.clone(), Action::FetchProduct);
    Product::fetch_by_name_exact(name, session, &db.0).await.into()
}

/// Will search by both name, phone and email.
#[openapi(tag = "Product")]
#[get("/search/<query>")]
pub async fn search_query(
    db: InternalDb,
    session: Session,
    query: &str,
) -> Convert<Vec<Product>> {
    check_permissions!(session.clone(), Action::FetchProduct);
    Product::search(query, session, &db.0).await.into()
}

#[openapi(tag = "Product")]
#[get("/search/with_promotions/<query>")]
pub async fn search_with_associated_promotions(
    db: InternalDb,
    session: Session,
    query: &str,
) -> Convert<Vec<ProductWPromotion>> {
    check_permissions!(session.clone(), Action::FetchProduct);
    Product::search_with_promotion(query, session, &db.0).await.into()
}

#[openapi(tag = "Product")]
#[post("/<id>", data = "<input_data>")]
async fn update(
    db: InternalDb,
    session: Session,
    input_data: Validated<Json<Product>>,
    id: &str,
) -> Convert<Product> {
    check_permissions!(session.clone(), Action::ModifyProduct);
    Product::update(input_data.data(), session, id, &db.0).await.into()
}

#[openapi(tag = "Product")]
#[post("/", data = "<input_data>")]
pub async fn create(
    db: InternalDb,
    input_data: Validated<Json<Product>>,
    session: Session,
) -> Result<Json<Product>, Error> {
    check_permissions!(session.clone(), Action::CreateProduct);

    let data = Product::insert(input_data.data(), session.clone(), &db.0).await?;
    let converted: Convert<Product> = Product::fetch_by_id(&data.last_insert_id, session, &db.0).await.into();
    converted.0
}

#[openapi(tag = "Product")]
#[post("/generate")]
async fn generate(
    db: InternalDb,
    session: Session,
) -> Convert<Vec<Product>> {
    check_permissions!(session.clone(), Action::GenerateTemplateContent);
    Product::generate(session, &db.0).await.into()
}

#[openapi(tag = "Product")]
#[get("/promotion/<id>")]
pub async fn get_promotion(
    db: InternalDb,
    session: Session,
    id: i32,
) -> Convert<Promotion> {
    check_permissions!(session.clone(), Action::FetchProduct);
    Promotion::fetch_by_id(&id.to_string(), session, &db.0).await.into()
}

#[openapi(tag = "Product")]
#[get("/promotion/search/<query>")]
pub async fn get_promotion_by_query(
    db: InternalDb,
    session: Session,
    query: &str,
) -> Convert<Vec<Promotion>> {
    check_permissions!(session.clone(), Action::FetchProduct);
    Promotion::fetch_by_query(query, session, &db.0).await.into()
}

#[openapi(tag = "Product")]
#[post("/promotion/<id>", data = "<input_data>")]
async fn update_promotion(
    input_data: Validated<Json<PromotionInput>>,
    db: InternalDb,
    session: Session,
    id: &str,
) -> Convert<Promotion> {
    check_permissions!(session.clone(), Action::ModifyProduct);
    Promotion::update(input_data.data(), session, id, &db.0).await.into()
}

#[openapi(tag = "Product")]
#[post("/promotion", data = "<input_data>")]
pub async fn create_promotion(
    db: InternalDb,
    input_data: Validated<Json<PromotionInput>>,
    session: Session
) -> Result<Json<Promotion>, Error> {
    check_permissions!(session.clone(), Action::CreateProduct);

    let data = Promotion::insert(input_data.data(), session.clone(), &db.0).await?;
    let converted: Convert<Promotion> = Promotion::fetch_by_id(&data.last_insert_id, session, &db.0).await.into();
    converted.0
}

#[openapi(tag = "Product")]
#[post("/generate/promotion")]
async fn generate_promotion(
    db: InternalDb,
    session: Session
) -> Convert<Vec<Promotion>> {
    check_permissions!(session.clone(), Action::GenerateTemplateContent);
    Promotion::generate(session, &db.0).await.into()
}
