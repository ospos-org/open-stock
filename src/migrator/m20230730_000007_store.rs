use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20230730_000007_store"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Store::Table)
                    .engine("InnoDB".to_string())
                    .col(ColumnDef::new(Store::Id).string().not_null().primary_key())
                    .col(ColumnDef::new(Store::Name).text().not_null())
                    .col(ColumnDef::new(Store::Contact).json().not_null())
                    .col(ColumnDef::new(Store::Code).text().not_null())
                    .to_owned(),
            )
            .await
    }

    // Define how to rollback this migration: Drop the Bakery table.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Store::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum Store {
    #[iden = "Store"]
    Table,
    #[iden = "id"]
    Id,
    #[iden = "name"]
    Name,
    #[iden = "contact"]
    Contact,
    #[iden = "code"]
    Code,
}
