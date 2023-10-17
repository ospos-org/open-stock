use chrono::{DateTime, Utc};
use sea_orm::ActiveValue::Set;
use serde_json::json;
use crate::{Product, ProductIdentification, ProductVisibility, Session, TagList, Url, VariantCategoryList, VariantInformation};
use crate::products::{ActiveModel, Model};

impl Product {
    pub(crate) fn into_active(self, session: Session) -> ActiveModel {
        ActiveModel {
            sku: Set(self.sku),
            name: Set(self.name),
            company: Set(self.company),
            variants: Set(json!(self.variants)),
            variant_groups: Set(json!(self.variant_groups)),
            images: Set(json!(self.images)),
            tags: Set(json!(self.tags)),
            description: Set(self.description),
            specifications: Set(json!(self.specifications)),
            identification: Set(json!(self.identification)),
            visible: Set(json!(self.visible)),
            name_long: Set(self.name_long),
            description_long: Set(self.description_long),
            tenant_id: Set(session.tenant_id),
            created_at: Default::default(),
            updated_at: Default::default(),
        }
    }
}

impl Into<Product> for Model {
    fn into(self) -> Product {
        Product {
            name: self.name,
            company: self.company,
            variant_groups: serde_json::from_value::<VariantCategoryList>(self.variant_groups)
                .unwrap(),
            variants: serde_json::from_value::<Vec<VariantInformation>>(self.variants).unwrap(),
            sku: self.sku,
            images: serde_json::from_value::<Vec<Url>>(self.images).unwrap(),
            tags: serde_json::from_value::<TagList>(self.tags).unwrap(),
            description: self.description,
            specifications: serde_json::from_value::<Vec<(String, String)>>(self.specifications)
                .unwrap(),
            identification: serde_json::from_value::<ProductIdentification>(self.identification)
                .unwrap(),
            visible: serde_json::from_value::<ProductVisibility>(self.visible).unwrap(),
            created_at: DateTime::from_utc(self.created_at, Utc),
            name_long: self.name_long,
            description_long: self.description_long,
            updated_at: DateTime::from_utc(self.updated_at, Utc),
        }
    }
}