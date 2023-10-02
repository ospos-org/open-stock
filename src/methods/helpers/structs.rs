use serde::{Deserialize, Serialize};

use crate::{Customer, Employee, Kiosk, Product, Promotion, Store, Tenant, Transaction};

#[cfg(feature = "types")]
#[derive(Serialize, Deserialize)]
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

#[derive(Serialize, Deserialize, Clone)]
pub struct NewTenantInput {
    pub(crate) name: String,
    pub(crate) email: String,
    pub(crate) address: String,
    pub(crate) password: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct NewTenantResponse {
    pub tenant_id: String,
    pub api_key: String,
    pub employee_id: String,
}
