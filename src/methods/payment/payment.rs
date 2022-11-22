use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Payment {
    pub payment_method: PaymentMethod,
    pub fulfillment_date: DateTime<Utc>
}

#[derive(Debug, Serialize, Deserialize)]
pub enum PaymentMethod {
    Card, Cash, Transfer
}