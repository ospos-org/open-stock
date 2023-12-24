use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::{Customer, Employee, Kiosk, Product, Promotion, Store, Tenant, Transaction};

#[derive(Serialize, Deserialize, JsonSchema, Validate)]
pub struct Distance {
    pub store_id: String,
    pub store_code: String,
    /// Defaults to the diameter of the earth, i.e. longest distance between two
    /// points (minimizes priority if incorrect data is provided)
    pub distance: f64,
}

#[cfg(feature = "types")]
#[derive(Serialize, Deserialize, JsonSchema, Validate)]
pub struct All {
    pub employee: Employee,
    pub stores: Vec<Store>,
    pub tenants: Vec<Tenant>,
    pub products: Vec<Product>,
    pub customer: Customer,
    pub transaction: Transaction,
    pub promotions: Vec<Promotion>,
    pub kiosk: Kiosk,
}

#[derive(Serialize, Deserialize, Clone, JsonSchema, Validate)]
pub struct NewTenantInput {
    pub(crate) name: String,
    pub(crate) email: String,
    pub(crate) address: String,
    pub(crate) password: String,
}

#[derive(Serialize, Deserialize, Clone, JsonSchema, Validate)]
pub struct NewTenantResponse {
    pub tenant_id: String,
    pub api_key: String,
    pub employee_id: String,
}
