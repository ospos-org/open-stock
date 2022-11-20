use chrono::{DateTime, Utc};

#[derive(Debug)]
pub struct Payment {
    pub payment_method: PaymentMethod,
    pub fulfillment_date: DateTime<Utc>
}

#[derive(Debug)]
pub enum PaymentMethod {
    Card, Cash, Transfer
}