use std::fmt::Display;

use super::{Employee as EmployeeObj, ProductExchange};
#[cfg(feature = "process")]
use crate::entities::employee::Entity as Employee;
#[cfg(feature = "process")]
use crate::entities::session::Entity as SessionEntity;

use crate::{example_employee, session, AccountType, Employee as EmployeeStruct, EmployeeInput};

#[cfg(feature = "process")]
use crate::entities;
use crate::methods::{stml::Order, Access, Action, Attendance, EmployeeAuth};
use chrono::{DateTime, Days, Utc};
use lazy_static::lazy_static;
use okapi::openapi3::Responses;
use regex::Regex;
use rocket::http::{Cookie, SameSite};
use rocket::time::OffsetDateTime;
#[cfg(feature = "process")]
use rocket::{http::CookieJar, serde::json::Json, Responder};
use rocket_okapi::gen::OpenApiGenerator;

use crate::session::{ActiveModel, Model};
use rocket_okapi::response::OpenApiResponderInner;
use schemars::JsonSchema;
use sea_orm::ActiveValue::Set;
#[cfg(feature = "process")]
use sea_orm::{ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QuerySelect};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, JsonSchema, Validate)]
pub struct Name {
    pub first: String,
    pub middle: String,
    pub last: String,
}

impl Name {
    pub(crate) fn from_string(name: String) -> Self {
        let names: Vec<&str> = name.split(' ').collect();

        Name {
            first: names.first().map_or("", |x| x).to_string(),
            middle: names.get(1).map_or("", |x| x).to_string(),
            last: names.get(2).map_or("", |x| x).to_string(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema, Validate)]
pub struct ContactInformation {
    pub name: String,
    pub mobile: MobileNumber,
    pub email: Email,
    pub landline: String,
    pub address: Address,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema, Validate)]
pub struct ContactInformationInput {
    pub name: String,
    pub mobile: String,
    pub email: String,
    pub landline: String,
    pub address: Address,
}

impl ContactInformationInput {
    pub fn into_major(self) -> ContactInformation {
        ContactInformation {
            name: self.name,
            email: Email::from(self.email),
            mobile: MobileNumber::from(self.mobile),
            landline: self.landline,
            address: self.address,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema, Validate)]
pub struct MobileNumber {
    pub number: String,
    pub valid: bool,
}

/// Performs
/// ```regex
/// ^(\+\d{1,2}\s?)?1?\-?\.?\s?\(?\d{3}\)?[\s.-]?\d{3}[\s.-]?\d{4}$
/// ```
pub fn verify_phone_number_with_country_code(ph: &str) -> bool {
    // prevent re-compilation of regex
    lazy_static! {
        static ref RE: Regex =
            Regex::new(r"^(\+\d{1,2}\s?)?1?\-?\.?\s?\(?\d{3}\)?[\s.-]?\d{3}[\s.-]?\d{4}$").unwrap();
    }

    RE.is_match(ph)
}

/// Performs:
/// ```regex
/// ^1?\-?\.?\s?\(?\d{3}\)?[\s.-]?\d{3}[\s.-]?\d{4}$
/// ```
pub fn verify_phone_number_without_country_code(ph: &str) -> bool {
    // prevent re-compilation of regex
    lazy_static! {
        static ref RE: Regex =
            Regex::new(r"^1?\-?\.?\s?\(?\d{3}\)?[\s.-]?\d{3}[\s.-]?\d{4}$").unwrap();
    }

    RE.is_match(ph)
}

impl MobileNumber {
    pub fn from(number: String) -> Self {
        let valid = if number.starts_with('+') {
            verify_phone_number_with_country_code(number.as_str())
        } else {
            verify_phone_number_without_country_code(number.as_str())
        };

        MobileNumber { number, valid }
    }
}

pub type OrderList = Vec<Order>;
pub type NoteList = Vec<Note>;
pub type HistoryList = Vec<History<ProductExchange>>;

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema, Validate)]
pub struct History<T> {
    pub item: T,
    pub reason: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema, Validate)]
pub struct Email {
    pub root: String,
    pub domain: String,
    pub full: String,
}

impl Email {
    pub fn from(email: String) -> Self {
        let split = email.split('@');
        let col = split.collect::<Vec<&str>>();

        let root = match col.first() {
            Some(root) => *root,
            None => "",
        };

        let domain = match col.get(1) {
            Some(domain) => *domain,
            None => "",
        };

        Email {
            root: root.into(),
            domain: domain.into(),
            full: email,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Validate)]
pub struct Note {
    pub message: String,
    pub author: String,
    pub timestamp: DateTime<Utc>,
}

impl Display for Note {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: {}",
            self.timestamp.format("%d/%m/%Y %H:%M"),
            self.message
        )
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema, Validate)]
pub struct Address {
    pub street: String,
    pub street2: String,
    pub city: String,
    pub country: String,
    pub po_code: String,
    pub lat: f64,
    pub lon: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Validate)]
pub struct Location {
    pub store_code: String,
    pub store_id: String,

    // Address is stored in the contact information.
    pub contact: ContactInformation,
}

pub type Url = String;

pub type TagList = Vec<Tag>;
pub type Tag = String;
pub type Id = String;

#[derive(Debug, JsonSchema, Validate)]
pub struct SessionRaw {
    pub id: String,
    pub key: String,
    pub employee_id: String,
    pub expiry: DateTime<Utc>,
    pub variant: SessionVariant,
    pub tenant_id: String,
}

#[derive(Debug, Clone, JsonSchema, Serialize, Deserialize)]
pub enum SessionVariant {
    // Stores ID of AT.
    RefreshToken(String),
    AccessToken,
}

#[derive(Debug, Clone, JsonSchema, Validate, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub key: String,
    pub employee: EmployeeObj,
    pub expiry: DateTime<Utc>,
    pub tenant_id: String,
    pub variant: SessionVariant,
}

impl Session {
    pub fn default_with_tenant(tenant_id: String) -> Self {
        let default_employee = example_employee();

        Session {
            id: String::new(),
            key: String::new(),
            employee: default_employee.into(),
            expiry: Utc::now().checked_add_days(Days::new(1)).unwrap(),
            tenant_id,
            variant: SessionVariant::AccessToken,
        }
    }
}

impl From<Session> for session::ActiveModel {
    fn from(val: Session) -> Self {
        ActiveModel {
            id: Set(val.id),
            key: Set(val.key),
            tenant_id: Set(val.tenant_id),
            employee_id: Set(val.employee.id),
            expiry: Set(val.expiry.naive_utc()),
            variant: Set(json!(val.variant)),
        }
    }
}

impl From<Model> for SessionRaw {
    fn from(value: Model) -> Self {
        SessionRaw {
            id: value.id,
            key: value.key,
            employee_id: value.employee_id,
            expiry: DateTime::from_naive_utc_and_offset(value.expiry, Utc),
            variant: serde_json::from_value::<SessionVariant>(value.variant).unwrap(),
            tenant_id: value.tenant_id,
        }
    }
}

impl Session {
    pub fn has_permission(self, permission: Action) -> bool {
        let action = self
            .employee
            .level
            .into_iter()
            .find(|x| x.action == permission)
            .unwrap_or(Access {
                action: permission,
                authority: 0,
            });

        if action.action == Action::GenerateTemplateContent {
            true
        } else {
            action.authority >= 1
        }
    }

    pub fn ingestion(
        employee: EmployeeInput,
        tenant_id: String,
        employee_id: Option<String>,
    ) -> Self {
        let mut converted_employee: EmployeeStruct = employee.into();
        converted_employee.id = employee_id.map_or(Uuid::new_v4().to_string(), |x| x);

        Self {
            id: Uuid::new_v4().to_string(),
            key: Uuid::new_v4().to_string(),
            employee: converted_employee,
            expiry: Utc::now().checked_add_days(Days::new(1)).unwrap(),
            variant: SessionVariant::AccessToken,
            tenant_id,
        }
    }
}

#[cfg(feature = "process")]
pub fn get_key_cookie(cookies: &CookieJar<'_>) -> Option<String> {
    cookies
        .get("os-stock-key")
        .map(|crumb| crumb.value().to_string())
}

#[cfg(feature = "process")]
pub async fn verify_cookie(key: String, db: &DatabaseConnection) -> Result<Session, DbErr> {
    let session = SessionEntity::find()
        .having(entities::session::Column::Key.eq(key.clone()))
        .find_also_related(Employee)
        .one(db)
        .await?;

    match session {
        Some((val, Some(e))) => Ok(Session {
            id: val.id,
            key: val.key,
            tenant_id: val.tenant_id,
            employee: EmployeeObj {
                id: e.id.clone(),
                rid: e.rid.clone(),
                name: serde_json::from_value::<Name>(e.name.clone()).unwrap(),
                auth: serde_json::from_value::<EmployeeAuth>(e.auth.clone()).unwrap(),
                contact: serde_json::from_value::<ContactInformation>(e.contact.clone()).unwrap(),
                clock_history: serde_json::from_value::<Vec<History<Attendance>>>(
                    e.clock_history.clone(),
                )
                .unwrap(),
                account_type: serde_json::from_value::<AccountType>(e.account_type).unwrap(),
                level: serde_json::from_value::<Vec<Access<Action>>>(e.level).unwrap(),
                created_at: Default::default(),
                updated_at: Default::default(),
            },
            expiry: DateTime::from_naive_utc_and_offset(val.expiry, Utc),
            variant: SessionVariant::AccessToken,
        }),
        None => Err(DbErr::RecordNotFound(format!(
            "Record {} does not exist.",
            key
        ))),
        Some((_, None)) => Err(DbErr::RecordNotFound(format!(
            "Bounded Employee does not exist for key {}",
            key
        ))),
    }
}

#[cfg(feature = "process")]
pub fn create_cookie(api_key: String) -> Cookie<'static> {
    use std::time::Duration;

    let now = OffsetDateTime::now_utc();
    let expiry = now + Duration::from_secs(10 * 60);

    Cookie::build(("os-stock-key", api_key.clone()))
        .expires(expiry)
        .path("/")
        .secure(true)
        .same_site(SameSite::None)
        .http_only(true)
        .build()
}

#[cfg(feature = "process")]
pub async fn _handle_cookie(
    db: &DatabaseConnection,
    cookies: &CookieJar<'_>,
) -> Result<Session, DbErr> {
    match get_key_cookie(cookies) {
        Some(val) => verify_cookie(val, db).await,
        None => Err(DbErr::Custom("Cookies not set.".to_string())),
    }
}

#[cfg(feature = "process")]
pub async fn cookie_status_wrapper(
    db: &DatabaseConnection,
    cookies: &CookieJar<'_>,
) -> Result<Session, Error> {
    match get_key_cookie(cookies) {
        Some(val) => match verify_cookie(val, db).await {
            Ok(v) => Ok(v),
            Err(err) => {
                println!("[err]: {}", err);
                Err(ErrorResponse::custom_unauthorized(
                    "Unable to validate cookie, user does not have valid session.",
                ))
            }
        },
        None => Err(ErrorResponse::custom_unauthorized(
            "Unable to fetch user cookie.",
        )),
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    message: String,
    code: String,
}

#[cfg(feature = "process")]
impl ErrorResponse {
    pub fn create_error(message: &str) -> Error {
        Error::StandardError(Json(ErrorResponse {
            message: message.to_string(),
            code: "error.custom".to_string(),
        }))
    }

    pub fn input_error() -> Error {
        Error::InputError(Json(ErrorResponse {
            message: "Unable to update fields due to malformed inputs".to_string(),
            code: "error.input".to_string(),
        }))
    }

    pub fn unauthorized(action: Action) -> Error {
        Error::Unauthorized(Json(ErrorResponse {
            message: format!("User lacks {:?} permission.", action),
            code: "error.unauthorized".to_string(),
        }))
    }

    pub fn custom_unauthorized(message: &str) -> Error {
        Error::Unauthorized(Json(ErrorResponse {
            message: message.to_string(),
            code: "error.unauthorized.custom".to_string(),
        }))
    }

    pub fn db_err(message: DbErr) -> Error {
        Error::DbError(Json(ErrorResponse {
            message: format!("SQL error, reason: {}", message),
            code: "error.database.query".to_string(),
        }))
    }
}

impl From<DbErr> for Error {
    fn from(value: DbErr) -> Self {
        ErrorResponse::db_err(value)
    }
}

impl<T: Into<Error>> From<Option<T>> for Error {
    fn from(value: Option<T>) -> Self
    where
        T: Into<Error>,
    {
        match value {
            Some(err) => err.into(),
            None => ErrorResponse::create_error("Unable to retrieve database instance."),
        }
    }
}

struct Wrapper<T>(T);
impl<T: JsonSchema> Into<Json<T>> for Wrapper<T> {
    fn into(self) -> Json<T> {
        Json(self.0)
    }
}

#[cfg(feature = "process")]
#[derive(Debug, Responder)]
pub enum Error {
    #[response(status = 500, content_type = "json")]
    StandardError(Json<ErrorResponse>),
    #[response(status = 400, content_type = "json")]
    InputError(Json<ErrorResponse>),
    #[response(status = 401, content_type = "json")]
    Unauthorized(Json<ErrorResponse>),
    #[response(status = 500, content_type = "json")]
    DbError(Json<ErrorResponse>),
    #[response(status = 500, content_type = "text")]
    DemoDisabled(String),
}

impl OpenApiResponderInner for Error {
    fn responses(_gen: &mut OpenApiGenerator) -> rocket_okapi::Result<Responses> {
        Ok(Responses::default())
    }
}

pub struct VoidableResult<T>(pub Result<T, Error>);

impl<T> VoidableResult<T> {
    pub fn void(self) -> Result<(), Error> {
        self.into()
    }
}

impl<T> From<Result<T, Error>> for VoidableResult<T> {
    fn from(value: Result<T, Error>) -> Self {
        VoidableResult(value)
    }
}

impl<T> Into<Result<(), Error>> for VoidableResult<T> {
    fn into(self) -> Result<(), Error> {
        self.0.map(|_| ())
    }
}
