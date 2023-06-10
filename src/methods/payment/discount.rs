use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiscountValue {
    Percentage(u32),
    Absolute(u32),
}

/*
    Format: [del] | [val]
        [del] represents the type (delimiter) - p for Percentage, a for Absolute
        [val] represents the value (defaults to negative as it is a discount - only unsigned int)

    e.g. a|5 ($5.00 absolute)
         p|0.15 (15% percentage)
*/

impl ToString for DiscountValue {
    fn to_string(&self) -> String {
        let (del, val) = match self {
            DiscountValue::Percentage(val) => ("p", val),
            DiscountValue::Absolute(val) => ("a", val),
        };

        format!("{}|{}", del, val)
    }
}

pub type DiscountMap = Vec<DiscountValue>;

pub fn greatest_discount(map: DiscountMap, price: f32) -> DiscountValue {
    let mut greatest_discount = DiscountValue::Absolute(0);

    for item in map {
        if is_greater_discount(greatest_discount.clone(), item.clone(), price) {
            greatest_discount = item.clone()
        }
    }

    greatest_discount
}

pub fn is_greater_discount(predicate: DiscountValue, discount: DiscountValue, price: f32) -> bool {
    apply_discount(discount, price) < apply_discount(predicate, price)
}

pub fn apply_discount(discount: DiscountValue, price: f32) -> f32 {
    match discount {
        DiscountValue::Percentage(val) => price - (price * ((val as f32) / 100.00)),
        DiscountValue::Absolute(val) => price - (val as f32),
    }
}

impl FromStr for DiscountValue {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let split = s.split('|').collect::<Vec<&str>>();

        let val = match split.get(1) {
            Some(va) => {
                match va.parse::<u32>() {
                    Ok(v) => Ok(v),
                    Err(_) => Err("Was unable to parse value (type: u32) of DiscountValue when in String form, defaulting to 0.".to_string()),
                }
            }
            None => Err(format!("Unable to split DiscountValue string, given: {:?}", split))
        };

        match val {
            Ok(v) => match *split.first().unwrap() {
                "p" => Ok(DiscountValue::Percentage(v)),
                "a" => Ok(DiscountValue::Absolute(v)),
                _ => Err(
                    "Was unable to convert String to DiscountValue, defaulting to 0.".to_string(),
                ),
            },
            Err(err) => Err(err),
        }
    }
}
