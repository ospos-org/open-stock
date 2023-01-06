use sea_orm::{DbConn, DbErr, EntityTrait, ColumnTrait, QuerySelect, Set, ActiveModelTrait, InsertResult};
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::entities::prelude::Store as StoreEntity;
use crate::entities::store;
use crate::methods::{MobileNumber, Email, Address};
use crate::{methods::{Id, ContactInformation}};
use serde_json::{json};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Store {
    pub id: Id,
    pub name: String,
    pub contact: ContactInformation,
    pub code: String
}

impl Store {
    pub async fn insert(store: Store, db: &DbConn) -> Result<InsertResult<store::ActiveModel>, DbErr> {
        let insert_crud = store::ActiveModel {
            name: Set(store.name),
            id: Set(store.id),
            contact: Set(json!(store.contact)),
            code: Set(store.code),
        };

        match StoreEntity::insert(insert_crud).exec(db).await {
            Ok(res) => Ok(res),
            Err(err) => Err(err)
        }
    }

    pub async fn fetch_by_id(id: &str, db: &DbConn) -> Result<Store, DbErr> {
        let store = StoreEntity::find_by_id(id.to_string()).one(db).await?;
        
        match store {
            Some(e) => {
                Ok(Store { 
                    id: e.id, 
                    name: e.name,
                    contact: serde_json::from_value::<ContactInformation>(e.contact).unwrap(), 
                    code: serde_json::from_value::<String>(serde_json::Value::String(e.code)).unwrap(), 
                })
            },
            None => {
                Err(DbErr::RecordNotFound(id.to_string()))
            },
        }
    }

    pub async fn fetch_by_code(code: &str, db: &DbConn) -> Result<Store, DbErr> {
        let store = StoreEntity::find().having(store::Column::Code.eq(code)).one(db).await?;
        
        match store {
            Some(e) => {
                Ok(Store { 
                    id: e.id, 
                    name: e.name,
                    contact: serde_json::from_value::<ContactInformation>(e.contact).unwrap(), 
                    code: serde_json::from_value::<String>(serde_json::Value::String(e.code)).unwrap(), 
                })
            },
            None => {
                Err(DbErr::RecordNotFound(code.to_string()))
            },
        }
    }

    pub async fn update(store: Store, id: &str, db: &DbConn) -> Result<Store, DbErr> {
        store::ActiveModel {
            id: Set(id.to_string()),
            name: Set(store.name.clone()),
            code: Set(store.code.clone()),
            contact: Set(json!(store.contact)),
        }.update(db).await?;

        Ok(store)
    }

    pub async fn generate(db: &DbConn) -> Result<Store, DbErr> {
        // Create Transaction
        let store = example_store();
        
        // Insert & Fetch Transaction
        match Store::insert(store, db).await {
            Ok(data) => {
                match Store::fetch_by_id(&data.last_insert_id, db).await {
                    Ok(res) => {
                        Ok(res)
                    },  
                    Err(e) => Err(e)
                }
            },
            Err(e) => Err(e),
        }
    }
}

fn example_store() -> Store {
    Store { 
        id: Uuid::new_v4().to_string(), 
        name: "Carbine".to_string(), 
        contact: ContactInformation {
            name: "Torpedo7".into(),
            mobile: MobileNumber {
                region_code: "+64".into(),
                root: "021212120".into()
            },
            email: Email {
                root: "order".into(),
                domain: "torpedo7.com".into(),
                full: "order@torpedo7.com".into()
            },
            landline: "".into(),
            address: Address {
                street: "9 Carbine Road".into(),
                street2: "".into(),
                city: "Auckland".into(),
                country: "New Zealand".into(),
                po_code: "100".into()
            }
        }, 
        code: "001".to_string() 
    }
}