use sea_orm::{EnumIter, Iterable};
use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20230730_000003_transactions"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Transactions::Table)
                    .engine("InnoDB".to_string())
                    .col(
                        ColumnDef::new(Transactions::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Transactions::Customer).json().not_null())
                    .col(
                        ColumnDef::new(Transactions::TransactionType)
                            .enumeration(TransactionType::Table, TransactionType::iter().skip(1))
                            .not_null(),
                    )
                    .col(ColumnDef::new(Transactions::Products).json().not_null())
                    .col(ColumnDef::new(Transactions::OrderTotal).float().not_null())
                    .col(ColumnDef::new(Transactions::Payment).json().not_null())
                    .col(
                        ColumnDef::new(Transactions::OrderDate)
                            .date_time()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Transactions::OrderNotes).json().not_null())
                    .col(ColumnDef::new(Transactions::Salesperson).text().not_null())
                    .col(ColumnDef::new(Transactions::Kiosk).text().not_null())
                    .to_owned(),
            )
            .await
    }

    // Define how to rollback this migration: Drop the Bakery table.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Transactions::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum Transactions {
    #[iden = "Transactions"]
    Table,
    #[iden = "id"]
    Id,
    #[iden = "customer"]
    Customer,
    #[iden = "transaction_type"]
    TransactionType,
    #[iden = "products"]
    Products,
    #[iden = "order_total"]
    OrderTotal,
    #[iden = "payment"]
    Payment,
    #[iden = "order_date"]
    OrderDate,
    #[iden = "order_notes"]
    OrderNotes,
    #[iden = "salesperson"]
    Salesperson,
    #[iden = "kiosk"]
    Kiosk,
}

#[derive(Iden, EnumIter)]
pub enum TransactionType {
    Table,
    #[iden = "in"]
    In,
    #[iden = "out"]
    Out,
    #[iden = "pending-in"]
    PendingIn,
    #[iden = "pending-out"]
    PendingOut,
    #[iden = "saved"]
    Saved,
    #[iden = "quote"]
    Quote,
}
