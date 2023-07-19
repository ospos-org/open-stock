use crate::entities::prelude::Kiosk as Ksk;
use crate::History;
use chrono::{DateTime, Utc};
use sea_orm::{DbConn, DbErr, EntityTrait};
use serde::{Deserialize, Serialize};

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
}
