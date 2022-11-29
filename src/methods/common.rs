use std::fmt::Display;

use crate::{methods::stml::Order, entities};
use chrono::{Utc, DateTime};
use rocket::{http::CookieJar};
use sea_orm::{DatabaseConnection, DbErr, EntityTrait, QuerySelect, ColumnTrait};
use serde::{Serialize, Deserialize};
use crate::entities::session::Entity as SessionEntity;
use crate::entities::employee::Entity as Employee;

use super::{ProductExchange};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Name {
    pub first: String,
    pub middle: String,
    pub last: String
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContactInformation {
    pub name: String,
    pub mobile: MobileNumber,
    pub email: Email,
    pub landline: String,
    pub address: Address
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MobileNumber {
    pub region_code: String,
    pub root: String
}

impl MobileNumber {
    pub fn from(number: String) -> Self {
        MobileNumber { region_code: "+64".into(), root: number }
    }
}

pub type OrderList = Vec<Order>;
pub type NoteList = Vec<Note>;
pub type HistoryList = Vec<History<ProductExchange>>;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct History<T> {
    pub item: T,
    pub reason: String,
    pub timestamp: DateTime<Utc>
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Email {
    pub root: String,
    pub domain: String,
    pub full: String
}

impl Email {
    pub fn from(email: String) -> Self {
        let split = email.split("@");
        let col = split.collect::<Vec<&str>>();

        let root = match col.get(0) {
            Some(root) => {
                *root
            },
            None => "",
        };

        let domain = match col.get(1) {
            Some(domain) => {
                *domain
            },
            None => "",
        };

        Email {
            root: root.into(),
            domain: domain.into(),
            full: email
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub message: String,
    pub timestamp: DateTime<Utc>
}

impl Display for Note {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f, 
            "{}: {}",
            self.timestamp.format("%d/%m/%Y %H:%M"), self.message
        )
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Address {
    pub street: String,
    pub street2: String,
    pub city: String,
    pub country: String,
    pub po_code: String
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub code: String,
    // Address is stored in the contact information.
    pub contact: ContactInformation
}

pub type Url = String;

pub type TagList = Vec<Tag>;
pub type Tag = String;
pub type Id = String;

#[derive(Debug)]
pub struct Session {
    pub id: String,
    pub key: String,
    pub employee_id: String,
    pub expiry: DateTime<Utc>,
}

pub fn get_key_cookie(cookies: &CookieJar<'_>) -> Option<String> {
    match cookies.get("key")
        .map(|crumb| format!("{}", crumb.value())) {
            Some(val) => {
                println!("{}", val);
                Some(val)
            }
            None => {
                println!("Uh oh, the cookie jar is empty.");
                None
            }
        }
}

pub async fn verify_cookie(key: String, db: &DatabaseConnection) -> Result<Session, DbErr> {
    let session = SessionEntity::find()
        .having(entities::session::Column::Key.eq(key.clone()))
        .find_also_related(Employee)
        .one(db).await?;
    
    match session {
        Some((val, empl)) => {
            println!("{:?} and {:?}", val, empl.unwrap());
            
            Ok(Session {
                id: val.id,
                key: val.key,
                employee_id: val.employee_id,
                expiry: DateTime::from_utc(val.expiry, Utc)
            })
        },
        None => Err(DbErr::RecordNotFound(format!("Record {} does not exist.", key))),
    }
}

pub async fn handle_cookie(db: &DatabaseConnection, cookies: &CookieJar<'_>) {
    match get_key_cookie(cookies) {
        Some(val) => {
            match verify_cookie(val, db).await {
                Ok(ses) => {
                    println!("{:?}", ses);
                },
                Err(err) => println!("[err]: {}", err),
            }
        },
        None => {},
    }
}