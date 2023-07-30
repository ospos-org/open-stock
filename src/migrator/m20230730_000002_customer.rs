use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20230730_000002_customer"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Customer::Table)
                    .col(
                        ColumnDef::new(Customer::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Customer::Name).text().not_null())
                    .col(ColumnDef::new(Customer::Contact).json().not_null())
                    .col(ColumnDef::new(Customer::CustomerNotes).json().not_null())
                    .col(ColumnDef::new(Customer::Balance).float().not_null())
                    .col(ColumnDef::new(Customer::SpecialPricing).json().not_null())
                    .col(
                        ColumnDef::new(Customer::AcceptsMarketing)
                            .boolean()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await
    }

    // Define how to rollback this migration: Drop the Bakery table.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Customer::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum Customer {
    Table,
    #[iden = "id"]
    Id,
    #[iden = "name"]
    Name,
    #[iden = "contact"]
    Contact,
    #[iden = "customer_notes"]
    CustomerNotes,
    #[iden = "balance"]
    Balance,
    #[iden = "special_pricing"]
    SpecialPricing,
    #[iden = "accepts_marketing"]
    AcceptsMarketing,
}
