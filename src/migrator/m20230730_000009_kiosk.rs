use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20230730_000009_kiosk"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Kiosk::Table)
                    .col(ColumnDef::new(Kiosk::Id).string().not_null().primary_key())
                    .col(ColumnDef::new(Kiosk::Name).text().not_null())
                    .col(ColumnDef::new(Kiosk::StoreId).string().not_null())
                    .col(ColumnDef::new(Kiosk::Preferences).json().not_null())
                    .col(ColumnDef::new(Kiosk::Disabled).boolean().not_null())
                    .col(ColumnDef::new(Kiosk::LastOnline).date_time().not_null())
                    .to_owned(),
            )
            .await
    }

    // Define how to rollback this migration: Drop the Bakery table.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Kiosk::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum Kiosk {
    Table,
    #[iden = "id"]
    Id,
    #[iden = "name"]
    Name,
    #[iden = "store_id"]
    StoreId,
    #[iden = "preferences"]
    Preferences,
    #[iden = "disabled"]
    Disabled,
    #[iden = "last_online"]
    LastOnline,
}
