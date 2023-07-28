use crate::entities::prelude::Kiosk as Ksk;
use crate::{entities::authrecord::ActiveModel as AuthRecord, entities::kiosk::ActiveModel};
use chrono::{DateTime, Utc};
use sea_orm::{ActiveModelTrait, DbConn, DbErr, DeleteResult, EntityTrait, InsertResult, Set};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone)]
pub struct KioskPreferences {
    printer_id: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AuthenticationLog {
    pub employee_id: String,
    pub successful: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Kiosk {
    /// Standard Unique Identification
    id: String,

    /// A user-set custom identifier (Should not be used for unique identification)
    name: String,

    /// The long-form identification of the store to which the kiosk resides.
    store_id: String,

    /// Kiosk Preferences, i.e. Preferred printer [`KioskPreferences`]
    preferences: KioskPreferences,

    /// Lock-down, i.e. Externally disable a Kiosk for any reason
    disabled: bool,

    // The timestamp for the kiosk's last time online.
    last_online: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct KioskInit {
    name: String,
    store_id: String,
    preferences: KioskPreferences,
    disabled: bool,
    last_online: DateTime<Utc>,
}

impl Kiosk {
    pub async fn generate(id: &str, db: &DbConn) -> Result<Kiosk, DbErr> {
        let ksk: KioskInit = example_kiosk();
        // Insert & Fetch Customer
        let result = Kiosk::insert(ksk, db, Some(id)).await.unwrap();
        match Kiosk::fetch_by_id(&result.last_insert_id, db).await {
            Ok(kiosk) => Ok(kiosk),
            Err(e) => Err(e),
        }
    }

    pub async fn fetch_by_id(id: &str, db: &DbConn) -> Result<Kiosk, DbErr> {
        let kiosk = Ksk::find_by_id(id.to_string()).one(db).await?;

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
        };

        match Ksk::insert(insert_crud).exec(db).await {
            Ok(res) => Ok(res),
            Err(err) => Err(err),
        }
    }

    pub async fn update(kiosk: KioskInit, id: &str, db: &DbConn) -> Result<Kiosk, DbErr> {
        ActiveModel {
            id: Set(id.to_string()),
            name: Set(kiosk.name),
            store_id: Set(kiosk.store_id),
            preferences: Set(json!(kiosk.preferences)),
            disabled: Set(kiosk.disabled as i8),
            last_online: Set(kiosk.last_online.naive_utc()),
        }
        .update(db)
        .await?;

        Self::fetch_by_id(id, db).await
    }

    pub async fn delete(id: &str, db: &DbConn) -> Result<DeleteResult, DbErr> {
        match Ksk::delete_by_id(id).exec(db).await {
            Ok(res) => Ok(res),
            Err(err) => Err(err),
        }
    }

    pub async fn auth_log(
        id: &str,
        log: AuthenticationLog,
        db: &DbConn,
    ) -> Result<crate::entities::authrecord::Model, DbErr> {
        let kiosk = Self::fetch_by_id(id, db).await?;

        AuthRecord {
            id: Set(Uuid::new_v4().to_string()),
            kiosk_id: Set(kiosk.id),
            timestamp: Set(Utc::now().naive_utc()),
            attempt: Set(json!(log)),
        }
        .insert(db)
        .await
    }

    pub async fn update_preferences(
        id: &str,
        preferences: KioskPreferences,
        db: &DbConn,
    ) -> Result<Kiosk, DbErr> {
        let kiosk = Self::fetch_by_id(id, db).await?;

        ActiveModel {
            id: Set(kiosk.id),
            name: Set(kiosk.name),
            store_id: Set(kiosk.store_id),
            preferences: Set(json!(preferences)),
            disabled: Set(kiosk.disabled as i8),
            last_online: Set(kiosk.last_online.naive_utc()),
        }
        .update(db)
        .await?;

        Self::fetch_by_id(id, db).await
    }

    pub async fn update_online_to_now(id: &str, db: &DbConn) -> Result<Kiosk, DbErr> {
        let kiosk = Self::fetch_by_id(id, db).await?;

        ActiveModel {
            id: Set(kiosk.id),
            name: Set(kiosk.name),
            store_id: Set(kiosk.store_id),
            preferences: Set(json!(kiosk.preferences)),
            disabled: Set(kiosk.disabled as i8),
            last_online: Set(Utc::now().naive_utc()),
        }
        .update(db)
        .await?;

        Self::fetch_by_id(id, db).await
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