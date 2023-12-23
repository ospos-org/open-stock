#[cfg(feature = "process")]
use crate::entities::prelude::Kiosk as Ksk;
#[cfg(feature = "process")]
use crate::{entities::authrecord::ActiveModel as AuthRecord, entities::kiosk::ActiveModel};
#[cfg(feature = "process")]
use crate::{entities::kiosk, Session};
use chrono::{DateTime, Utc};
use schemars::JsonSchema;
#[cfg(feature = "process")]
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbConn, DbErr, DeleteResult, EntityTrait, InsertResult,
    QueryFilter, Set,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;
use validator::Validate;
use crate::entities::kiosk::Model;

#[cfg(feature = "types")]
#[derive(Serialize, Deserialize, Clone, JsonSchema, Validate)]
pub struct KioskPreferences {
    pub printer_id: String,
}

#[cfg(feature = "types")]
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema, Validate)]
pub struct AuthenticationLog {
    pub employee_id: String,
    pub successful: bool,
}

#[cfg(feature = "types")]
#[derive(Serialize, Deserialize, Clone, JsonSchema, Validate)]
pub struct Kiosk {
    /// Standard Unique Identification
    pub id: String,

    /// A user-set custom identifier (Should not be used for unique identification)
    pub name: String,

    /// The long-form identification of the store to which the kiosk resides.
    pub store_id: String,

    /// Kiosk Preferences, i.e. Preferred printer [`KioskPreferences`]
    pub preferences: KioskPreferences,

    /// Lock-down, i.e. Externally disable a Kiosk for any reason
    pub disabled: bool,

    // The timestamp for the kiosk's last time online.
    pub last_online: DateTime<Utc>,
}

#[cfg(feature = "types")]
#[derive(Serialize, Deserialize, Clone, JsonSchema, Validate)]
pub struct KioskInit {
    name: String,
    store_id: String,
    preferences: KioskPreferences,
    disabled: bool,
    last_online: DateTime<Utc>,
}

#[cfg(feature = "methods")]
impl Kiosk {
    pub async fn generate(id: &str, session: Session, db: &DbConn) -> Result<Kiosk, DbErr> {
        let ksk: KioskInit = example_kiosk();
        // Insert & Fetch Customer
        let result = Kiosk::insert(ksk, session.clone(), db, Some(id))
            .await
            .unwrap();
        match Kiosk::fetch_by_id(&result.last_insert_id, session, db).await {
            Ok(kiosk) => Ok(kiosk),
            Err(e) => Err(e),
        }
    }

    pub async fn fetch_by_id(id: &str, session: Session, db: &DbConn) -> Result<Kiosk, DbErr> {
        let kiosk = Ksk::find_by_id(id.to_string())
            .filter(kiosk::Column::TenantId.eq(session.tenant_id))
            .one(db)
            .await?;

        match kiosk {
            Some(k) => Ok(Kiosk {
                id: k.id,
                name: k.name,
                store_id: k.store_id,
                preferences: serde_json::from_value::<KioskPreferences>(k.preferences).unwrap(),
                disabled: k.disabled != 0,
                last_online: DateTime::from_utc(k.last_online, Utc),
            }),
            None => Err(DbErr::RecordNotFound(id.to_string())),
        }
    }

    pub async fn insert(
        kiosk: KioskInit,
        session: Session,
        db: &DbConn,
        id: Option<&str>,
    ) -> Result<InsertResult<ActiveModel>, DbErr> {
        let id = match id {
            Some(id) => id.to_string(),
            None => Uuid::new_v4().to_string(),
        };

        let insert_crud = ActiveModel {
            id: Set(id),
            name: Set(kiosk.name),
            store_id: Set(kiosk.store_id),
            preferences: Set(json!(kiosk.preferences)),
            disabled: Set(kiosk.disabled as i8),
            last_online: Set(kiosk.last_online.naive_utc()),
            tenant_id: Set(session.tenant_id),
        };

        match Ksk::insert(insert_crud).exec(db).await {
            Ok(res) => Ok(res),
            Err(err) => Err(err),
        }
    }

    pub async fn insert_raw(
        kiosk: Kiosk,
        session: Session,
        db: &DbConn,
    ) -> Result<Model, DbErr> {
        match kiosk.into_active(session).insert(db).await {
            Ok(res) => Ok(res),
            Err(err) => Err(err),
        }
    }

    pub async fn update(
        kiosk: KioskInit,
        session: Session,
        id: &str,
        db: &DbConn,
    ) -> Result<Kiosk, DbErr> {
        ActiveModel {
            id: Set(id.to_string()),
            name: Set(kiosk.name),
            store_id: Set(kiosk.store_id),
            preferences: Set(json!(kiosk.preferences)),
            disabled: Set(kiosk.disabled as i8),
            last_online: Set(kiosk.last_online.naive_utc()),
            tenant_id: Set(session.clone().tenant_id),
        }
        .update(db)
        .await?;

        Self::fetch_by_id(id, session, db).await
    }

    pub async fn delete(id: &str, session: Session, db: &DbConn) -> Result<DeleteResult, DbErr> {
        match Ksk::delete_by_id(id)
            .filter(kiosk::Column::TenantId.eq(session.tenant_id))
            .exec(db)
            .await
        {
            Ok(res) => Ok(res),
            Err(err) => Err(err),
        }
    }

    pub async fn auth_log(
        id: &str,
        session: Session,
        log: AuthenticationLog,
        db: &DbConn,
    ) -> Result<crate::entities::authrecord::Model, DbErr> {
        let kiosk = Self::fetch_by_id(id, session.clone(), db).await?;

        AuthRecord {
            id: Set(Uuid::new_v4().to_string()),
            kiosk_id: Set(kiosk.id),
            timestamp: Set(Utc::now().naive_utc()),
            attempt: Set(json!(log)),
            tenant_id: Set(session.tenant_id),
        }
        .insert(db)
        .await
    }

    pub async fn update_preferences(
        id: &str,
        session: Session,
        preferences: KioskPreferences,
        db: &DbConn,
    ) -> Result<Kiosk, DbErr> {
        let kiosk = Self::fetch_by_id(id, session.clone(), db).await?;

        ActiveModel {
            id: Set(kiosk.id),
            name: Set(kiosk.name),
            store_id: Set(kiosk.store_id),
            preferences: Set(json!(preferences)),
            disabled: Set(kiosk.disabled as i8),
            last_online: Set(kiosk.last_online.naive_utc()),
            tenant_id: Set(session.clone().tenant_id),
        }
        .update(db)
        .await?;

        Self::fetch_by_id(id, session, db).await
    }

    pub async fn update_online_to_now(
        id: &str,
        session: Session,
        db: &DbConn,
    ) -> Result<Kiosk, DbErr> {
        let kiosk = Self::fetch_by_id(id, session.clone(), db).await?;

        ActiveModel {
            id: Set(kiosk.id),
            name: Set(kiosk.name),
            store_id: Set(kiosk.store_id),
            preferences: Set(json!(kiosk.preferences)),
            disabled: Set(kiosk.disabled as i8),
            last_online: Set(Utc::now().naive_utc()),
            tenant_id: Set(session.clone().tenant_id),
        }
        .update(db)
        .await?;

        Self::fetch_by_id(id, session, db).await
    }
}

pub fn example_kiosk() -> KioskInit {
    KioskInit {
        name: "Front Counter".to_string(),
        store_id: "c4a1d88b-e8a0-4dcd-ade2-1eea82254816".to_string(),
        preferences: KioskPreferences {
            printer_id: "none".to_string(),
        },
        disabled: false,
        last_online: Utc::now(),
    }
}
