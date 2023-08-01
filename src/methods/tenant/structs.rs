use crate::entities::prelude::Tenants;
use chrono::{DateTime, Utc};
use sea_orm::{DbConn, DbErr, EntityTrait};
use serde::{Deserialize, Serialize};

use crate::{tenants, Id};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TenantSettings {}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Tenant {
    pub tenant_id: Id,

    pub registration_date: DateTime<Utc>,
    pub settings: TenantSettings,
}

impl Tenant {
    pub async fn fetch_by_id(id: &str, db: &DbConn) -> Result<Tenant, DbErr> {
        let tsn = Tenants::find_by_id(id.to_string()).one(db).await?;

        if tsn.is_none() {
            return Err(DbErr::Custom(
                "Unable to query value, returns none".to_string(),
            ));
        }

        let t = tsn.unwrap();

        let t = Tenant {
            tenant_id: t.tenant_id,
            registration_date: DateTime::from_utc(t.registration_date, Utc),
            settings: serde_json::from_value::<TenantSettings>(t.settings).unwrap(),
        };

        Ok(t)
    }
}
