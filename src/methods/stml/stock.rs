use rocket_okapi::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::methods::Location;

#[cfg(feature = "types")]
pub type StockList = Vec<Stock>;

#[cfg(feature = "types")]
#[derive(Deserialize, Serialize, Clone, JsonSchema)]
pub struct Stock {
    pub store: Location,
    pub quantity: Quantity,
}

#[cfg(feature = "types")]
#[derive(Deserialize, Serialize, Clone, JsonSchema)]
pub struct Quantity {
    pub quantity_sellable: f32,
    pub quantity_unsellable: f32,
    pub quantity_on_order: f32,
    pub quantity_allocated: f32,
}
