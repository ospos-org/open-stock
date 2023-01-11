use std::time::Duration;

use chrono::{Utc, Duration as ChronoDuration};
use rocket::http::{CookieJar, Cookie, SameSite};
use rocket::time::{OffsetDateTime};
use rocket::{http::Status, get};
use rocket::{routes, post};
use rocket::serde::json::Json;
use sea_orm::{EntityTrait, Set};
use sea_orm_rocket::{Connection};
use serde::{Serialize, Deserialize};
use serde_json::json;
use uuid::Uuid;
use crate::check_permissions;
use crate::entities::session;
use crate::methods::{Name, History, cookie_status_wrapper};
use crate::pool::Db;

use super::{Employee, EmployeeInput, Attendance, TrackType, Action};

pub fn routes() -> Vec<rocket::Route> {
    routes![get, get_by_name, get_by_name_exact, get_by_level, create, update, log, generate, auth]
}

#[get("/<id>")]
pub async fn get(conn: Connection<'_, Db>, id: &str, cookies: &CookieJar<'_>) -> Result<Json<Employee>, Status> {
    let db = conn.into_inner();

    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session.clone(), Action::FetchEmployee);

    if session.employee.id == id {
        Ok(Json(session.employee))
    }else {
        let employee = Employee::fetch_by_id(&id.to_string(), db).await.unwrap();
        Ok(Json(employee))
    }
}

#[get("/name/<name>")]
pub async fn get_by_name(conn: Connection<'_, Db>, name: &str, cookies: &CookieJar<'_>) -> Result<Json<Vec<Employee>>, Status> {
    let db = conn.into_inner();
    
    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session, Action::FetchEmployee);

    let employee = Employee::fetch_by_name(name, db).await.unwrap();
    Ok(Json(employee))
}

#[get("/!name", data = "<name>")]
pub async fn get_by_name_exact(conn: Connection<'_, Db>, name: Json<Name>, cookies: &CookieJar<'_>) -> Result<Json<Vec<Employee>>, Status> {
    let db = conn.into_inner();
    let new_transaction = name.clone().into_inner();
    
    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session.clone(), Action::FetchEmployee);

    if session.employee.name == new_transaction {
        Ok(Json(vec![session.employee]))
    }else {
        let employee = Employee::fetch_by_name_exact(json!(new_transaction), db).await.unwrap();
        Ok(Json(employee))
    }
}

#[get("/level/<level>")]
pub async fn get_by_level(conn: Connection<'_, Db>, level: i32, cookies: &CookieJar<'_>) -> Result<Json<Vec<Employee>>, Status> {
    let db = conn.into_inner();
    let new_transaction = level.clone();

    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session, Action::FetchEmployee);

    let employee = Employee::fetch_by_level(new_transaction, db).await.unwrap();
    Ok(Json(employee))
}

#[post("/generate")]
async fn generate(
    conn: Connection<'_, Db>,
    cookies: &CookieJar<'_>
) -> Result<Json<Employee>, Status> {
    let db = conn.into_inner();

    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session, Action::GenerateTemplateContent);

    match Employee::generate(db).await {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(Status::BadRequest)
    }
}

#[post("/<id>", data = "<input_data>")]
async fn update(
    conn: Connection<'_, Db>,
    id: &str,
    cookies: &CookieJar<'_>,
    input_data: Json<Employee>,
) -> Result<Json<Employee>, Status> {
    let input_data = input_data.clone().into_inner();
    let db = conn.into_inner();
    
    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session.clone(), Action::ModifyEmployee);

    if session.employee.level.into_iter().find(| x | x.action == Action::ModifyEmployee).unwrap().authority >= 1 {
        match Employee::update(input_data, id, db).await {
            Ok(res) => {
                Ok(Json(res))
            },
            Err(_) => Err(Status::BadRequest),
        }
    }else {
        Err(Status::Unauthorized)
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Auth {
    pub pass: String
}

#[post("/auth/<id>", data = "<input_data>")]
pub async fn auth(id: &str, conn: Connection<'_, Db>, input_data: Json<Auth>, cookies: &CookieJar<'_>) -> Result<Json<String>, Status> {
    let input = input_data.clone().into_inner();
    let db = conn.into_inner();

    match Employee::verify(id, &input.pass, db).await {
        Ok(data) => {
            if !data {
                Err(Status::Unauthorized)
            }else {
                // User is authenticated, lets give them an API key to work with...
                let api_key = Uuid::new_v4().to_string();
                let session_id = Uuid::new_v4().to_string();
                let exp = Utc::now().checked_add_signed(ChronoDuration::minutes(10)).unwrap();

                match session::Entity::insert(session::ActiveModel {
                    id: Set(session_id.to_string()),
                    key: Set(api_key.clone()),
                    employee_id: Set(id.to_string()),
                    expiry: Set(exp.naive_utc()),
                }).exec(db).await {
                    Ok(_) => {
                        let now = OffsetDateTime::now_utc();
                        let expiry = now + Duration::from_secs(10 * 60);

                        let cookie = Cookie::build("key", api_key.clone())
                            .expires(expiry)
                            .path("/")
                            .secure(true)
                            .same_site(SameSite::None)
                            .http_only(true)
                            .finish();

                        cookies.add(cookie);

                        Ok(Json(api_key))
                    },
                    Err(_) => Err(Status::InternalServerError)
                }
            }
        },
        Err(reason) => {
            println!("[dberr]: {}", reason);
            Err(Status::InternalServerError)
        },
    }
}

#[post("/", data = "<input_data>")]
pub async fn create(conn: Connection<'_, Db>, input_data: Json<EmployeeInput>, cookies: &CookieJar<'_>) -> Result<Json<Employee>, Status> {
    let new_transaction = input_data.clone().into_inner();
    let db = conn.into_inner();

    let session = cookie_status_wrapper(db, cookies).await?;
    check_permissions!(session.clone(), Action::CreateEmployee);

    match Employee::insert(new_transaction, db).await {
        Ok(data) => {
            match Employee::fetch_by_id(&data.last_insert_id, db).await {
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

#[derive(Serialize, Deserialize, Clone)]
pub struct LogRequest {
    pub till: String,
    pub reason: String,
    pub in_or_out: String
}

#[post("/log/<id>", data="<input_data>")]
pub async fn log(conn: Connection<'_, Db>, input_data: Json<LogRequest>, id: &str) -> Result<Json<Employee>, Status> {
    let db = conn.into_inner();
    let data = input_data.into_inner();

    let track_type = if data.in_or_out.to_lowercase() == "in" {
        TrackType::In 
    }else  {
        TrackType::Out
    };

    let new_attendance = History::<Attendance> {
        item: Attendance {
            track_type: track_type,
            till: data.till
        },
        reason: "".to_string(),
        timestamp: Utc::now()
    };

    match Employee::fetch_by_id(id, db).await {
        Ok(mut data) => {
            data.clock_history.push(new_attendance);

            match Employee::update(data, id, db).await {
                Ok(data) => {
                    Ok(Json(data))
                }
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