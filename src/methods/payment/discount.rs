use std::{str::FromStr};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiscountValue {
    Percentage(u32), Absolute(u32)
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
            DiscountValue::Absolute(val) => ("a", val)
        };

        format!("{}|{}", del, val)
    }
}

impl FromStr for DiscountValue {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let split = s.split("|").collect::<Vec<&str>>();

        let val = match split.get(1) {
            Some(va) => {
                match va.parse::<u32>() {
                    Ok(v) => Ok(v),
                    Err(_) => Err(format!("Was unable to parse value (type: u32) of DiscountValue when in String form, defaulting to 0.")),
                }
            }
            None => Err(format!("Unable to split DiscountValue string, given: {:?}", split))
        };

        match val {
            Ok(v) => {
                match *split.get(0).unwrap() {
                    "p" => Ok(DiscountValue::Percentage(v)),
                    "a" => Ok(DiscountValue::Absolute(v)),
                    _ => Err(format!("Was unable to convert String to DiscountValue, defaulting to 0."))
                }
            }
            Err(err) => {
                Err(err)
            }
        }
    }
}