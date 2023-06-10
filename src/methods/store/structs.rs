use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbConn, DbErr, EntityTrait, InsertResult, QuerySelect,
    RuntimeErr, Set,
};
use serde::{Deserialize, Serialize};

use crate::entities::prelude::Store as StoreEntity;
use crate::entities::store;
use crate::methods::{convert_addr_to_geo, Address, Email, MobileNumber};
use crate::methods::{ContactInformation, Id};
use serde_json::json;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Store {
    pub id: Id,
    pub name: String,
    pub contact: ContactInformation,
    pub code: String,
}

impl Store {
    pub async fn insert(
        store: Store,
        db: &DbConn,
    ) -> Result<InsertResult<store::ActiveModel>, DbErr> {
        let insert_crud = store::ActiveModel {
            name: Set(store.name),
            id: Set(store.id),
            contact: Set(json!(store.contact)),
            code: Set(store.code),
        };

        match StoreEntity::insert(insert_crud).exec(db).await {
            Ok(res) => Ok(res),
            Err(err) => Err(err),
        }
    }

    pub async fn insert_many(
        stores: Vec<Store>,
        db: &DbConn,
    ) -> Result<InsertResult<store::ActiveModel>, DbErr> {
        let entities = stores.into_iter().map(|s| store::ActiveModel {
            name: Set(s.name),
            id: Set(s.id),
            contact: Set(json!(s.contact)),
            code: Set(s.code),
        });

        match StoreEntity::insert_many(entities).exec(db).await {
            Ok(res) => Ok(res),
            Err(err) => Err(err),
        }
    }

    pub async fn fetch_by_id(id: &str, db: &DbConn) -> Result<Store, DbErr> {
        let store = StoreEntity::find_by_id(id.to_string()).one(db).await?;

        match store {
            Some(e) => Ok(Store {
                id: e.id,
                name: e.name,
                contact: serde_json::from_value::<ContactInformation>(e.contact).unwrap(),
                code: serde_json::from_value::<String>(serde_json::Value::String(e.code)).unwrap(),
            }),
            None => Err(DbErr::RecordNotFound(id.to_string())),
        }
    }

    pub async fn fetch_by_code(code: &str, db: &DbConn) -> Result<Store, DbErr> {
        let store = StoreEntity::find()
            .having(store::Column::Code.eq(code))
            .one(db)
            .await?;

        match store {
            Some(e) => Ok(Store {
                id: e.id,
                name: e.name,
                contact: serde_json::from_value::<ContactInformation>(e.contact).unwrap(),
                code: serde_json::from_value::<String>(serde_json::Value::String(e.code)).unwrap(),
            }),
            None => Err(DbErr::RecordNotFound(code.to_string())),
        }
    }

    pub async fn fetch_all(db: &DbConn) -> Result<Vec<Store>, DbErr> {
        let stores = StoreEntity::find().all(db).await?;

        let mapped = stores
            .iter()
            .map(|e| Store {
                id: e.id.clone(),
                name: e.name.clone(),
                contact: serde_json::from_value::<ContactInformation>(e.contact.clone()).unwrap(),
                code: serde_json::from_value::<String>(serde_json::Value::String(e.code.clone()))
                    .unwrap(),
            })
            .collect();

        Ok(mapped)
    }

    pub async fn update(store: Store, id: &str, db: &DbConn) -> Result<Store, DbErr> {
        let addr = convert_addr_to_geo(&format!(
            "{} {} {} {}",
            store.contact.address.street,
            store.contact.address.street2,
            store.contact.address.po_code,
            store.contact.address.city
        ));

        match addr {
            Ok(ad) => {
                let mut new_contact = store.contact;
                new_contact.address = ad;

                store::ActiveModel {
                    id: Set(id.to_string()),
                    name: Set(store.name.clone()),
                    code: Set(store.code.clone()),
                    contact: Set(json!(new_contact)),
                }
                .update(db)
                .await?;

                Self::fetch_by_id(id, db).await
            }
            Err(_) => Err(DbErr::Query(RuntimeErr::Internal(
                "Invalid address format".to_string(),
            ))),
        }
    }

    pub async fn generate(db: &DbConn) -> Result<Vec<Store>, DbErr> {
        // Create Transaction
        let stores = example_stores();

        match Store::insert_many(stores, db).await {
            Ok(_) => match Store::fetch_all(db).await {
                Ok(res) => Ok(res),
                Err(e) => Err(e),
            },
            Err(e) => Err(e),
        }
    }
}

fn example_stores() -> Vec<Store> {
    vec![
        Store {
            id: "628f74d7-de00-4956-a5b6-2031e0c72128".to_string(),
            name: "Mt Wellington".to_string(),
            contact: ContactInformation {
                name: "Torpedo7 Mt Wellington".into(),
                mobile: MobileNumber {
                    number: "+6421212120".into(),
                    valid: true,
                },
                email: Email {
                    root: "order".into(),
                    domain: "torpedo7.com".into(),
                    full: "order@torpedo7.com".into(),
                },
                landline: "".into(),
                address: Address {
                    street: "315-375 Mount Wellington Highway".into(),
                    street2: "Mount Wellington".into(),
                    city: "Auckland".into(),
                    country: "New Zealand".into(),
                    po_code: "1060".into(),
                    lat: -36.915501,
                    lon: 174.838745,
                },
            },
            code: "001".to_string(),
        },
        Store {
            id: "c4a1d88b-e8a0-4dcd-ade2-1eea82254816".to_string(),
            name: "Westfield".to_string(),
            contact: ContactInformation {
                name: "Torpedo7 Westfield".into(),
                mobile: MobileNumber {
                    number: "+6421212120".into(),
                    valid: true,
                },
                email: Email {
                    root: "order".into(),
                    domain: "torpedo7.com".into(),
                    full: "order@torpedo7.com".into(),
                },
                landline: "".into(),
                address: Address {
                    street: "309 Broadway, Westfield Shopping Centre".into(),
                    street2: "Newmarket".into(),
                    city: "Auckland".into(),
                    country: "New Zealand".into(),
                    po_code: "1023".into(),
                    lat: -36.871820,
                    lon: 174.776730,
                },
            },
            code: "002".to_string(),
        },
        Store {
            id: "a91509fa-2783-43ae-8c3c-5d5bc5cb6c95".to_string(),
            name: "Albany".to_string(),
            contact: ContactInformation {
                name: "Torpedo7 Albany".into(),
                mobile: MobileNumber {
                    number: "+6421212120".into(),
                    valid: true,
                },
                email: Email {
                    root: "order".into(),
                    domain: "torpedo7.com".into(),
                    full: "order@torpedo7.com".into(),
                },
                landline: "".into(),
                address: Address {
                    street: "6 Mercari Way".into(),
                    street2: "Albany".into(),
                    city: "Auckland".into(),
                    country: "New Zealand".into(),
                    po_code: "0632".into(),
                    lat: -36.7323515,
                    lon: 174.7082982,
                },
            },
            code: "003".to_string(),
        },
    ]
}
