use redis;
use std::env;
#[macro_use] extern crate rocket;

mod helpers; 

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/product/<product>")]
fn get_product(product: String) -> String {
    format!("Hello, {}!", product.as_str())
}

/// Runs all the examples and propagates errors up.
fn start_redis(url: &str) -> redis::RedisResult<()> {
    // general connection handling
    let client = redis::Client::open(url)?;
    let mut con = client.get_connection()?;

    assert_eq!(helpers::redis_test(&mut con), "{\"base_name\":\"demo_product\",\"id\":19238190283018,\"handle\":\"pd25932918\",\"variations\":[{\"stock_locations\":[{\"l_id\":1237197395129,\"items\":15}],\"price\":162.99,\"name\":\"demo_product small\"}]}".to_string());

    Ok(())
}

#[rocket::main]
async fn main() {
    // at this point the errors are fatal, let's just fail hard.
    let url = if env::args().nth(1) == Some("--tls".into()) {
        "redis://127.0.0.1:6380/#insecure"
    } else {
        "redis://127.0.0.1:6379/"
    };

    helpers::log("rust", "Starting stock.[REDIS]");

    match start_redis(url) {
        Err(err) => {
            helpers::log("redis", &format!("Failed to start redis: {}", err).to_string());
        }
        Ok(()) => {
            helpers::log("rust", "Redis Connected");
            helpers::log("rust", "Starting stock.[ROCKET]::launch");

            let result = rocket::build()
                .mount("/hello", routes![index])
                .mount("/", routes![get_product])
                .launch()
                .await;

            helpers::log("rocket", &format!("{:#?}", result).to_string());
            helpers::log("rust", "Ending stock.[ROCKET]::deorbit")
        }
    }
}