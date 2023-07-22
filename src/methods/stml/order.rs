use std::fmt::Display;

use crate::methods::{
    ContactInformation, DiscountValue, History, HistoryList, Id, Location, NoteList,
    ProductPurchaseList, Store, Url,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: Id,

    pub destination: Location,
    pub origin: Location,

    pub products: ProductPurchaseList,

    pub status: OrderStatusAssignment,
    pub status_history: Vec<History<OrderStatusAssignment>>,
    pub order_history: HistoryList,

    pub previous_failed_fulfillment_attempts: Vec<History<Store>>,

    pub order_notes: NoteList,
    pub reference: String,
    pub creation_date: DateTime<Utc>,

    pub discount: DiscountValue,
    pub order_type: OrderType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum OrderType {
    Direct,
    Shipment,
    Pickup,
    Quote,
}

impl ToString for Order {
    fn to_string(&self) -> String {
        match serde_json::to_string(self) {
            Ok(s) => s,
            Err(_) => "{}".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderState {
    pub timestamp: DateTime<Utc>,
    pub status: OrderStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderStatusAssignment {
    pub status: OrderStatus,
    pub assigned_products: Vec<Id>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "value")]
#[serde(rename_all = "lowercase")]
pub enum OrderStatus {
    /// Open Cart, Till Cart or Being Processed, the date represents the time it was placed.
    Queued(DateTime<Utc>),
    /// Delivery items: Contains a transit information docket - with assigned items and tracking information.
    Transit(Box<TransitInformation>),
    /// Click-n-collect item or Delivery being processed with date when processing started.
    Processing(DateTime<Utc>),
    /// Click-n-collect item, date represents when it was readied-for-pickup.
    InStore(DateTime<Utc>),
    /// In-store purchase or Delivered Item, date represents when it was completed.
    Fulfilled(DateTime<Utc>),
    /// Was unable to fulfill, reason is given
    Failed(String),
}

impl Display for OrderStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match self {
            OrderStatus::Queued(_) => "QUEUED",
            OrderStatus::Transit(_) => "TRANSIT",
            OrderStatus::Processing(_) => "PROCESSING",
            OrderStatus::InStore(_) => "IN-STORE",
            OrderStatus::Fulfilled(_) => "FULFILLED",
            OrderStatus::Failed(_reason) => "FAILED:",
        };

        write!(f, "{}", output)
    }
}

impl OrderStatus {
    pub fn is_queued(&self) -> bool {
        match *self {
            OrderStatus::Queued(_) => true,
            OrderStatus::Transit(_) => false,
            OrderStatus::Processing(_) => true,
            OrderStatus::InStore(_) => false,
            OrderStatus::Fulfilled(_) => false,
            OrderStatus::Failed(_) => false,
        }
    }

    pub fn is_not_fulfilled_nor_failed(&self) -> bool {
        match *self {
            OrderStatus::Queued(_) => true,
            OrderStatus::Transit(_) => true,
            OrderStatus::Processing(_) => true,
            OrderStatus::InStore(_) => false,
            OrderStatus::Fulfilled(_) => false,
            OrderStatus::Failed(_) => false,
        }
    }
}

impl Display for OrderStatusAssignment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let pdts: String = self
            .assigned_products
            .iter()
            .map(|p| p.to_string())
            .collect();

        write!(f, "Status:{}\nItems:\n{}", self.status, pdts)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TransitInformation {
    pub shipping_company: ContactInformation,
    pub query_url: Url,
    pub tracking_code: String,
    pub assigned_products: Vec<Id>,
}
