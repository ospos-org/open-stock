use std::{fmt::{Display, self}};

use argonautica::{Verifier, Hasher};
use chrono::Utc;
use sea_orm::{DbConn, DbErr, Set, EntityTrait, QuerySelect, ColumnTrait, InsertResult, ActiveModelTrait};
use serde::{Serialize, Deserialize};
use serde_json::{json};
use uuid::Uuid;

use argonautica::config::{Backend, Variant, Version};
use futures_cpupool::CpuPool;

use crate::{methods::{Id, Name, ContactInformation, History, MobileNumber, Email, Address}, entities::employee};
use crate::entities::prelude::Employee as Epl;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Employee {
    pub id: Id,
    pub name: Name,
    pub auth: EmployeeAuth,
    pub contact: ContactInformation,
    pub clock_history: Vec<History<Attendance>>,
    pub level: Vec<Access<Action>>
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Access<T> {
    pub action: T,
    pub authority: i32
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Action {
    CreateCustomer, DeleteCustomer, ModifyCustomer, FetchCustomer,
    CreateEmployee, DeleteEmployee, ModifyEmployee, FetchEmployee,
    CreateTransaction, DeleteTransaction, ModifyTransaction, FetchTransaction,
    CreateProduct, DeleteProduct, ModifyProduct, CreateStockAdjustmentIntent, ClearStockAdjustmentIntent, FetchProduct,
    CreateStore, DeleteStore, ModifyStore, FetchStore,
    CreateSupplier, DeleteSupplier, ModifySupplier, FetchSupplier,
    
    AccessAdminPanel, SuperUserDo, GenerateTemplateContent
}

/// Stores a password hash, signed as a key using the users login ID.
/// Upon logging in using a client portal, the pre-sign object is signed using the provided ID - if the hash matches that which is given, authentication can be approved.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EmployeeAuth {
    pub hash: String
}

#[derive(Serialize, Deserialize, Clone)]
pub struct EmployeeInput {
    pub name: Name,
    pub contact: ContactInformation,
    pub password: String,
    pub clock_history: Vec<History<Attendance>>,
    pub level: Vec<Access<Action>>
}

impl Display for Employee {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let clock_history: String = self.clock_history.iter()
            .map(|f| 
                format!(
                    "{}: {} ({})\n", 
                    f.timestamp.format("%d/%m/%Y %H:%M"), 
                    f.item.track_type.to_string(), 
                    f.item.till
                )
            ).collect();

        write!(
            f, 
            "{} {} ({:?})\n{}\n({}) {} {}\n\n[Clock History]\n{}
            ", 
            self.name.first, self.name.last, self.level, 
            self.id, 
            self.contact.mobile.region_code, self.contact.mobile.root, self.contact.email.full,
            clock_history
        )
    }
}

impl Employee {
    pub async fn insert(empl: EmployeeInput, db: &DbConn) -> Result<InsertResult<employee::ActiveModel>, DbErr> {
        
        let id = Uuid::new_v4().to_string();

        let mut hasher = Hasher::default();
        hasher
            .configure_backend(Backend::C) // Default is `Backend::C`
            .configure_cpu_pool(CpuPool::new(8))
            .configure_hash_len(16) // Default is `32`
            .configure_iterations(124) // Default is `192`
            .configure_lanes(8) // Default is number of logical cores on your machine
            .configure_memory_size(4096) // Default is `4096`
            .configure_password_clearing(false) // Default is `false`
            .configure_secret_key_clearing(false) // Default is `false`
            .configure_threads(8) // Default is number of logical cores on your machine
            .configure_variant(Variant::Argon2id) // Default is `Variant::Argon2id`
            .configure_version(Version::_0x13); // Default is `Version::_0x13`

        let hash = hasher
            .with_password(empl.password)
            .with_secret_key("\
                secret key that you should really store in a .env file \
                instead of in code, but this is just an example\
            ")
            .hash()
            .unwrap();

        let insert_crud = employee::ActiveModel {
            id: Set(id),
            name: Set(json!(empl.name)),
            auth: Set(json!(EmployeeAuth {
                hash: hash
            })),
            contact: Set(json!(empl.contact)),
            clock_history: Set(json!(empl.clock_history)),
            level: Set(json!(empl.level)),
        };

        match Epl::insert(insert_crud).exec(db).await {
            Ok(res) => Ok(res),
            Err(err) => Err(err)
        }
    }

    pub async fn verify(id: &str, pass: &str, db: &DbConn) -> Result<bool, DbErr> {
        let empl = Self::fetch_by_id(id, db).await?;

        let mut verifier = Verifier::default();
        let is_valid = verifier
            .with_hash(empl.auth.hash)
            .with_password(pass)
            .with_secret_key("\
                secret key that you should really store in a .env file \
                instead of in code, but this is just an example\
            ")
            .verify()
            .unwrap();

        Ok(is_valid)
    }

    pub async fn fetch_by_id(id: &str, db: &DbConn) -> Result<Employee, DbErr> {
        let empl = Epl::find_by_id(id.to_string()).one(db).await?;
        
        match empl {
            Some(e) => {
                Ok(Employee { 
                    id: e.id, 
                    name: serde_json::from_value::<Name>(e.name).unwrap(),
                    auth: serde_json::from_value::<EmployeeAuth>(e.auth).unwrap(),
                    contact: serde_json::from_value::<ContactInformation>(e.contact).unwrap(), 
                    clock_history: serde_json::from_value::<Vec<History<Attendance>>>(e.clock_history).unwrap(), 
                    level: serde_json::from_value::<Vec<Access<Action>>>(e.level.clone()).unwrap()
                })
            },
            None => {
                Err(DbErr::RecordNotFound(id.to_string()))
            },
        }
    }

    pub async fn fetch_by_name(name: &str, db: &DbConn) -> Result<Vec<Employee>, DbErr> {
        let res = employee::Entity::find()
            .having(employee::Column::Name.contains(name))
            .all(db).await?;
            
        let mapped = res.iter().map(|e| 
            Employee { 
                id: e.id.clone(), 
                name: serde_json::from_value::<Name>(e.name.clone()).unwrap(), 
                auth: serde_json::from_value::<EmployeeAuth>(e.auth.clone()).unwrap(),
                contact: serde_json::from_value::<ContactInformation>(e.contact.clone()).unwrap(), 
                clock_history: serde_json::from_value::<Vec<History<Attendance>>>(e.clock_history.clone()).unwrap(), 
                level: serde_json::from_value::<Vec<Access<Action>>>(e.level.clone()).unwrap()
            }
        ).collect();

        Ok(mapped)
    }

    pub async fn fetch_by_name_exact(name: serde_json::Value, db: &DbConn) -> Result<Vec<Employee>, DbErr> {
        let res = employee::Entity::find()
            .having(employee::Column::Name.eq(name))
            .all(db).await?;
            
        let mapped = res.iter().map(|e| 
            Employee { 
                id: e.id.clone(), 
                name: serde_json::from_value::<Name>(e.name.clone()).unwrap(), 
                auth: serde_json::from_value::<EmployeeAuth>(e.auth.clone()).unwrap(),
                contact: serde_json::from_value::<ContactInformation>(e.contact.clone()).unwrap(), 
                clock_history: serde_json::from_value::<Vec<History<Attendance>>>(e.clock_history.clone()).unwrap(), 
                level: serde_json::from_value::<Vec<Access<Action>>>(e.level.clone()).unwrap()
            }
        ).collect();

        Ok(mapped)
    }

    pub async fn fetch_by_level(level: i32, db: &DbConn) -> Result<Vec<Employee>, DbErr> {
        let res = employee::Entity::find()
            .having(employee::Column::Level.eq(level))
            .all(db).await?;
            
        let mapped = res.iter().map(|e| 
            Employee { 
                id: e.id.clone(), 
                name: serde_json::from_value::<Name>(e.name.clone()).unwrap(), 
                auth: serde_json::from_value::<EmployeeAuth>(e.auth.clone()).unwrap(),
                contact: serde_json::from_value::<ContactInformation>(e.contact.clone()).unwrap(), 
                clock_history: serde_json::from_value::<Vec<History<Attendance>>>(e.clock_history.clone()).unwrap(), 
                level: serde_json::from_value::<Vec<Access<Action>>>(e.level.clone()).unwrap()
            }
        ).collect();

        Ok(mapped)
    }

    pub async fn update(empl: Employee, id: &str, db: &DbConn) -> Result<Employee, DbErr> {
        employee::ActiveModel {
            id: Set(id.to_string()),
            name: Set(json!(empl.name)),
            auth: Set(json!(empl.auth)),
            contact: Set(json!(empl.contact)),
            clock_history: Set(json!(empl.clock_history)),
            level: Set(json!(empl.level)),
        }.update(db).await?;

        Ok(empl)
        // Self::fetch_by_id(id, db).await
    }

    pub async fn generate(db: &DbConn) -> Result<Employee, DbErr> {
        // Create Transaction
        let empl = example_employee();
        
        // Insert & Fetch Transaction
        match Employee::insert(empl, db).await {
            Ok(data) => {
                match Employee::fetch_by_id(&data.last_insert_id, db).await {
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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Attendance {
    pub track_type: TrackType,
    pub till: Id
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum TrackType {
    In, Out
}

impl ToString for TrackType {
    fn to_string(&self) -> String {
        match self {
            TrackType::In => "IN".to_string(),
            TrackType::Out => "OUT".to_string(),
        }
    }
}

pub fn example_employee() -> EmployeeInput {
    EmployeeInput {
        password: "1232".to_string(),
        name: Name {
            first: "Carl".to_string(),
            middle: "".to_string(),
            last: "Kennith".to_string()
        },
        contact: ContactInformation {
            name: "Carl Kennith".into(),
            mobile: MobileNumber::from("021212120".to_string()),
            email: Email::from("carl@kennith.com".to_string()),
            landline: "".into(),
            address: Address {
                street: "9 Carbine Road".into(),
                street2: "".into(),
                city: "Auckland".into(),
                country: "New Zealand".into(),
                po_code: "100".into(),
            },
        },
        clock_history: vec![
            History::<Attendance> { item: Attendance { track_type: TrackType::In, till: "5".to_string() }, reason: "".to_string(), timestamp: Utc::now() },
            History::<Attendance> { item: Attendance { track_type: TrackType::Out, till: "6".to_string() }, reason: "".to_string(), timestamp: Utc::now() },
            History::<Attendance> { item: Attendance { track_type: TrackType::In, till: "1".to_string() }, reason: "".to_string(), timestamp: Utc::now() },
            History::<Attendance> { item: Attendance { track_type: TrackType::Out, till: "3".to_string() }, reason: "".to_string(), timestamp: Utc::now() },
            History::<Attendance> { item: Attendance { track_type: TrackType::In, till: "4".to_string() }, reason: "".to_string(), timestamp: Utc::now() },
            History::<Attendance> { item: Attendance { track_type: TrackType::Out, till: "4".to_string() }, reason: "Left Early".to_string(), timestamp: Utc::now() },
            History::<Attendance> { item: Attendance { track_type: TrackType::In, till: "4".to_string() }, reason: "".to_string(), timestamp: Utc::now() },
            History::<Attendance> { item: Attendance { track_type: TrackType::Out, till: "5".to_string() }, reason: "".to_string(), timestamp: Utc::now() },
        ],
        level: vec![
            Access {
                action: Action::ModifyEmployee,
                authority: 4
            }
        ]
    }
}