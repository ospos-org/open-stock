use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20230730_000008_promotion"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Promotion::Table)
                    .engine("InnoDB".to_string())
                    .col(
                        ColumnDef::new(Promotion::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Promotion::TenantId).string().not_null())
                    .col(ColumnDef::new(Promotion::Name).text().not_null())
                    .col(ColumnDef::new(Promotion::Buy).json().not_null())
                    .col(ColumnDef::new(Promotion::Get).json().not_null())
                    .col(ColumnDef::new(Promotion::ValidTill).date_time().not_null())
                    .col(ColumnDef::new(Promotion::Timestamp).date_time().not_null())
                    .to_owned(),
            )
            .await
    }

    // Define how to rollback this migration: Drop the Bakery table.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Promotion::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum Promotion {
    #[iden = "Promotion"]
    Table,
    #[iden = "id"]
    Id,
    #[iden = "name"]
    Name,
    #[iden = "buy"]
    Buy,
    #[iden = "get"]
    Get,
    #[iden = "valid_till"]
    ValidTill,
    #[iden = "timestamp"]
    Timestamp,
    #[iden = "tenant_id"]
    TenantId,
}
