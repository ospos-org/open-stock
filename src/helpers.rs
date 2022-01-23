use chrono::prelude::*;
use serde::{Serialize, Deserialize};

use std::io::{self, Write};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

pub fn title(input: &str) -> String {
    return format!("{}{}{}", "[", input.to_uppercase(), "]");
}

pub fn log(origin: &str, descriptor: &str) {
    let time = Local::now();

    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green)));
    write!(&mut stdout, "\n{}", time.format("%Y-%m-%d %H:%M:%S").to_string());
    stdout.set_color(ColorSpec::new().set_fg(Some(Color::Blue)));
    write!(&mut stdout, " {}", title(origin));
    stdout.set_color(ColorSpec::new().set_fg(Some(Color::Black)));
    write!(&mut stdout, "\t{}", descriptor);

    // println!("{} {} - {}",time.format("%Y-%m-%d %H:%M:%S").to_string(), title(origin), descriptor);
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