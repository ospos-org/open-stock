use chrono::{DateTime, Utc};
use sea_orm::ActiveValue::Set;
use serde_json::json;
#[cfg(feature = "process")]
use crate::entities::{
    sea_orm_active_enums::TransactionType as SeaORMTType
};
use crate::{NoteList, OrderList, Payment, Session, Transaction, TransactionCustomer, TransactionInit, TransactionInput, TransactionType};
use crate::transactions::{ActiveModel, Model};

impl From<SeaORMTType> for TransactionType {
    fn from(value: SeaORMTType) -> Self {
        match value {
            SeaORMTType::In => TransactionType::In,
            SeaORMTType::Out => TransactionType::Out,
            SeaORMTType::PendingIn => TransactionType::PendingIn,
            SeaORMTType::PendingOut => TransactionType::PendingOut,
            SeaORMTType::Saved => TransactionType::Saved,
            SeaORMTType::Quote => TransactionType::Quote,
        }
    }
}

impl From<TransactionType> for SeaORMTType {
    fn from(value: TransactionType) -> Self {
        match value {
            TransactionType::In => SeaORMTType::In,
            TransactionType::Out => SeaORMTType::Out,
            TransactionType::PendingIn => SeaORMTType::PendingIn,
            TransactionType::PendingOut => SeaORMTType::PendingOut,
            TransactionType::Saved => SeaORMTType::Saved,
            TransactionType::Quote => SeaORMTType::Quote,
        }
    }
}

impl TransactionInput {
    pub(crate) fn into_active(self, id: String, session: Session) -> ActiveModel {
        ActiveModel {
            id: Set(id),
            customer: Set(json!(self.customer)),
            transaction_type: Set(self.transaction_type.into()),
            products: Set(json!(self.products)),
            order_total: Set(self.order_total),
            payment: Set(json!(self.payment)),
            order_date: Set(self.order_date.naive_utc()),
            order_notes: Set(json!(self.order_notes)),
            salesperson: Set(session.employee.id),
            kiosk: Set(self.kiosk),
            tenant_id: Set(session.tenant_id),
            created_at: Set(Utc::now().naive_utc()),
            updated_at: Set(Utc::now().naive_utc()),
        }
    }
}

impl TransactionInit {
    pub(crate) fn into_active(self, id: String, session: Session) -> ActiveModel {
        ActiveModel {
            id: Set(id),
            customer: Set(json!(self.customer)),
            transaction_type: Set(self.transaction_type.into()),
            products: Set(json!(self.products)),
            order_total: Set(self.order_total),
            payment: Set(json!(self.payment)),
            order_date: Set(self.order_date.naive_utc()),
            order_notes: Set(json!(self.order_notes)),
            salesperson: Set(session.employee.id),
            kiosk: Set(self.kiosk),
            tenant_id: Set(session.tenant_id),
            created_at: Set(Utc::now().naive_utc()),
            updated_at: Set(Utc::now().naive_utc()),
        }
    }
}

impl Transaction {
    pub(crate) fn into_active(self, tenant_id: String) -> ActiveModel {
        ActiveModel {
            id: Set(self.id),
            customer: Set(json!(self.customer)),
            transaction_type: Set(self.transaction_type.into()),
            products: Set(json!(self.products)),
            order_total: Set(self.order_total),
            payment: Set(json!(self.payment)),
            order_date: Set(self.order_date.naive_utc()),
            order_notes: Set(json!(self.order_notes)),
            salesperson: Set(self.salesperson),
            kiosk: Set(self.kiosk),
            tenant_id: Set(tenant_id),
            created_at: Set(self.created_at.naive_utc()),
            updated_at: Set(self.updated_at.naive_utc())
        }
    }
}

impl From<Model> for Transaction {
    fn from(val: Model) -> Self {
        Transaction {
            id: val.id,
            transaction_type: val.transaction_type.into(),

            customer: serde_json::from_value::<TransactionCustomer>(val.customer).unwrap(),
            products: serde_json::from_value::<OrderList>(val.products).unwrap(),

            order_total: val.order_total,
            payment: serde_json::from_value::<Vec<Payment>>(val.payment).unwrap(),

            order_date: DateTime::from_naive_utc_and_offset(val.order_date, Utc),
            order_notes: serde_json::from_value::<NoteList>(val.order_notes).unwrap(),

            salesperson: val.salesperson,
            kiosk: val.kiosk,

            created_at: DateTime::from_naive_utc_and_offset(val.created_at, Utc),
            updated_at: DateTime::from_naive_utc_and_offset(val.updated_at, Utc),
        }
    }
}