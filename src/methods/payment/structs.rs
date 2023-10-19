use std::fmt::Display;

use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[cfg(feature = "types")]
#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
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
    pub delay_duration: String,
}

#[cfg(feature = "types")]
#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct Price {
    pub quantity: f32,
    pub currency: String,
}

#[cfg(feature = "types")]
#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct PaymentProcessor {
    pub location: String,
    pub employee: String,
    pub software_version: String,
    pub token: String,
}

#[cfg(feature = "types")]
impl PaymentProcessor {
    /// `anonymous(source: String) -> Self`
    ///
    /// Use this to create a payment processor from
    /// an anonymous origin. Often used to convert
    /// from unknown types or non-similar origins.
    ///
    /// e.g. Shopify Imported Transaction
    pub fn anonymous(source: String) -> Self {
        Self {
            location: source,
            employee: String::new(),
            software_version: String::new(),
            token: String::new(),
        }
    }
}

#[cfg(feature = "types")]
#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub enum Processable {
    /// CardDetails() (CardTransaction)
    CardDetails(Box<CardDetails>),
    /// Anonymous(Origin: String)
    Anonymous(String),
}

#[cfg(feature = "types")]
#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub enum PaymentStatus {
    Unfulfilled(String),
    Pending(String),
    Processing(String),
    Failed(Processable),
    Complete(Processable),
}

#[cfg(feature = "types")]
#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub enum PaymentAction {
    Cancel,
    Complete,
    RequireFurtherAction,
}

#[cfg(feature = "types")]
#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
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
    pub card_payment_timeline: PaymentTimeline,
}

#[cfg(feature = "types")]
#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct PaymentTimeline {
    pub authorized_at: String,
    pub captured_at: String,
}

#[cfg(feature = "types")]
#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub enum PaymentMethod {
    Card,
    Cash,
    Transfer,
    Other(String),
}

impl Display for PaymentMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PaymentMethod::Card => write!(f, "CARD"),
            PaymentMethod::Cash => write!(f, "CASH"),
            PaymentMethod::Transfer => write!(f, "TRANSFER"),
            PaymentMethod::Other(value) => write!(f, "OTHER[{}]", value),
        }
    }
}

impl Display for Payment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} on {}",
            self.payment_method,
            self.fulfillment_date.format("%d/%m/%Y %H:%M")
        )
    }
}
