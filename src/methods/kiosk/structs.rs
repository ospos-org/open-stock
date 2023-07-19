use serde::{Deserialize, Serialize};

use crate::History;

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
    last_online: String,

    // A list of all login attempts to the kiosk
    login_history: Vec<History<AuthenticationLog>>,
}
