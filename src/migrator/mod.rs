use sea_orm_migration::prelude::*;

mod m20230730_000001_products;
mod m20230730_000002_customer;
mod m20230730_000003_transactions;
mod m20230730_000004_employee;
mod m20230730_000005_supplier;
mod m20230730_000006_session;
mod m20230730_000007_store;
mod m20230730_000008_promotion;
mod m20230730_000009_kiosk;
mod m20230730_000010_authrec;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20230730_000001_products::Migration),
            Box::new(m20230730_000002_customer::Migration),
            Box::new(m20230730_000003_transactions::Migration),
            Box::new(m20230730_000004_employee::Migration),
            Box::new(m20230730_000005_supplier::Migration),
            Box::new(m20230730_000006_session::Migration),
            Box::new(m20230730_000007_store::Migration),
            Box::new(m20230730_000008_promotion::Migration),
            Box::new(m20230730_000009_kiosk::Migration),
            Box::new(m20230730_000010_authrec::Migration),
        ]
    }
}
