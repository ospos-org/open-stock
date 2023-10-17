use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20230730_000004_employee"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Employee::Table)
                    .engine("InnoDB".to_string())
                    .col(
                        ColumnDef::new(Employee::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Employee::TenantId).string().not_null())
                    .col(ColumnDef::new(Employee::RId).string().not_null())
                    .col(ColumnDef::new(Employee::Name).json().not_null())
                    .col(ColumnDef::new(Employee::Contact).json().not_null())
                    .col(ColumnDef::new(Employee::Auth).json().not_null())
                    .col(ColumnDef::new(Employee::ClockHistory).json().not_null())
                    .col(ColumnDef::new(Employee::Level).json().not_null())
                    .col(ColumnDef::new(Employee::AccountType).json().not_null())
                    .col(ColumnDef::new(Employee::CreatedAt).date_time().not_null())
                    .col(ColumnDef::new(Employee::UpdatedAt).date_time().not_null())
                    .to_owned(),
            )
            .await
    }

    // Define how to rollback this migration: Drop the Bakery table.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Employee::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum Employee {
    #[iden = "Employee"]
    Table,
    #[iden = "id"]
    Id,
    #[iden = "rid"]
    RId,
    #[iden = "name"]
    Name,
    #[iden = "contact"]
    Contact,
    #[iden = "auth"]
    Auth,
    #[iden = "clock_history"]
    ClockHistory,
    #[iden = "level"]
    Level,
    #[iden = "tenant_id"]
    TenantId,
    #[iden = "account_type"]
    AccountType,
    #[iden = "created_at"]
    CreatedAt,
    #[iden = "updated_at"]
    UpdatedAt,
}
