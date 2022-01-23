use chrono::prelude::*;
use std::fmt;
use redis::{self};
use serde::{Serialize, Deserialize};

pub fn title(input: &str) -> String {
    return format!("{}{}{}", "[", input.to_uppercase(), "]");
}

pub fn log(origin: &str, descriptor: &str) {
    let time = Local::now();

    println!("{} {} - {}",time.format("%Y-%m-%d %H:%M:%S").to_string(), title(origin), descriptor);
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Product {
    pub base_name: String,
    pub id: i64,
    pub handle: String,
    pub variations: Vec<Variation>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Variation {
    pub stock_locations: Vec<Location>,
    pub price: f32,
    pub name: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Location {
    pub l_id: u64,
    pub items: i64
}

// impl fmt::Debug for Product {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         f.debug_struct("Location")
//          .field("Name", &self.base_name)
//          .field("ID", &self.id)
//          .finish()
//     }
// }