use redis::{self};
use std::env;
use serde::{Serialize, Deserialize};
use serde_json;

mod helpers; 

/// Runs all the examples and propagates errors up.
fn start_api(url: &str) -> redis::RedisResult<()> {
    // general connection handling
    let client = redis::Client::open(url)?;
    let mut con = client.get_connection()?;

    helpers::log("redis", "connected");
    
    let demo = helpers::Product {
        base_name: "demo_product".to_string(),
        id: 019238190283018,
        handle: "pd25932918".to_string(),
        variations: vec![
            helpers::Variation {
                stock_locations: vec![
                    helpers::Location {
                        l_id: 1237197395129,
                        items: 15
                    }
                ],
                price: 162.99,
                name: "demo_product small".to_string()
            }
        ]
    };

    let response: Option<(String,)> = redis::pipe()
        .atomic()
        .cmd("SET").arg(demo.id).arg(serde_json::to_string(&demo).unwrap()).ignore()
        .cmd("GET").arg(demo.id)
        .query(&mut con)?;

    match response {
        None => {
            helpers::log("redis", "no response");
        }
        Some(response) => {
            let (serialized, ) = response;

            let json: helpers::Product = serde_json::from_str(&serialized.to_string()).unwrap();
            println!("{:?}", json);

            // helpers::log("redis", json)
        }
    }

    Ok(())
}


fn main() {
    // at this point the errors are fatal, let's just fail hard.
    let url = if env::args().nth(1) == Some("--tls".into()) {
        "redis://127.0.0.1:6380/#insecure"
    } else {
        "redis://127.0.0.1:6379/"
    };

    match start_api(url) {
        Err(err) => {
            println!("Could not execute example:");
            println!("  {}: {}", err.category(), err);
        }
        Ok(()) => {
            helpers::log("rust", "execution->complete");
        }
    }
}