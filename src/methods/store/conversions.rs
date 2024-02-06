use crate::entities::store::{ActiveModel, Model};
use crate::{ContactInformation, Session, Store};
use chrono::{DateTime, Utc};
use sea_orm::ActiveValue::Set;
use serde_json::json;

impl Store {
    pub(crate) fn into_active(self, session: Session) -> ActiveModel {
        ActiveModel {
            name: Set(self.name),
            id: Set(self.id),
            contact: Set(json!(self.contact)),
            code: Set(self.code),
            tenant_id: Set(session.tenant_id),
            created_at: Set(self.created_at.naive_utc()),
            updated_at: Set(self.updated_at.naive_utc()),
        }
    }
}

impl From<Model> for Store {
    fn from(val: Model) -> Self {
        Store {
            id: val.id,
            name: val.name,
            contact: serde_json::from_value::<ContactInformation>(val.contact).unwrap(),
            code: serde_json::from_value::<String>(serde_json::Value::String(val.code)).unwrap(),
            updated_at: DateTime::from_naive_utc_and_offset(val.updated_at, Utc),
            created_at: DateTime::from_naive_utc_and_offset(val.created_at, Utc),
        }
    }
}
