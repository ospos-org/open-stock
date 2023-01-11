use std::fmt::Display;

use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Payment {
    pub id: String,
    pub payment_method: PaymentMethod,
    pub fulfillment_date: DateTime<Utc>,

    pub amount: Price,
    pub processing_fee: Price,

    pub status: PaymentStatus,
    pub processor: PaymentProcessor,
    pub order_ids: Vec<String>,

    pub delay_action: PaymentAction,
    /// Duration in the RFC3339 format
    pub delay_duration: String 
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Price {
    pub quantity: f32,
    pub currency: String
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PaymentProcessor {
    pub location: String,
    pub employee: String,
    pub software_version: String,
    pub token: String
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum PaymentStatus {
    Unfulfilled, Pending, Processing, Failed(CardDetails), Complete(CardDetails)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum PaymentAction {
    Cancel, Complete, RequireFurtherAction
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CardDetails {
    pub card_brand: String,
    pub last_4: String,
    pub exp_month: String,
    pub exp_year: String,
    pub fingerprint: String,
    pub card_type: String,
    pub prepaid_type: String,
    pub bin: String,

    pub entry_method: String,
    pub cvv_accepted: String,
    pub avs_accepted: String,
    pub auth_result_code: String,
    pub statement_description: String,
    pub card_payment_timeline: PaymentTimeline
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PaymentTimeline {
    pub authorized_at: String,
    pub captured_at: String
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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