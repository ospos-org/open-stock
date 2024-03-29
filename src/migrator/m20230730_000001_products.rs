use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20230730_000001_products"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        manager
            .create_table(
                Table::create()
                    .table(Products::Table)
                    .engine("InnoDB".to_string())
                    .col(
                        ColumnDef::new(Products::Sku)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Products::Name).string().not_null())
                    .col(ColumnDef::new(Products::NameLong).string().not_null())
                    .col(ColumnDef::new(Products::TenantId).string().not_null())
                    .col(ColumnDef::new(Products::Company).string().not_null())
                    .col(ColumnDef::new(Products::Variants).json().not_null())
                    .col(ColumnDef::new(Products::VariantGroups).json().not_null())
                    .col(ColumnDef::new(Products::Images).json().not_null())
                    .col(ColumnDef::new(Products::Tags).json().not_null())
                    .col(ColumnDef::new(Products::Identification).json().not_null())
                    .col(ColumnDef::new(Products::Description).text().not_null())
                    .col(ColumnDef::new(Products::DescriptionLong).text().not_null())
                    .col(ColumnDef::new(Products::Specifications).json().not_null())
                    .col(ColumnDef::new(Products::Visible).json().not_null())
                    .col(ColumnDef::new(Products::CreatedAt).date_time().not_null())
                    .col(ColumnDef::new(Products::UpdatedAt).date_time().not_null())
                    .to_owned(),
            )
            .await?;

        db.execute_unprepared("ALTER TABLE `Products` ADD FULLTEXT indx(`name`,`company`)")
            .await?;

        Ok(())
    }

    // Define how to rollback this migration: Drop the Bakery table.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Products::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum Products {
    #[iden = "Products"]
    Table,
    #[iden = "sku"]
    Sku,
    #[iden = "name"]
    Name,
    #[iden = "name_long"]
    NameLong,
    #[iden = "company"]
    Company,
    #[iden = "variants"]
    Variants,
    #[iden = "variant_groups"]
    VariantGroups,
    #[iden = "images"]
    Images,
    #[iden = "tags"]
    Tags,
    #[iden = "identification"]
    Identification,
    #[iden = "description"]
    Description,
    #[iden = "description_long"]
    DescriptionLong,
    #[iden = "specifications"]
    Specifications,
    #[iden = "visible"]
    Visible,
    #[iden = "tenant_id"]
    TenantId,
    #[iden = "created_at"]
    CreatedAt,
    #[iden = "updated_at"]
    UpdatedAt,
}
