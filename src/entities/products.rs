//! `SeaORM` Entity. Generated by sea-orm-codegen 0.10.3

use sea_orm::entity::prelude::*;
use std::convert::TryInto;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "Products")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub sku: String,
    pub name: String,
    pub variants: Json,
    #[sea_orm(column_type = "Text")]
    pub loyalty_discount: String,
    pub images: Json,
    pub tags: Json,
    #[sea_orm(column_type = "Text")]
    pub description: String,
    pub specifications: Json,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
