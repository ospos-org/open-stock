use crate::methods::Location;

pub type StockList = Vec<Stock>;

pub struct Stock {
    pub store: Location,
    pub quantity: Quantity
}

pub struct Quantity {
    pub quantity_on_hand: i128,
    pub quantity_on_order: i128,
    pub quantity_on_floor: i128
}