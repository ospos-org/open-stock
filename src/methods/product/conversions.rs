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
            created_at: Set(self.created_at.naive_utc()),
            updated_at: Set(self.created_at.naive_utc()),
        }
    }
}

impl From<Model> for Product {
    fn from(val: Model) -> Self {
        Product {
            name: val.name,
            company: val.company,
            variant_groups: serde_json::from_value::<VariantCategoryList>(val.variant_groups)
                .unwrap(),
            variants: serde_json::from_value::<Vec<VariantInformation>>(val.variants).unwrap(),
            sku: val.sku,
            images: serde_json::from_value::<Vec<Url>>(val.images).unwrap(),
            tags: serde_json::from_value::<TagList>(val.tags).unwrap(),
            description: val.description,
            specifications: serde_json::from_value::<Vec<(String, String)>>(val.specifications)
                .unwrap(),
            identification: serde_json::from_value::<ProductIdentification>(val.identification)
                .unwrap(),
            visible: serde_json::from_value::<ProductVisibility>(val.visible).unwrap(),
            created_at: DateTime::from_naive_utc_and_offset(val.created_at, Utc),
            name_long: val.name_long,
            description_long: val.description_long,
            updated_at: DateTime::from_naive_utc_and_offset(val.updated_at, Utc),
        }
    }
}