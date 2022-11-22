use sea_orm::{DbConn, DbErr, Set, EntityTrait};
use serde::{Serialize, Deserialize};
use serde_json::json;

use crate::{methods::{Id, Name, ContactInformation, History}, entities::employee};
use crate::entities::prelude::Employee as Epl;

pub struct Employee {
    pub id: Id,
    pub name: Name,
    pub contact: ContactInformation,
    pub clock_history: Vec<History<Attendance>>,
    pub level: i32
}

impl Employee {
    pub async fn insert(empl: Employee, db: &DbConn) -> Result<(), DbErr> {
        let insert_crud = employee::ActiveModel {
            id: Set(empl.id),
            name: Set(json!(empl.name)),
            contact: Set(json!(empl.contact)),
            clock_history: Set(json!(empl.clock_history)),
            level: Set(empl.level),
        };

        match Epl::insert(insert_crud).exec(db).await {
            Ok(_) => Ok(()),
            Err(err) => Err(err)
        }
    }

    pub async fn fetch_by_id(id: &str, db: &DbConn) -> Result<Employee, DbErr> {
        let empl = Epl::find_by_id(id.to_string()).one(db).await?;
        let e = empl.unwrap();

        Ok(Employee { 
            id: e.id, 
            name: serde_json::from_value::<Name>(e.name).unwrap(), 
            contact: serde_json::from_value::<ContactInformation>(e.contact).unwrap(), 
            clock_history: serde_json::from_value::<Vec<History<Attendance>>>(e.clock_history).unwrap(), 
            level: e.level
        })
    }
}

#[derive(Serialize, Deserialize)]
pub struct Attendance {
    pub track_type: TrackType,
    pub till: Id
}

#[derive(Serialize, Deserialize)]
pub enum TrackType {
    In, Out
}