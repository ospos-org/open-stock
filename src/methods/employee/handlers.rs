use crate::catchers::Validated;
use crate::entities::session;
use crate::guards::Convert;
use crate::methods::{cookie_status_wrapper, Error, ErrorResponse, History, Name};
use crate::pool::{Db, InternalDb};
use crate::SessionVariant;
use crate::{
    check_permissions, create_cookie, example_employee, tenants, Auth, AuthenticationLog, Customer,
    Kiosk, LogRequest, Session,
};
use chrono::{Days, Duration as ChronoDuration, Utc};
use okapi::openapi3::OpenApi;
use rocket::get;
use rocket::http::CookieJar;
use rocket::post;
use rocket::serde::json::Json;
use rocket_db_pools::Connection;
use rocket_okapi::settings::OpenApiSettings;
use rocket_okapi::{openapi, openapi_get_routes_spec};
use sea_orm::{EntityTrait, Set};
use serde_json::json;
use uuid::Uuid;

use super::{Action, Attendance, Employee, EmployeeInput, TrackType};

pub fn documented_routes(settings: &OpenApiSettings) -> (Vec<rocket::Route>, OpenApi) {
    openapi_get_routes_spec![
        settings:
        get,
        get_recent,
        update_by_input,
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
pub async fn whoami(session: Session) -> Convert<Employee> {
    check_permissions!(session.clone(), Action::FetchEmployee);
    Ok(session.employee).into()
}

#[openapi(tag = "Employee")]
#[get("/<id>")]
pub async fn get(db: InternalDb, id: &str, session: Session) -> Convert<Employee> {
    check_permissions!(session.clone(), Action::FetchEmployee);

    if session.employee.id == id {
        Ok(session.employee).into()
    } else {
        Employee::fetch_by_id(id, session, &db.0).await.into()
    }
}

#[openapi(tag = "Employee")]
#[get("/rid/<rid>")]
pub async fn get_by_rid(db: InternalDb, rid: &str, session: Session) -> Convert<Vec<Employee>> {
    check_permissions!(session.clone(), Action::FetchEmployee);
    Employee::fetch_by_rid(rid, session, &db.0).await.into()
}

#[openapi(tag = "Employee")]
#[get("/recent")]
pub async fn get_recent(db: InternalDb, session: Session) -> Convert<Vec<Employee>> {
    check_permissions!(session.clone(), Action::FetchCustomer);
    Employee::fetch_recent(session, &db.0).await.into()
}

#[openapi(tag = "Employee")]
#[get("/name/<name>")]
pub async fn get_by_name(db: InternalDb, session: Session, name: &str) -> Convert<Vec<Employee>> {
    check_permissions!(session.clone(), Action::FetchEmployee);
    Employee::fetch_by_name(name, session, &db.0).await.into()
}

#[openapi(tag = "Employee")]
#[get("/!name", data = "<name>")]
pub async fn get_by_name_exact(
    db: InternalDb,
    session: Session,
    name: Json<Name>,
) -> Convert<Vec<Employee>> {
    check_permissions!(session.clone(), Action::FetchEmployee);

    if session.employee.name == name.0 {
        Ok(vec![session.employee]).into()
    } else {
        Employee::fetch_by_name_exact(json!(name.0), session, &db.0)
            .await
            .into()
    }
}

#[openapi(tag = "Employee")]
#[get("/level/<level>")]
pub async fn get_by_level(db: InternalDb, session: Session, level: i32) -> Convert<Vec<Employee>> {
    check_permissions!(session.clone(), Action::FetchEmployee);
    Employee::fetch_by_level(level, session, &db.0).await.into()
}

#[openapi(tag = "Employee")]
#[post("/generate")]
async fn generate(db: InternalDb, session: Session) -> Convert<Employee> {
    check_permissions!(session.clone(), Action::GenerateTemplateContent);
    Employee::generate(&db.0, session).await.into()
}

#[openapi(tag = "Employee")]
#[post("/<id>", data = "<input_data>")]
async fn update(
    db: InternalDb,
    session: Session,
    id: &str,
    input_data: Validated<Json<Employee>>,
) -> Convert<Employee> {
    check_permissions!(session.clone(), Action::ModifyEmployee);
    Employee::update(input_data.data(), session, id, &db.0)
        .await
        .into()
}

#[openapi(tag = "Employee")]
#[post("/input/<id>", data = "<input_data>")]
async fn update_by_input(
    db: InternalDb,
    session: Session,
    id: &str,
    input_data: Validated<Json<EmployeeInput>>,
) -> Convert<Employee> {
    check_permissions!(session.clone(), Action::ModifyEmployee);
    Employee::update_by_input(input_data.data(), session, id, &db.0)
        .await
        .into()
}

#[openapi(tag = "Employee")]
#[post("/auth/<id>", data = "<input_data>")]
pub async fn auth(
    db: InternalDb,
    input_data: Validated<Json<Auth>>,
    cookies: &CookieJar<'_>,
    id: &str,
) -> Result<Json<String>, Error> {
    let input = input_data.data();
    let default_session = Session::default_with_tenant(input.tenant_id.clone());

    let verified = Employee::verify(id, default_session, &input.pass, &db.0).await?;

    match verified {
        false => Err(ErrorResponse::custom_unauthorized(
            "Invalid password or id.",
        )),
        true => {
            // User is authenticated, lets give them an API key to work with...
            let api_key = Uuid::new_v4().to_string();
            let session_id = Uuid::new_v4().to_string();
            let exp = Utc::now()
                .checked_add_signed(ChronoDuration::minutes(10))
                .unwrap();

            let tenant_data: Option<tenants::Model> =
                tenants::Entity::find_by_id(input.tenant_id.clone())
                    .one(&db.0)
                    .await?;

            match tenant_data {
                Some(data) => {
                    session::Entity::insert(session::ActiveModel {
                        id: Set(session_id.to_string()),
                        key: Set(api_key.clone()),
                        employee_id: Set(id.to_string()),
                        expiry: Set(exp.naive_utc()),
                        tenant_id: Set(data.tenant_id),
                        variant: Set(json!(SessionVariant::AccessToken)),
                    })
                    .exec(&db.0)
                    .await?;

                    cookies.add(create_cookie(api_key.clone()));
                    Ok(Json(api_key))
                }
                None => Err(ErrorResponse::create_error("Tenant does not exist.")),
            }
        }
    }
}

#[openapi(tag = "Employee")]
#[post("/auth/rid/<rid>", data = "<input_data>")]
pub async fn auth_rid(
    db: InternalDb,
    input_data: Validated<Json<Auth>>,
    cookies: &CookieJar<'_>,
    rid: &str,
) -> Result<Json<String>, Error> {
    let input = input_data.data();
    let session = Session::default_with_tenant(input.tenant_id.clone());

    match Employee::verify_with_rid(rid, session.clone(), &input.pass, &db.0).await {
        Ok(data) => {
            let auth_log = AuthenticationLog {
                employee_id: data.id.to_string(),
                successful: true,
            };
            Kiosk::auth_log(&input.kiosk_id, session.clone(), auth_log, &db.0).await?;

            let api_key = Uuid::new_v4().to_string();
            let session_id = Uuid::new_v4().to_string();
            let exp = Utc::now()
                .checked_add_signed(ChronoDuration::minutes(10))
                .unwrap();

            let tenant_data: Option<tenants::Model> =
                tenants::Entity::find_by_id(input.tenant_id.clone())
                    .one(&db.0)
                    .await?;

            match tenant_data {
                Some(tenant) => {
                    session::Entity::insert(session::ActiveModel {
                        id: Set(session_id.to_string()),
                        key: Set(api_key.clone()),
                        employee_id: Set(data.id.to_string()),
                        expiry: Set(exp.naive_utc()),
                        tenant_id: Set(tenant.tenant_id),
                        variant: Set(json!(SessionVariant::AccessToken)),
                    })
                    .exec(&db.0)
                    .await?;

                    cookies.add(create_cookie(api_key.clone()));
                    Ok(Json(api_key))
                }
                None => Err(ErrorResponse::create_error("Tenant does not exist.")),
            }
        }
        Err(err) => {
            let auth_log = AuthenticationLog {
                employee_id: rid.to_string(),
                successful: false,
            };
            Kiosk::auth_log(&input.kiosk_id, session.clone(), auth_log, &db.0).await?;

            Err(ErrorResponse::custom_unauthorized(&format!(
                "Invalid password or id. Reason: {:?}",
                err
            )))
        }
    }
}

#[openapi(tag = "Employee")]
#[post("/", data = "<input_data>")]
pub async fn create(
    db: InternalDb,
    session: Session,
    input_data: Validated<Json<EmployeeInput>>,
) -> Result<Json<Employee>, Error> {
    check_permissions!(session.clone(), Action::CreateEmployee);

    let data = Employee::insert(input_data.data(), &db.0, session.clone(), None, None).await?;
    let converted: Convert<Employee> = Employee::fetch_by_id(&data.last_insert_id, session, &db.0)
        .await
        .into();
    converted.0
}

#[openapi(tag = "Employee")]
#[post("/log/<id>", data = "<input_data>")]
pub async fn log(
    db: InternalDb,
    session: Session,
    input_data: Validated<Json<LogRequest>>,
    id: &str,
) -> Result<Json<Employee>, Error> {
    check_permissions!(session.clone(), Action::FetchEmployee);

    let data = input_data.data();

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
        reason: "OpenStock - Log".to_string(),
        timestamp: Utc::now(),
    };

    let mut data = Employee::fetch_by_id(id, session.clone(), &db.0).await?;
    data.clock_history.push(new_attendance);

    let converted: Convert<Employee> = Employee::update_no_geom(data, session, id, &db.0)
        .await
        .into();
    converted.0
}

#[openapi(tag = "Employee")]
#[get("/log/<id>")]
pub async fn get_status(
    db: InternalDb,
    session: Session,
    id: &str,
) -> Result<Json<History<Attendance>>, Error> {
    check_permissions!(session.clone(), Action::FetchEmployee);

    let mut data = Employee::fetch_by_id(id, session, &db.0).await?;

    // First time employee is just considered "clocked out"
    if data.clock_history.is_empty() {
        return Ok(Json(History {
            item: Attendance {
                track_type: TrackType::Out,
                kiosk: "new-employee".to_string(),
            },
            reason: "This employee has never clocked in.".to_string(),
            timestamp: Utc::now(),
        }));
    }

    data.clock_history
        .sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    Ok(Json(data.clock_history[0].clone()))
}
