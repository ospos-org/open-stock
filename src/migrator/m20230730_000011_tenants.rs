use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20230730_000011_tenants"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Tenants::Table)
                    .engine("InnoDB".to_string())
                    .col(
                        ColumnDef::new(Tenants::TenantId)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Tenants::RegistrationDate)
                            .date_time()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Tenants::Settings).json().not_null())
                    .col(ColumnDef::new(Tenants::CreatedAt).date_time().not_null())
                    .col(ColumnDef::new(Tenants::UpdatedAt).date_time().not_null())
                    .to_owned(),
            )
            .await
    }

    // Define how to rollback this migration: Drop the Bakery table.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Tenants::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum Tenants {
    #[iden = "Tenants"]
    Table,
    #[iden = "tenant_id"]
    TenantId,
    #[iden = "registration_date"]
    RegistrationDate,
    #[iden = "settings"]
    Settings,
    #[iden = "created_at"]
    CreatedAt,
    #[iden = "updated_at"]
    UpdatedAt,
}
