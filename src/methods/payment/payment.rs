use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Debug, Serialize, sqlx::Type)]
pub struct Payment {
    pub payment_method: PaymentMethod,
    pub fulfillment_date: DateTime<Utc>
}

#[derive(Debug, Serialize)]
pub enum PaymentMethod {
    Card, Cash, Transfer
}