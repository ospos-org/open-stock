use chrono::{DateTime, Utc};
use sea_orm::ActiveValue::Set;
use serde_json::json;
use uuid::Uuid;
use crate::{Access, AccountType, Action, Attendance, ContactInformation, Employee, EmployeeAuth, EmployeeInput, History, Name};
use crate::entities::employee::{ActiveModel, Model};

impl From<EmployeeInput> for Employee {
    fn from(value: EmployeeInput) -> Self {
        let id = Uuid::new_v4().to_string();

        Employee {
            id,
            rid: value.rid.to_string(),
            name: value.name,
            auth: EmployeeAuth {
                hash: String::new(),
            },
            contact: value.contact,
            clock_history: value.clock_history,
            level: value.level,
            account_type: value.account_type,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

impl EmployeeInput {
    pub(crate) fn into_active(
        self,
        id: String,
        rid: i32,
        tenant_id: String,
        hash: String
    ) -> ActiveModel {
        ActiveModel {
            id: Set(id),
            rid: Set(format!("{:0>#4}", rid)),
            name: Set(json!(self.name)),
            auth: Set(json!(EmployeeAuth { hash })),
            contact: Set(json!(self.contact)),
            clock_history: Set(json!(self.clock_history)),
            level: Set(json!(self.level)),
            tenant_id: Set(tenant_id),
            account_type: Set(json!(self.account_type)),
            created_at: Set(Utc::now().naive_utc()),
            updated_at: Set(Utc::now().naive_utc())
        }
    }
}

impl Into<Employee> for Model {
    fn into(self) -> Employee {
        Employee {
            id: self.id,
            rid: self.rid,
            account_type: serde_json::from_value::<AccountType>(self.account_type).unwrap(),
            name: serde_json::from_value::<Name>(self.name).unwrap(),
            auth: serde_json::from_value::<EmployeeAuth>(self.auth).unwrap(),
            contact: serde_json::from_value::<ContactInformation>(self.contact).unwrap(),
            clock_history: serde_json::from_value::<Vec<History<Attendance>>>(self.clock_history)
                .unwrap(),
            level: serde_json::from_value::<Vec<Access<Action>>>(self.level).unwrap(),
            created_at: DateTime::from_utc(self.created_at, Utc),
            updated_at: DateTime::from_utc(self.updated_at, Utc)
        }
    }
}