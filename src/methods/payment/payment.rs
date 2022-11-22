use std::fmt::Display;

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

impl Display for PaymentMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PaymentMethod::Card => write!(f, "CARD"),
            PaymentMethod::Cash => write!(f, "CASH"),
            PaymentMethod::Transfer => write!(f, "TRANSFER"),
        }
    }
}

impl Display for Payment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} on {}", self.payment_method, self.fulfillment_date.format("%d/%m/%Y %H:%M"))
    }
}