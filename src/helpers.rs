use chrono::prelude::*;
use serde::{Serialize, Deserialize};

use std::io::{Write};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

pub fn title(input: &str) -> String {
    return format!("{}{}{}", "[", input.to_uppercase(), "]");
}

pub fn log(origin: &str, descriptor: &str) {
    let time = Local::now();

    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    write_with_color(&mut stdout, Color::Green, format!("\n{}", time.format("%Y-%m-%d %H:%M:%S").to_string()).to_string());
    write_with_color(&mut stdout, Color::Blue, format!(" {}", title(origin)).to_string());
    write_with_color(&mut stdout, Color::White, format!("\t{}", descriptor));
}

fn write_with_color(stdout: &mut StandardStream, color: Color, output: String) {
    let _res = stdout.set_color(ColorSpec::new().set_fg(Some(color)));
    let _res2 = write!(stdout, "{}", output);
}

pub fn redis_test(con: &mut redis::Connection) -> String {
    let demo = Product {
        base_name: "demo_product".to_string(),
        id: 019238190283018,
        handle: "pd25932918".to_string(),
        variations: vec![
            Variation {
                stock_locations: vec![
                    Location {
                        l_id: 1237197395129,
                        items: 15
                    }
                ],
                price: 162.99,
                name: "demo_product small".to_string()
            }
        ]
    };

    let response: Result<(String,), redis::RedisError> = redis::pipe()
        .atomic()
        .cmd("SET").arg(demo.id).arg(serde_json::to_string(&demo).unwrap()).ignore()
        .cmd("GET").arg(demo.id)
        .query(con);

    match response {
        Err(error) => {
            // log("redis", &error.to_string());

            return error.to_string();
        }
        Ok(response) => {
            let (serialized, ) = response;

            let _json: Product = serde_json::from_str(&serialized.to_string()).unwrap();
            // println!("{:?}", json);

            // log("redis", &serialized);

            return serialized;
        }
    }
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