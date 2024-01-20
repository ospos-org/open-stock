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

impl From<Model> for Employee {
    fn from(val: Model) -> Self {
        Employee {
            id: val.id,
            rid: val.rid,
            account_type: serde_json::from_value::<AccountType>(val.account_type).unwrap(),
            name: serde_json::from_value::<Name>(val.name).unwrap(),
            auth: serde_json::from_value::<EmployeeAuth>(val.auth).unwrap(),
            contact: serde_json::from_value::<ContactInformation>(val.contact).unwrap(),
            clock_history: serde_json::from_value::<Vec<History<Attendance>>>(val.clock_history)
                .unwrap(),
            level: serde_json::from_value::<Vec<Access<Action>>>(val.level).unwrap(),
            created_at: DateTime::from_naive_utc_and_offset(val.created_at, Utc),
            updated_at: DateTime::from_naive_utc_and_offset(val.updated_at, Utc)
        }
    }
}