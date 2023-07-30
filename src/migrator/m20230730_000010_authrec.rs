use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20230730_000010_authrec"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(AuthRecord::Table)
                    .col(
                        ColumnDef::new(AuthRecord::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(AuthRecord::KioskId).text().not_null())
                    .col(ColumnDef::new(AuthRecord::Attempt).json().not_null())
                    .col(ColumnDef::new(AuthRecord::Timestamp).date_time().not_null())
                    .to_owned(),
            )
            .await
    }

    // Define how to rollback this migration: Drop the Bakery table.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AuthRecord::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum AuthRecord {
    #[iden = "AuthRecord"]
    Table,
    #[iden = "id"]
    Id,
    #[iden = "kiosk_id"]
    KioskId,
    #[iden = "attempt"]
    Attempt,
    #[iden = "timestamp"]
    Timestamp,
}
