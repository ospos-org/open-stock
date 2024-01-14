use crate::entities::session;
use crate::methods::{cookie_status_wrapper, Error, ErrorResponse, History, Name};
use crate::pool::Db;
use crate::catchers::Validated;
use crate::{check_permissions, example_employee, tenants, AuthenticationLog, Kiosk, Session, create_cookie, LogRequest, Auth};
use chrono::{Days, Duration as ChronoDuration, Utc};
use std::time::Duration;

use okapi::openapi3::OpenApi;
use rocket::get;
use rocket::http::{Cookie, CookieJar, SameSite};
use rocket::serde::json::Json;
use rocket::time::OffsetDateTime;
use rocket::{post};
use rocket_db_pools::Connection;
use rocket_okapi::{openapi, openapi_get_routes_spec};
use rocket_okapi::settings::OpenApiSettings;
use sea_orm::{EntityTrait, Set};
use serde_json::json;
use uuid::Uuid;

use super::{Action, Attendance, Employee, EmployeeInput, TrackType};

pub fn documented_routes(settings: &OpenApiSettings) -> (Vec<rocket::Route>, OpenApi) {
    openapi_get_routes_spec![
        settings:
        get,
        whoami,
        get_by_name,
        get_by_rid,
        auth_rid,
        get_by_name_exact,
        get_by_level,
        create,
        update,
        log,
        generate,
        auth,
        get_status,
    ]
}

#[openapi(tag = "Employee")]
#[get("/")]
pub async fn whoami(
    conn: Connection<Db>,
    cookies: &CookieJar<'_>,
) -> Result<Json<Employee>, Error> {
    let db = conn.into_inner();

    let session = cookie_status_wrapper(&db, cookies).await?;
    check_permissions!(session.clone(), Action::FetchEmployee);

    Ok(Json(session.employee))
}

#[openapi(tag = "Employee")]
#[get("/<id>")]
pub async fn get(
    conn: Connection<Db>,
    id: &str,
    cookies: &CookieJar<'_>,
) -> Result<Json<Employee>, Error> {
    let db = conn.into_inner();

    let session = cookie_status_wrapper(&db, cookies).await?;
    check_permissions!(session.clone(), Action::FetchEmployee);

    if session.employee.id == id {
        Ok(Json(session.employee))
    } else {
        match Employee::fetch_by_id(id, session, &db).await {
            Ok(employee) => Ok(Json(employee)),
            Err(err) => Err(ErrorResponse::db_err(err)),
        }
    }
}

#[openapi(tag = "Employee")]
#[get("/rid/<rid>")]
pub async fn get_by_rid(
    conn: Connection<Db>,
    rid: &str,
    cookies: &CookieJar<'_>,
) -> Result<Json<Vec<Employee>>, Error> {
    let db = conn.into_inner();

    let session = cookie_status_wrapper(&db, cookies).await?;
    check_permissions!(session.clone(), Action::FetchEmployee);

    match Employee::fetch_by_rid(rid, session, &db).await {
        Ok(employee) => Ok(Json(employee)),
        Err(err) => Err(ErrorResponse::db_err(err)),
    }
}

#[openapi(tag = "Employee")]
#[get("/name/<name>")]
pub async fn get_by_name(
    conn: Connection<Db>,
    name: &str,
    cookies: &CookieJar<'_>,
) -> Result<Json<Vec<Employee>>, Error> {
    let db = conn.into_inner();

    let session = cookie_status_wrapper(&db, cookies).await?;
    check_permissions!(session.clone(), Action::FetchEmployee);

    match Employee::fetch_by_name(name, session, &db).await {
        Ok(employees) => Ok(Json(employees)),
        Err(reason) => Err(ErrorResponse::db_err(reason)),
    }
}

#[openapi(tag = "Employee")]
#[get("/!name", data = "<name>")]
pub async fn get_by_name_exact(
    conn: Connection<Db>,
    name: Json<Name>,
    cookies: &CookieJar<'_>,
) -> Result<Json<Vec<Employee>>, Error> {
    let db = conn.into_inner();
    let new_transaction = name.clone().into_inner();

    let session = cookie_status_wrapper(&db, cookies).await?;
    check_permissions!(session.clone(), Action::FetchEmployee);

    if session.employee.name == new_transaction {
        Ok(Json(vec![session.employee]))
    } else {
        match Employee::fetch_by_name_exact(json!(new_transaction), session, &db).await {
            Ok(employees) => Ok(Json(employees)),
            Err(reason) => Err(ErrorResponse::db_err(reason)),
        }
    }
}

#[openapi(tag = "Employee")]
#[get("/level/<level>")]
pub async fn get_by_level(
    conn: Connection<Db>,
    level: i32,
    cookies: &CookieJar<'_>,
) -> Result<Json<Vec<Employee>>, Error> {
    let db = conn.into_inner();

    let session = cookie_status_wrapper(&db, cookies).await?;
    check_permissions!(session.clone(), Action::FetchEmployee);

    match Employee::fetch_by_level(level, session, &db).await {
        Ok(employees) => Ok(Json(employees)),
        Err(reason) => Err(ErrorResponse::db_err(reason)),
    }
}

#[openapi(tag = "Employee")]
#[post("/generate")]
async fn generate(
    conn: Connection<Db>,
    cookies: &CookieJar<'_>,
) -> Result<Json<Employee>, Error> {
    let db = conn.into_inner();

    let session = cookie_status_wrapper(&db, cookies).await?;
    check_permissions!(session.clone(), Action::GenerateTemplateContent);

    match Employee::generate(&db, session).await {
        Ok(res) => Ok(Json(res)),
        Err(err) => Err(ErrorResponse::db_err(err)),
    }
}

#[openapi(tag = "Employee")]
#[post("/<id>", data = "<input_data>")]
async fn update(
    conn: Connection<Db>,
    id: &str,
    cookies: &CookieJar<'_>,
    input_data: Validated<Json<Employee>>,
) -> Result<Json<Employee>, Error> {
    let input_data = input_data.clone().0.into_inner();
    let db = conn.into_inner();

    let session = cookie_status_wrapper(&db, cookies).await?;
    check_permissions!(session.clone(), Action::ModifyEmployee);

    if session
        .clone()
        .employee
        .level
        .into_iter()
        .find(|x| x.action == Action::ModifyEmployee)
        .unwrap()
        .authority
        >= 1
    {
        match Employee::update(input_data, session, id, &db).await {
            Ok(res) => Ok(Json(res)),
            Err(_) => Err(ErrorResponse::input_error()),
        }
    } else {
        Err(ErrorResponse::unauthorized(Action::ModifyEmployee))
    }
}

#[openapi(tag = "Employee")]
#[post("/auth/<id>", data = "<input_data>")]
pub async fn auth(
    id: &str,
    conn: Connection<Db>,
    input_data: Validated<Json<Auth>>,
    cookies: &CookieJar<'_>,
) -> Result<Json<String>, Error> {
    let input = input_data.clone().0.into_inner();
    let db = conn.into_inner();

    let default_employee = example_employee();

    match Employee::verify(
        id,
        Session {
            id: String::new(),
            key: String::new(),
            employee: default_employee.into(),
            expiry: Utc::now().checked_add_days(Days::new(1)).unwrap(),
            tenant_id: input.tenant_id.clone(),
        },
        &input.pass,
        &db,
    )
    .await
    {
        Ok(data) => {
            if !data {
                Err(ErrorResponse::custom_unauthorized(
                    "Invalid password or id.",
                ))
            } else {
                // User is authenticated, lets give them an API key to work with...
                let api_key = Uuid::new_v4().to_string();
                let session_id = Uuid::new_v4().to_string();
                let exp = Utc::now()
                    .checked_add_signed(ChronoDuration::minutes(10))
                    .unwrap();

                let tenant_data = match tenants::Entity::find_by_id(input.tenant_id.clone())
                    .one(&db)
                    .await
                {
                    Ok(optional_data) => match optional_data {
                        Some(data) => data,
                        None => {
                            return Err(ErrorResponse::custom_unauthorized(
                                "Tenant ID does not exist.",
                            ))
                        }
                    },
                    Err(error) => return Err(ErrorResponse::db_err(error)),
                };

                match session::Entity::insert(session::ActiveModel {
                    id: Set(session_id.to_string()),
                    key: Set(api_key.clone()),
                    employee_id: Set(id.to_string()),
                    expiry: Set(exp.naive_utc()),
                    tenant_id: Set(tenant_data.tenant_id),
                })
                .exec(&db)
                .await
                {
                    Ok(_) => {
                        cookies.add(create_cookie(api_key.clone()));
                        Ok(Json(api_key))
                    }
                    Err(reason) => Err(ErrorResponse::db_err(reason)),
                }
            }
        }
        Err(reason) => {
            println!("[dberr]: {}", reason);
            Err(ErrorResponse::db_err(reason))
        }
    }
}

#[openapi(tag = "Employee")]
#[post("/auth/rid/<rid>", data = "<input_data>")]
pub async fn auth_rid(
    rid: &str,
    conn: Connection<Db>,
    input_data: Validated<Json<Auth>>,
    cookies: &CookieJar<'_>,
) -> Result<Json<String>, Error> {
    let input = input_data.clone().0.into_inner();
    let db = conn.into_inner();

    let default_employee = example_employee();
    let session = Session {
        id: String::new(),
        key: String::new(),
        employee: default_employee.into(),
        expiry: Utc::now().checked_add_days(Days::new(1)).unwrap(),
        tenant_id: input.tenant_id.clone(),
    };

    match Employee::verify_with_rid(rid, session.clone(), &input.pass, &db).await {
        Ok(data) => {
            Kiosk::auth_log(
                &input.kiosk_id,
                session.clone(),
                AuthenticationLog {
                    employee_id: data.id.to_string(),
                    successful: true,
                },
                &db,
            )
            .await
            .map_err(ErrorResponse::db_err)?;

            // User is authenticated, lets give them an API key to work with...
            let api_key = Uuid::new_v4().to_string();
            let session_id = Uuid::new_v4().to_string();
            let exp = Utc::now()
                .checked_add_signed(ChronoDuration::minutes(10))
                .unwrap();

            let tenant_data = match tenants::Entity::find_by_id(input.tenant_id.clone())
                .one(&db)
                .await
            {
                Ok(optional_data) => match optional_data {
                    Some(data) => data,
                    None => {
                        return Err(ErrorResponse::custom_unauthorized(
                            "Tenant ID does not exist.",
                        ))
                    }
                },
                Err(error) => return Err(ErrorResponse::db_err(error)),
            };

            match session::Entity::insert(session::ActiveModel {
                id: Set(session_id.to_string()),
                key: Set(api_key.clone()),
                employee_id: Set(data.id.to_string()),
                expiry: Set(exp.naive_utc()),
                tenant_id: Set(tenant_data.tenant_id),
            })
            .exec(&db)
            .await
            {
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
                }
                Err(reason) => Err(ErrorResponse::db_err(reason)),
            }
        }
        Err(reason) => {
            Kiosk::auth_log(
                &input.kiosk_id,
                session.clone(),
                AuthenticationLog {
                    employee_id: rid.to_string(),
                    successful: false,
                },
                &db,
            )
            .await
            .map_err(ErrorResponse::db_err)?;

            Err(ErrorResponse::custom_unauthorized(&format!(
                "Invalid password or id. Reason: {}",
                reason
            )))
        }
    }
}

#[openapi(tag = "Employee")]
#[post("/", data = "<input_data>")]
pub async fn create(
    conn: Connection<Db>,
    input_data: Validated<Json<EmployeeInput>>,
    cookies: &CookieJar<'_>,
) -> Result<Json<Employee>, Error> {
    let new_transaction = input_data.clone().0.into_inner();
    let db = conn.into_inner();

    let session = cookie_status_wrapper(&db, cookies).await?;
    check_permissions!(session.clone(), Action::CreateEmployee);

    match Employee::insert(
        new_transaction, &db, session.clone(), None, None
    ).await {
        Ok(data) =>
            match Employee::fetch_by_id(
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
            Err(ErrorResponse::input_error())
        }
    }
}

#[openapi(tag = "Employee")]
#[post("/log/<id>", data = "<input_data>")]
pub async fn log(
    conn: Connection<Db>,
    input_data: Validated<Json<LogRequest>>,
    id: &str,
    cookies: &CookieJar<'_>,
) -> Result<Json<Employee>, Error> {
    let db = conn.into_inner();
    let data = input_data.0.into_inner();

    let session = cookie_status_wrapper(&db, cookies).await?;
    check_permissions!(session.clone(), Action::FetchEmployee);

    let track_type = if data.in_or_out.to_lowercase() == "in" {
        TrackType::In
    } else {
        TrackType::Out
    };

    let new_attendance = History::<Attendance> {
        item: Attendance {
            track_type,
            kiosk: data.kiosk,
        },
        reason: "".to_string(),
        timestamp: Utc::now(),
    };

    match Employee::fetch_by_id(id, session.clone(), &db).await {
        Ok(mut data) => {
            data.clock_history.push(new_attendance);

            match Employee::update_no_geom(data, session, id, &db).await {
                Ok(data) => Ok(Json(data)),
                Err(reason) => {
                    println!("[dberr]: {}", reason);
                    Err(ErrorResponse::db_err(reason))
                }
            }
        }
        Err(reason) => {
            println!("[dberr]: {}", reason);
            Err(ErrorResponse::input_error())
        }
    }
}

#[openapi(tag = "Employee")]
#[get("/log/<id>")]
pub async fn get_status(
    conn: Connection<Db>,
    id: &str,
    cookies: &CookieJar<'_>,
) -> Result<Json<History<Attendance>>, Error> {
    let db = conn.into_inner();

    let session = cookie_status_wrapper(&db, cookies).await?;
    check_permissions!(session.clone(), Action::FetchEmployee);

    match Employee::fetch_by_id(id, session, &db).await {
        Ok(mut data) => {
            // First time employee is just considered "clocked out"
            if data.clock_history.is_empty() {
                return Ok(Json(History {
                    item: Attendance {
                        track_type: TrackType::Out,
                        kiosk: "new-employee".to_string(),
                    },
                    reason: String::new(),
                    timestamp: Utc::now(),
                }));
            }

            data.clock_history
                .sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

            Ok(Json(data.clock_history[0].clone()))
        }
        Err(reason) => {
            println!("[dberr]: {}", reason);
            Err(ErrorResponse::input_error())
        }
    }
}
