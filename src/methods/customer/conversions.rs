use crate::entities::customer::ActiveModel;
use crate::{ContactInformation, Customer, CustomerInput, NoteList};
use chrono::{DateTime, Utc};
use sea_orm::ActiveValue::Set;
use serde_json::json;
use uuid::Uuid;

#[cfg(feature = "process")]
use crate::entities::customer;

impl CustomerInput {
    pub(crate) fn into_active(self, tenant_id: String) -> ActiveModel {
        let id = Uuid::new_v4().to_string();

        ActiveModel {
            id: Set(id),
            name: Set(self.name),

            contact: Set(json!(self.contact.into_major())),
            customer_notes: Set(json!(self.customer_notes)),

            balance: Set(self.balance),
            special_pricing: Set(json!(self.special_pricing)),
            accepts_marketing: Set(self.accepts_marketing),
            tenant_id: Set(tenant_id),

            created_at: Set(Utc::now().naive_utc()),
            updated_at: Set(Utc::now().naive_utc()),
        }
    }

    pub(crate) fn from_existing(self, customer: Customer, tenant_id: String) -> ActiveModel {
        ActiveModel {
            id: Set(customer.id),
            name: Set(self.name),

            contact: Set(json!(self.contact.into_major())),
            customer_notes: Set(json!(self.customer_notes)),
            accepts_marketing: Set(self.accepts_marketing),
            tenant_id: Set(tenant_id),

            updated_at: Set(Utc::now().naive_utc()),

            ..Default::default()
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

impl From<customer::Model> for Customer {
    fn from(val: customer::Model) -> Self {
        Customer {
            id: val.id,
            name: val.name,
            contact: serde_json::from_value::<ContactInformation>(val.contact).unwrap(),
            customer_notes: serde_json::from_value::<NoteList>(val.customer_notes).unwrap(),
            special_pricing: serde_json::from_value::<String>(val.special_pricing).unwrap(),
            balance: val.balance,
            accepts_marketing: val.accepts_marketing,
            created_at: DateTime::from_naive_utc_and_offset(val.created_at, Utc),
            updated_at: DateTime::from_naive_utc_and_offset(val.updated_at, Utc),
        }
    }
}

impl From<&customer::Model> for Customer {
    fn from(val: &customer::Model) -> Self {
        Customer {
            id: val.id.clone(),
            name: val.name.clone(),
            contact: serde_json::from_value::<ContactInformation>(val.contact.clone()).unwrap(),
            customer_notes: serde_json::from_value::<NoteList>(val.customer_notes.clone()).unwrap(),
            special_pricing: serde_json::from_value::<String>(val.special_pricing.clone()).unwrap(),
            balance: val.balance,
            accepts_marketing: val.accepts_marketing,
            created_at: DateTime::from_naive_utc_and_offset(val.created_at, Utc),
            updated_at: DateTime::from_naive_utc_and_offset(val.updated_at, Utc),
        }
    }
}
