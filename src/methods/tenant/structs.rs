#[cfg(feature = "process")]
use crate::{entities::prelude::Tenants, tenants};
use chrono::{DateTime, Utc};
#[cfg(feature = "process")]
use sea_orm::{DbConn, DbErr, EntityTrait, InsertResult};
use serde::{Deserialize, Serialize};


use crate::Id;

#[cfg(feature = "types")]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct TenantSettings {}

#[cfg(feature = "types")]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Tenant {
    pub tenant_id: Id,

    pub registration_date: DateTime<Utc>,
    pub settings: TenantSettings,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>
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

        Ok(tsn.unwrap().into())
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
        Tenants::insert(tnt.into()).exec(db).await
    }
}

pub fn example_tenant(tenant_id: &str) -> Tenant {
    Tenant {
        tenant_id: tenant_id.to_string(),
        registration_date: Utc::now(),
        settings: TenantSettings {},
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}
