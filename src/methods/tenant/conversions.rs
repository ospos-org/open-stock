use crate::tenants::{ActiveModel, Model};
use crate::{Tenant, TenantSettings};
use chrono::{DateTime, Utc};
use sea_orm::ActiveValue::Set;
use serde_json::json;

impl From<Model> for Tenant {
    fn from(val: Model) -> Self {
        Tenant {
            tenant_id: val.tenant_id,
            registration_date: DateTime::from_naive_utc_and_offset(val.registration_date, Utc),
            settings: serde_json::from_value::<TenantSettings>(val.settings).unwrap(),

            created_at: DateTime::from_naive_utc_and_offset(val.created_at, Utc),
            updated_at: DateTime::from_naive_utc_and_offset(val.updated_at, Utc),
        }
    }
}

impl From<Tenant> for ActiveModel {
    fn from(val: Tenant) -> Self {
        ActiveModel {
            tenant_id: Set(val.tenant_id),
            registration_date: Set(val.registration_date.naive_utc()),
            settings: Set(json!(val.settings)),
            created_at: Set(val.created_at.naive_utc()),
            updated_at: Set(val.updated_at.naive_utc()),
        }
    }
}
