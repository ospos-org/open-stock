use std::fmt::{Display, self};

use sea_orm::{DbConn, DbErr, Set, EntityTrait};
use serde::{Serialize, Deserialize};
use serde_json::json;

use crate::{methods::{Id, Name, ContactInformation, History}, entities::employee};
use crate::entities::prelude::Employee as Epl;

#[derive(Serialize)]
pub struct Employee {
    pub id: Id,
    pub name: Name,
    pub contact: ContactInformation,
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

impl ToString for TrackType {
    fn to_string(&self) -> String {
        match self {
            TrackType::In => "IN".to_string(),
            TrackType::Out => "OUT".to_string(),
        }
    }
}