use chrono::{Utc, Duration};
use rocket::{http::Status, get, put};
use rocket::{routes, post, patch};
use rocket::serde::json::Json;
use sea_orm::{EntityTrait, Set};
use sea_orm_rocket::{Connection};
use serde::{Serialize, Deserialize};
use serde_json::json;
use uuid::Uuid;
use crate::entities::session;
use crate::methods::{Name, History};
use crate::pool::Db;

use super::{Employee, EmployeeInput, Attendance, TrackType};

pub fn routes() -> Vec<rocket::Route> {
    routes![get, get_by_name, get_by_name_exact, get_by_level, create, update, log, generate, auth]
}

#[get("/<id>")]
pub async fn get(conn: Connection<'_, Db>, id: &str) -> Result<Json<Employee>, Status> {
    let db = conn.into_inner();

    let employee = Employee::fetch_by_id(&id.to_string(), db).await.unwrap();
    Ok(Json(employee))
}

#[get("/name/<name>")]
pub async fn get_by_name(conn: Connection<'_, Db>, name: &str) -> Result<Json<Vec<Employee>>, Status> {
    let db = conn.into_inner();

    let employee = Employee::fetch_by_name(name, db).await.unwrap();
    Ok(Json(employee))
}

#[get("/!name", data = "<name>")]
pub async fn get_by_name_exact(conn: Connection<'_, Db>, name: Json<Name>) -> Result<Json<Vec<Employee>>, Status> {
    let db = conn.into_inner();
    let new_transaction = name.clone().into_inner();

    println!("{}", json!(new_transaction));

    let employee = Employee::fetch_by_name_exact(json!(new_transaction), db).await.unwrap();
    Ok(Json(employee))
}

#[get("/level/<level>")]
pub async fn get_by_level(conn: Connection<'_, Db>, level: i32) -> Result<Json<Vec<Employee>>, Status> {
    let db = conn.into_inner();
    let new_transaction = level.clone();

    println!("{}", json!(new_transaction));

    let employee = Employee::fetch_by_level(new_transaction, db).await.unwrap();
    Ok(Json(employee))
}

#[patch("/generate")]
async fn generate(
    conn: Connection<'_, Db>
) -> Result<Json<Employee>, Status> {
    let db = conn.into_inner();

    match Employee::generate(db).await {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(Status::BadRequest)
    }
}

#[put("/<id>", data = "<input_data>")]
async fn update(
    conn: Connection<'_, Db>,
    id: &str,
    input_data: Json<Employee>,
) -> Result<Json<Employee>, Status> {
    let input_data = input_data.clone().into_inner();
    let db = conn.into_inner();

    match Employee::update(input_data, id, db).await {
        Ok(res) => {
            Ok(Json(res))
        },
        Err(_) => Err(Status::BadRequest),
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Auth {
    pub pass: String
}

#[post("/auth/<id>", data = "<input_data>")]
pub async fn auth(id: &str, conn: Connection<'_, Db>, input_data: Json<Auth>) -> Result<Json<String>, Status> {
    let input = input_data.clone().into_inner();
    let db = conn.into_inner();

    match Employee::verify(id, &input.pass, db).await {
        Ok(data) => {
            if !data {
                Err(Status::Unauthorized)
            }else {
                // User is authenticated, lets give them an API key to work with...
                let api_key = Uuid::new_v4().to_string();

                let exp = Utc::now().checked_add_signed(Duration::minutes(10)).unwrap();

                match session::Entity::insert(session::ActiveModel {
                    id: Set(id.to_string()),
                    key: Set(api_key.clone()),
                    employee_id: Set(id.to_string()),
                    expiry: Set(exp.naive_utc()),
                }).exec(db).await {
                    Ok(_) => Ok(Json(api_key.clone())),
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
pub async fn create(conn: Connection<'_, Db>, input_data: Json<EmployeeInput>) -> Result<Json<Employee>, Status> {
    let new_transaction = input_data.clone().into_inner();
    let db = conn.into_inner();

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