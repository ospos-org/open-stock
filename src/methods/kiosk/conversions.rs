use crate::entities::kiosk::ActiveModel;
use crate::{Kiosk, Session};
use sea_orm::ActiveValue::Set;
use serde_json::json;

impl Kiosk {
    pub(crate) fn into_active(self, session: Session) -> ActiveModel {
        ActiveModel {
            id: Set(self.id),
            name: Set(self.name),
            store_id: Set(self.store_id),
            preferences: Set(json!(self.preferences)),
            disabled: Set(i8::from(self.disabled)),
            last_online: Set(self.last_online.naive_utc()),
            tenant_id: Set(session.tenant_id),
        }
    }
}
