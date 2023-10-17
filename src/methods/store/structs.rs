use chrono::{DateTime, Utc};
#[cfg(feature = "process")]
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbConn, DbErr, EntityTrait, InsertResult, QueryFilter,
    QuerySelect, RuntimeErr,
};
use sea_orm::Set;
use serde::{Deserialize, Serialize};

#[cfg(feature = "process")]
use crate::entities::prelude::Store as StoreEntity;
#[cfg(feature = "process")]
use crate::entities::store;
use crate::methods::{Address, Email, MobileNumber};

#[cfg(feature = "process")]
use crate::methods::convert_addr_to_geo;

use crate::methods::{ContactInformation, Id};
use crate::Session;
use serde_json::json;
use crate::methods::store::example::example_stores;

#[cfg(feature = "types")]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Store {
    pub id: Id,
    pub name: String,

    pub contact: ContactInformation,
    pub code: String,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>
}

#[cfg(feature = "methods")]
impl Store {
    pub async fn insert(
        store: Store,
        session: Session,
        db: &DbConn,
    ) -> Result<InsertResult<store::ActiveModel>, DbErr> {
        let insert_crud = store.into_active(session);

        match StoreEntity::insert(insert_crud).exec(db).await {
            Ok(res) => Ok(res),
            Err(err) => Err(err),
        }
    }

    pub async fn insert_many(
        stores: Vec<Store>,
        session: Session,
        db: &DbConn,
    ) -> Result<InsertResult<store::ActiveModel>, DbErr> {
        let entities = stores
            .into_iter()
            .map(|s|
                s.into_active(session.clone())
            );

        match StoreEntity::insert_many(entities).exec(db).await {
            Ok(res) => Ok(res),
            Err(err) => Err(err),
        }
    }

    pub async fn fetch_by_id(id: &str, session: Session, db: &DbConn) -> Result<Store, DbErr> {
        let store = StoreEntity::find_by_id(id.to_string())
            .filter(store::Column::TenantId.eq(session.tenant_id))
            .one(db)
            .await?;

        match store {
            Some(e) => Ok(e.into()),
            None => Err(DbErr::RecordNotFound(id.to_string())),
        }
    }

    pub async fn fetch_by_code(code: &str, session: Session, db: &DbConn) -> Result<Store, DbErr> {
        let store = StoreEntity::find()
            .filter(store::Column::TenantId.eq(session.tenant_id))
            .having(store::Column::Code.eq(code))
            .one(db)
            .await?;

        match store {
            Some(e) => Ok(e.into()),
            None => Err(DbErr::RecordNotFound(code.to_string())),
        }
    }

    pub async fn fetch_all(session: Session, db: &DbConn) -> Result<Vec<Store>, DbErr> {
        let stores = StoreEntity::find()
            .filter(store::Column::TenantId.eq(session.tenant_id))
            .all(db)
            .await?;

        let mapped = stores
            .iter()
            .map(|e| e.clone().into())
            .collect();

        Ok(mapped)
    }

    pub async fn update(
        store: Store,
        session: Session,
        id: &str,
        db: &DbConn,
    ) -> Result<Store, DbErr> {
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

                model
                    .update(db)
                    .await?;

                Self::fetch_by_id(id, session, db).await
            }
            Err(_) => Err(DbErr::Query(RuntimeErr::Internal(
                "Invalid address format".to_string(),
            ))),
        }
    }

    pub async fn generate(session: Session, db: &DbConn) -> Result<Vec<Store>, DbErr> {
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
