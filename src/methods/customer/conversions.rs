use chrono::{DateTime, Utc};
use sea_orm::ActiveValue::Set;
use serde_json::json;
use uuid::Uuid;
use crate::{ContactInformation, Customer, CustomerInput, NoteList};
use crate::entities::customer::ActiveModel;

#[cfg(feature = "process")]
use crate::entities::customer;

impl CustomerInput {
    pub(crate) fn into_active(self, tenant_id: String) -> ActiveModel {
        let id = Uuid::new_v4().to_string();

        ActiveModel {
            id: Set(id),
            name: Set(self.name),

            contact: Set(json!(self.contact)),
            customer_notes: Set(json!(self.customer_notes)),

            balance: Set(self.balance),
            special_pricing: Set(json!(self.special_pricing)),
            accepts_marketing: Set(self.accepts_marketing),
            tenant_id: Set(tenant_id),

            created_at: Set(Utc::now().naive_utc()),
            updated_at: Set(Utc::now().naive_utc()),
        }
    }
}

impl Customer {
    pub(crate) fn into_active(self, tenant_id: String) -> ActiveModel {
        ActiveModel {
            id: Set(self.id),
            name: Set(self.name),

            contact: Set(json!(self.contact)),
            customer_notes: Set(json!(self.customer_notes)),

            balance: Set(self.balance),
            special_pricing: Set(json!(self.special_pricing)),
            accepts_marketing: Set(self.accepts_marketing),
            tenant_id: Set(tenant_id),

            created_at: Set(self.created_at.naive_utc()),
            updated_at: Set(self.updated_at.naive_utc()),
        }
    }
}

impl Into<Customer> for customer::Model {
    fn into(self) -> Customer {
        Customer {
            id: self.id,
            name: self.name,
            contact: serde_json::from_value::<ContactInformation>(self.contact).unwrap(),
            customer_notes: serde_json::from_value::<NoteList>(self.customer_notes).unwrap(),
            special_pricing: serde_json::from_value::<String>(self.special_pricing).unwrap(),
            balance: self.balance,
            accepts_marketing: self.accepts_marketing,
            created_at: DateTime::from_utc(self.created_at, Utc),
            updated_at: DateTime::from_utc(self.updated_at, Utc),
        }
    }
}

impl Into<Customer> for &customer::Model {
    fn into(self) -> Customer {
        Customer {
            id: self.id.clone(),
            name: self.name.clone(),
            contact: serde_json::from_value::<ContactInformation>(self.contact.clone()).unwrap(),
            customer_notes: serde_json::from_value::<NoteList>(self.customer_notes.clone()).unwrap(),
            special_pricing: serde_json::from_value::<String>(self.special_pricing.clone()).unwrap(),
            balance: self.balance,
            accepts_marketing: self.accepts_marketing,
            created_at: DateTime::from_utc(self.created_at, Utc),
            updated_at: DateTime::from_utc(self.updated_at, Utc),
        }
    }
}