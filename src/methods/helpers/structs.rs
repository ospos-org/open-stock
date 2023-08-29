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
