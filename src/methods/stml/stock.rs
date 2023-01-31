use serde::{Deserialize, Serialize};

use crate::methods::Location;

pub type StockList = Vec<Stock>;

#[derive(Deserialize, Serialize, Clone)]
pub struct Stock {
    pub store: Location,
    pub quantity: Quantity
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Quantity {
    pub quantity_sellable: f32,
    pub quantity_unsellable: f32,
    pub quantity_on_order: f32,
    pub quantity_allocated: f32,
}