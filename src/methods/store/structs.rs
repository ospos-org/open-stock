use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use sea_orm::Set;
#[cfg(feature = "process")]
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbConn, DbErr, EntityTrait, InsertResult, QueryFilter,
    QuerySelect, RuntimeErr,
};
use serde::{Deserialize, Serialize};

#[cfg(feature = "process")]
use crate::entities::prelude::Store as StoreEntity;
#[cfg(feature = "process")]
use crate::entities::store;

#[cfg(feature = "process")]
use crate::methods::convert_addr_to_geo;

use crate::methods::store::example::example_stores;
use crate::methods::{ContactInformation, Id};
use crate::{methods::Error, Session};
use serde_json::json;
use validator::Validate;

#[cfg(feature = "types")]
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema, Validate)]
pub struct Store {
    pub id: Id,
    pub name: String,

    pub contact: ContactInformation,
    pub code: String,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[cfg(feature = "methods")]
impl Store {
    pub async fn insert(
        store: Store,
        session: Session,
        db: &DbConn,
    ) -> Result<InsertResult<store::ActiveModel>, Error> {
        let insert_crud = store.into_active(session);
        StoreEntity::insert(insert_crud)
            .exec(db)
            .await
            .map_err(|e| e.into())
    }

    pub async fn insert_many(
        stores: Vec<Store>,
        session: Session,
        db: &DbConn,
    ) -> Result<InsertResult<store::ActiveModel>, Error> {
        let entities = stores.into_iter().map(|s| s.into_active(session.clone()));

        StoreEntity::insert_many(entities)
            .exec(db)
            .await
            .map_err(|e| e.into())
    }

    pub async fn fetch_by_id(id: &str, session: Session, db: &DbConn) -> Result<Store, Error> {
        let store = StoreEntity::find_by_id(id.to_string())
            .filter(store::Column::TenantId.eq(session.tenant_id))
            .one(db)
            .await?;

        match store {
            Some(e) => Ok(e.into()),
            None => Err(DbErr::RecordNotFound(id.to_string()).into()),
        }
    }

    pub async fn fetch_by_code(code: &str, session: Session, db: &DbConn) -> Result<Store, Error> {
        let store = StoreEntity::find()
            .filter(store::Column::TenantId.eq(session.tenant_id))
            .having(store::Column::Code.eq(code))
            .one(db)
            .await?;

        match store {
            Some(e) => Ok(e.into()),
            None => Err(DbErr::RecordNotFound(code.to_string()).into()),
        }
    }

    pub async fn fetch_all(session: Session, db: &DbConn) -> Result<Vec<Store>, Error> {
        let stores = StoreEntity::find()
            .filter(store::Column::TenantId.eq(session.tenant_id))
            .all(db)
            .await?;

        let mapped = stores.iter().map(|e| e.clone().into()).collect();

        Ok(mapped)
    }

    pub async fn update(
        store: Store,
        session: Session,
        id: &str,
        db: &DbConn,
    ) -> Result<Store, Error> {
        let addr = convert_addr_to_geo(&format!(
            "{} {} {} {}",
            store.contact.address.street,
            store.contact.address.street2,
            store.contact.address.po_code,
            store.contact.address.city
        ));

        match addr {
            Ok(ad) => {
                let mut new_contact = store.contact.clone();
                new_contact.address = ad;

                let mut model = store.into_active(session.clone());

                model.contact = Set(json!(new_contact));

                model.update(db).await?;

                Self::fetch_by_id(id, session, db).await
            }
            Err(_) => {
                Err(DbErr::Query(RuntimeErr::Internal("Invalid address format".to_string())).into())
            }
        }
    }

    pub async fn generate(session: Session, db: &DbConn) -> Result<Vec<Store>, Error> {
        let stores = example_stores();

        match Store::insert_many(stores, session.clone(), db).await {
            Ok(_) => match Store::fetch_all(session, db).await {
                Ok(res) => Ok(res),
                Err(e) => Err(e),
            },
            Err(e) => Err(e),
        }
    }
}
