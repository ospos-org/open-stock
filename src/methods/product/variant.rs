use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::methods::{StockList, HistoryList, Url};

pub type VariantIdTag = Vec<VariantId>;
type VariantId = String;

pub type VariantCategoryList = Vec<VariantCategory>;

#[derive(Deserialize, Serialize, Clone)]
pub struct VariantCategory {
    pub category: String,
    pub variants: Vec<Variant>
}

#[derive(Deserialize, Serialize, Clone)]
pub struct VariantInformation {
    pub name: String,
    pub stock: StockList,
    pub images: Vec<Url>,
    pub marginal_price: f32,
    /// The group codes for all sub-variants; i.e. is White, Short Sleeve and Small.
    pub variant_code: VariantIdTag,
    pub order_history: HistoryList,
    pub stock_information: StockInformation
}

/// Represents all sub-variant types; i.e. All 'White' variants, whether small, long-sleeve, ... it represents the sub-group of all which are 'White'.
#[derive(Deserialize, Serialize, Clone)]
pub struct Variant {
    pub name: String,
    pub images: Vec<Url>,
    pub marginal_price: i32,
    pub variant_code: String,
    pub order_history: HistoryList,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct StockInformation {
    pub stock_group: String,
    pub sales_group: String,
    pub value_stream: String,

    pub brand: String,
    pub unit: String,

    /// Non-required field which outlines the tax code of the product if necessary.
    pub tax_code: String,

    /// The variant's weight in kilograms.
    pub weight: String,

    /// The volume of the product in meters cubed, kept specific to each variant.
    pub volume: String,

    /// A quantity considered to be the *maximum*. If the quantity dips below such value, it is suggested a restock should take place.
    pub max_volume: String,

    /// If the product's supply cannot be fulfilled at the current time, due to a lack of availability. 
    /// 
    /// By setting `back_order` to `true`, it allows for the purchase of the product on the promise it will be delivered to the customer or collected from the store at a later date. **This must be made clear and known to the customer.**
    pub back_order: bool,
    /// A product which is no longer source-able. Once the product's inventory is consumed it is indicated to never be replenished.
    pub discontinued: bool,
    /// A `non_diminishing` product is often a service rather than a product, i.e. freight. It is **not removed** from inventory upon consumption, rather attached.
    pub non_diminishing: bool 
}

impl Display for Variant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,
            "\t{} ({}) ${}", 
            self.name, self.variant_code, self.marginal_price 
        )
    }
}