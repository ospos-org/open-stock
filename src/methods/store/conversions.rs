use chrono::{DateTime, Utc};
use sea_orm::ActiveValue::Set;
use serde_json::json;
use crate::entities::store::{ActiveModel, Model};
use crate::{ContactInformation, Session, Store};

impl Store {
    pub(crate) fn into_active(self, session: Session) -> ActiveModel {
        ActiveModel {
            name: Set(self.name),
            id: Set(self.id),
            contact: Set(json!(self.contact)),
            code: Set(self.code),
            tenant_id: Set(session.tenant_id),
            created_at: Default::default(),
            updated_at: Default::default(),
        }
    }
}

impl Into<Store> for Model {
    fn into(self) -> Store {
        Store {
            id: self.id,
            name: self.name,
            contact: serde_json::from_value::<ContactInformation>(self.contact).unwrap(),
            code: serde_json::from_value::<String>(serde_json::Value::String(self.code)).unwrap(),
            updated_at: DateTime::from_utc(self.updated_at, Utc),
            created_at: DateTime::from_utc(self.created_at, Utc)
        }
    }
}