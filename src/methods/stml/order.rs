use chrono::{Utc, DateTime};
use uuid::Uuid;
use crate::methods::{Location, ProductPurchaseList, NoteList, ContactInformation, Url, DiscountValue};

#[derive(Debug)]
pub struct Order {
    pub id: Uuid,

    pub destination: Location,
    pub origin: Location,

    pub products: ProductPurchaseList,

    pub status: OrderStatus,
    pub status_history: Vec<OrderState>,

    pub order_notes: NoteList,
    pub reference: String,
    pub creation_date: DateTime<Utc>,
    
    pub discount: DiscountValue
}

#[derive(Debug)]
pub struct OrderState {
    date: DateTime<Utc>,
    status: OrderStatus
}

#[derive(Debug)]
pub enum OrderStatus {
    // Open Cart, Till Cart or Being Processed
    Queued,
    // Delivery items
    Transit(TransitInformation),
    // Click-n-collect item or Delivery being processed with date when processing started.
    Processing(DateTime<Utc>),
    // Click-n-collect item
    InStore,
    // In-store purchase or Delivered Item
    Fulfilled,
    // Was unable to fulfill, reason is given
    Failed(String)
}

#[derive(Debug)]
pub struct TransitInformation {
    pub shipping_company: ContactInformation,
    pub query_url: Url,
    pub tracking_code: String
}