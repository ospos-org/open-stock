use chrono::{DateTime, Utc};
use crate::methods::Customer;

pub struct Payment {
    payment_method: String,
    fulfillment_date: DateTime<Utc>
}