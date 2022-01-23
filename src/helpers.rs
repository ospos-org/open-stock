use chrono::prelude::*;

pub fn title(input: &str) -> String {
    return format!("{}{}{}", "[", input.to_uppercase(), "]");
}

pub fn log(origin: &str, descriptor: &str) {
    let time = Local::now();

    println!("{} {} - {}",time.format("%Y-%m-%d %H:%M:%S").to_string(), title(origin), descriptor);
}