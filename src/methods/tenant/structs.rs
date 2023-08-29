use crate::{entities::prelude::Tenants, tenants};
use chrono::{DateTime, Utc};
use sea_orm::Set;
use sea_orm::{DbConn, DbErr, EntityTrait, InsertResult};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::Id;

#[cfg(feature = "types")]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TenantSettings {}

#[cfg(feature = "types")]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Tenant {
    pub tenant_id: Id,

    pub registration_date: DateTime<Utc>,
    pub settings: TenantSettings,
}

#[cfg(feature = "methods")]
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

    pub async fn generate(db: &DbConn, tenant_id: &str) -> Result<Tenant, DbErr> {
        // Create Transaction
        let tsn = example_tenant(tenant_id);

        // Insert & Fetch Transaction
        match Tenant::insert(tsn, db).await {
            Ok(data) => match Tenant::fetch_by_id(&data.last_insert_id, db).await {
                Ok(res) => Ok(res),
                Err(e) => Err(e),
            },
            Err(e) => Err(e),
        }
    }

    pub async fn insert(
        tnt: Tenant,
        db: &DbConn,
    ) -> Result<InsertResult<tenants::ActiveModel>, DbErr> {
        let insert_crud = tenants::ActiveModel {
            tenant_id: Set(tnt.tenant_id),
            registration_date: Set(tnt.registration_date.naive_utc()),
            settings: Set(json!(tnt.settings)),
        };

        match Tenants::insert(insert_crud).exec(db).await {
            Ok(res) => Ok(res),
            Err(err) => Err(err),
        }
    }
}

pub fn example_tenant(tenant_id: &str) -> Tenant {
    Tenant {
        tenant_id: tenant_id.to_string(),
        registration_date: Utc::now(),
        settings: TenantSettings {},
    }
}
