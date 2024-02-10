use crate::entities::supplier::{ActiveModel, Model};
use crate::{ContactInformation, Name, Supplier, SupplierInput, Transaction};
use chrono::{DateTime, Utc};
use sea_orm::ActiveValue::Set;
use serde_json::json;

impl From<Model> for Supplier {
    fn from(val: Model) -> Self {
        Supplier {
            id: val.id,
            name: serde_json::from_value::<Name>(val.name).unwrap(),
            contact: serde_json::from_value::<ContactInformation>(val.contact).unwrap(),
            transaction_history: serde_json::from_value::<Vec<Transaction>>(
                val.transaction_history,
            )
            .unwrap(),
            created_at: DateTime::from_naive_utc_and_offset(val.created_at, Utc),
            updated_at: DateTime::from_naive_utc_and_offset(val.updated_at, Utc),
        }
    }
}

impl SupplierInput {
    pub(crate) fn into_active(self, id: String, tenant_id: String) -> ActiveModel {
        ActiveModel {
            id: Set(id.to_string()),
            name: Set(json!(self.name)),
            contact: Set(json!(self.contact)),
            transaction_history: Set(json!(self.transaction_history)),
            tenant_id: Set(tenant_id),
            created_at: Set(Utc::now().naive_utc()),
            updated_at: Set(Utc::now().naive_utc()),
        }
    }
}
