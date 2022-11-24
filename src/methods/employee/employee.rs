use std::fmt::{Display, self};

use sea_orm::{DbConn, DbErr, Set, EntityTrait, QuerySelect, ColumnTrait, InsertResult};
use serde::{Serialize, Deserialize};
use serde_json::json;
use uuid::Uuid;

use crate::{methods::{Id, Name, ContactInformation, History}, entities::employee};
use crate::entities::prelude::Employee as Epl;

#[derive(Serialize, Deserialize, Clone)]
pub struct Employee {
    pub id: Id,
    pub name: Name,
    pub auth: EmployeeAuth,
    pub contact: ContactInformation,
    pub clock_history: Vec<History<Attendance>>,
    pub level: i32
}

/// Stores a password hash, signed as a key using the users login ID.
/// Upon logging in using a client portal, the pre-sign object is signed using the provided ID - if the hash matches that which is given, authentication can be approved.
#[derive(Serialize, Deserialize, Clone)]
pub struct EmployeeAuth {
    pub hash: String
}

#[derive(Serialize, Deserialize, Clone)]
pub struct EmployeeInput {
    pub name: Name,
    pub contact: ContactInformation,
    pub auth: EmployeeAuth,
    pub clock_history: Vec<History<Attendance>>,
    pub level: i32
}

impl Display for Employee {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let clock_history: String = self.clock_history.iter()
            .map(|f| 
                format!(
                    "{}: {} ({})\n", 
                    f.timestamp.format("%d/%m/%Y %H:%M"), 
                    f.item.track_type.to_string(), 
                    f.item.till
                )
            ).collect();

        write!(
            f, 
            "{} {} ({})\n{}\n({}) {} {}\n\n[Clock History]\n{}
            ", 
            self.name.first, self.name.last, self.level, 
            self.id, 
            self.contact.mobile.region_code, self.contact.mobile.root, self.contact.email.full,
            clock_history
        )
    }
}

impl Employee {
    pub async fn insert(empl: EmployeeInput, db: &DbConn) -> Result<InsertResult<employee::ActiveModel>, DbErr> {
        let id = Uuid::new_v4().to_string();

        let insert_crud = employee::ActiveModel {
            id: Set(id),
            name: Set(json!(empl.name)),
            auth: Set(json!(empl.auth)),
            contact: Set(json!(empl.contact)),
            clock_history: Set(json!(empl.clock_history)),
            level: Set(empl.level),
        };

        match Epl::insert(insert_crud).exec(db).await {
            Ok(res) => Ok(res),
            Err(err) => Err(err)
        }
    }

    pub async fn fetch_by_id(id: &str, db: &DbConn) -> Result<Employee, DbErr> {
        let empl = Epl::find_by_id(id.to_string()).one(db).await?;
        let e = empl.unwrap();

        Ok(Employee { 
            id: e.id, 
            name: serde_json::from_value::<Name>(e.name).unwrap(),
            auth: serde_json::from_value::<EmployeeAuth>(e.auth).unwrap(),
            contact: serde_json::from_value::<ContactInformation>(e.contact).unwrap(), 
            clock_history: serde_json::from_value::<Vec<History<Attendance>>>(e.clock_history).unwrap(), 
            level: e.level
        })
    }

    pub async fn fetch_by_name(name: &str, db: &DbConn) -> Result<Vec<Employee>, DbErr> {
        let res = employee::Entity::find()
            .having(employee::Column::Name.contains(name))
            .all(db).await?;
            
        let mapped = res.iter().map(|e| 
            Employee { 
                id: e.id.clone(), 
                name: serde_json::from_value::<Name>(e.name.clone()).unwrap(), 
                auth: serde_json::from_value::<EmployeeAuth>(e.auth.clone()).unwrap(),
                contact: serde_json::from_value::<ContactInformation>(e.contact.clone()).unwrap(), 
                clock_history: serde_json::from_value::<Vec<History<Attendance>>>(e.clock_history.clone()).unwrap(), 
                level: e.level
            }
        ).collect();

        Ok(mapped)
    }

    pub async fn fetch_by_name_exact(name: serde_json::Value, db: &DbConn) -> Result<Vec<Employee>, DbErr> {
        let res = employee::Entity::find()
            .having(employee::Column::Name.eq(name))
            .all(db).await?;
            
        let mapped = res.iter().map(|e| 
            Employee { 
                id: e.id.clone(), 
                name: serde_json::from_value::<Name>(e.name.clone()).unwrap(), 
                auth: serde_json::from_value::<EmployeeAuth>(e.auth.clone()).unwrap(),
                contact: serde_json::from_value::<ContactInformation>(e.contact.clone()).unwrap(), 
                clock_history: serde_json::from_value::<Vec<History<Attendance>>>(e.clock_history.clone()).unwrap(), 
                level: e.level
            }
        ).collect();

        Ok(mapped)
    }

    pub async fn fetch_by_level(level: i32, db: &DbConn) -> Result<Vec<Employee>, DbErr> {
        let res = employee::Entity::find()
            .having(employee::Column::Level.eq(level))
            .all(db).await?;
            
        let mapped = res.iter().map(|e| 
            Employee { 
                id: e.id.clone(), 
                name: serde_json::from_value::<Name>(e.name.clone()).unwrap(), 
                auth: serde_json::from_value::<EmployeeAuth>(e.auth.clone()).unwrap(),
                contact: serde_json::from_value::<ContactInformation>(e.contact.clone()).unwrap(), 
                clock_history: serde_json::from_value::<Vec<History<Attendance>>>(e.clock_history.clone()).unwrap(), 
                level: e.level
            }
        ).collect();

        Ok(mapped)
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Attendance {
    pub track_type: TrackType,
    pub till: Id
}

#[derive(Serialize, Deserialize, Clone)]
pub enum TrackType {
    In, Out
}

impl ToString for TrackType {
    fn to_string(&self) -> String {
        match self {
            TrackType::In => "IN".to_string(),
            TrackType::Out => "OUT".to_string(),
        }
    }
}