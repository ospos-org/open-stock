use crate::entities::prelude::Kiosk as Ksk;
use crate::History;
use crate::{entities::kiosk::ActiveModel, entities::kiosk::Model};
use chrono::{DateTime, Utc};
use sea_orm::{DbConn, DbErr, DeleteResult, EntityTrait, InsertResult, Set};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone)]
pub struct KioskPreferences {
    printer_id: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AuthenticationLog {
    employee_id: String,
    successful: bool,
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

    // A list of all login attempts to the kiosk
    login_history: Vec<History<AuthenticationLog>>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct KioskInit {
    name: String,
    store_id: String,
    preferences: KioskPreferences,
    disabled: bool,
    last_online: DateTime<Utc>,
    login_history: Vec<History<AuthenticationLog>>,
}

impl Kiosk {
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
                login_history: serde_json::from_value::<Vec<History<AuthenticationLog>>>(
                    k.login_history,
                )
                .unwrap(),
            }),
            None => Err(DbErr::RecordNotFound(id.to_string())),
        }
    }

    pub async fn insert(kiosk: KioskInit, db: &DbConn) -> Result<InsertResult<ActiveModel>, DbErr> {
        let id = Uuid::new_v4().to_string();

        let insert_crud = ActiveModel {
            id: Set(id),
            name: Set(kiosk.name),
            store_id: Set(kiosk.store_id),
            preferences: Set(json!(kiosk.preferences)),
            disabled: Set(kiosk.disabled as i8),
            last_online: Set(kiosk.last_online.naive_utc()),
            login_history: Set(json!(kiosk.login_history)),
        };

        match Ksk::insert(insert_crud).exec(db).await {
            Ok(res) => Ok(res),
            Err(err) => Err(err),
        }
    }

    pub async fn update(kiosk: Kiosk, db: &DbConn) -> Result<Model, DbErr> {
        let insert_crud = ActiveModel {
            id: Set(kiosk.id),
            name: Set(kiosk.name),
            store_id: Set(kiosk.store_id),
            preferences: Set(json!(kiosk.preferences)),
            disabled: Set(kiosk.disabled as i8),
            last_online: Set(kiosk.last_online.naive_utc()),
            login_history: Set(json!(kiosk.login_history)),
        };

        match Ksk::update(insert_crud).exec(db).await {
            Ok(res) => Ok(res),
            Err(err) => Err(err),
        }
    }

    pub async fn delete(id: &str, db: &DbConn) -> Result<DeleteResult, DbErr> {
        match Ksk::delete_by_id(id).exec(db).await {
            Ok(res) => Ok(res),
            Err(err) => Err(err),
        }
    }

    pub async fn auth_log(id: &str, log: AuthenticationLog, db: &DbConn) -> Result<Model, DbErr> {
        let mut kiosk = Self::fetch_by_id(id, db).await?;

        let insert_crud = ActiveModel {
            id: Set(kiosk.id),
            name: Set(kiosk.name),
            store_id: Set(kiosk.store_id),
            preferences: Set(json!(kiosk.preferences)),
            disabled: Set(kiosk.disabled as i8),
            last_online: Set(kiosk.last_online.naive_utc()),
            login_history: Set(json!(kiosk.login_history.push(History {
                item: log,
                reason: "Auth Log".to_string(),
                timestamp: Utc::now()
            }))),
        };

        match Ksk::update(insert_crud).exec(db).await {
            Ok(res) => Ok(res),
            Err(err) => Err(err),
        }
    }

    pub async fn update_preferences(
        id: &str,
        preferences: KioskPreferences,
        db: &DbConn,
    ) -> Result<Model, DbErr> {
        let kiosk = Self::fetch_by_id(id, db).await?;

        let insert_crud = ActiveModel {
            id: Set(kiosk.id),
            name: Set(kiosk.name),
            store_id: Set(kiosk.store_id),
            preferences: Set(json!(preferences)),
            disabled: Set(kiosk.disabled as i8),
            last_online: Set(kiosk.last_online.naive_utc()),
            login_history: Set(json!(kiosk.login_history)),
        };

        match Ksk::update(insert_crud).exec(db).await {
            Ok(res) => Ok(res),
            Err(err) => Err(err),
        }
    }

    pub async fn update_online_to_now(id: &str, db: &DbConn) -> Result<Model, DbErr> {
        let kiosk = Self::fetch_by_id(id, db).await?;

        let insert_crud = ActiveModel {
            id: Set(kiosk.id),
            name: Set(kiosk.name),
            store_id: Set(kiosk.store_id),
            preferences: Set(json!(kiosk.preferences)),
            disabled: Set(kiosk.disabled as i8),
            last_online: Set(Utc::now().naive_utc()),
            login_history: Set(json!(kiosk.login_history)),
        };

        match Ksk::update(insert_crud).exec(db).await {
            Ok(res) => Ok(res),
            Err(err) => Err(err),
        }
    }
}
