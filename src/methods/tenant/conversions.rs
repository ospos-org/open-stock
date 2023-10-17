use chrono::{DateTime, Utc};
use sea_orm::ActiveValue::Set;
use serde_json::json;
use crate::{Tenant, TenantSettings};
use crate::tenants::{ActiveModel, Model};

impl Into<Tenant> for Model {
    fn into(self) -> Tenant {
        Tenant {
            tenant_id: self.tenant_id,
            registration_date: DateTime::from_utc(self.registration_date, Utc),
            settings: serde_json::from_value::<TenantSettings>(self.settings).unwrap(),

            created_at: DateTime::from_utc(self.created_at, Utc),
            updated_at: DateTime::from_utc(self.updated_at, Utc),
        }
    }
}

impl Into<ActiveModel> for Tenant {
    fn into(self) -> ActiveModel {
        ActiveModel {
            tenant_id: Set(self.tenant_id),
            registration_date: Set(self.registration_date.naive_utc()),
            settings: Set(json!(self.settings)),
            created_at: Set(self.created_at.naive_utc()),
            updated_at: Set(self.updated_at.naive_utc()),
        }
    }
}