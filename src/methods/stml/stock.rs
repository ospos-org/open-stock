use serde::{Deserialize, Serialize};

use crate::methods::Location;

pub type StockList = Vec<Stock>;

#[derive(Deserialize, Serialize)]
pub struct Stock {
    pub store: Location,
    pub quantity: Quantity
}

#[derive(Deserialize, Serialize)]
pub struct Quantity {
    pub quantity_on_hand: i32,
    pub quantity_on_order: i32,
    pub quantity_on_floor: i32
}