use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20230730_000006_session"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Session::Table)
                    .engine("InnoDB".to_string())
                    .col(
                        ColumnDef::new(Session::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Session::TenantId).string().not_null())
                    .col(ColumnDef::new(Session::Key).text().not_null())
                    .col(ColumnDef::new(Session::EmployeeId).text().not_null())
                    .col(ColumnDef::new(Session::Expiry).date_time().not_null())
                    .col(ColumnDef::new(Session::Variant).json().not_null())
                    .to_owned(),
            )
            .await
    }

    // Define how to rollback this migration: Drop the Bakery table.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Session::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum Session {
    #[iden = "Session"]
    Table,
    #[iden = "id"]
    Id,
    #[iden = "key"]
    Key,
    #[iden = "employee_id"]
    EmployeeId,
    #[iden = "expiry"]
    Expiry,
    #[iden = "tenant_id"]
    TenantId,
    #[iden = "variant"]
    Variant,
}
