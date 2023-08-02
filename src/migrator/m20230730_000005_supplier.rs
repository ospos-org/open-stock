use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20230730_000005_supplier"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Supplier::Table)
                    .engine("InnoDB".to_string())
                    .col(
                        ColumnDef::new(Supplier::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Supplier::TenantId).string().not_null())
                    .col(ColumnDef::new(Supplier::Name).json().not_null())
                    .col(ColumnDef::new(Supplier::Contact).json().not_null())
                    .col(
                        ColumnDef::new(Supplier::TransactionHistory)
                            .json()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await
    }

    // Define how to rollback this migration: Drop the Bakery table.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Supplier::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum Supplier {
    #[iden = "Supplier"]
    Table,
    #[iden = "id"]
    Id,
    #[iden = "name"]
    Name,
    #[iden = "contact"]
    Contact,
    #[iden = "transaction_history"]
    TransactionHistory,
    #[iden = "tenant_id"]
    TenantId,
}
