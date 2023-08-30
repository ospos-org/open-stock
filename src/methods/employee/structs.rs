use std::fmt::{self, Display};

use chrono::Utc;
#[cfg(feature = "process")]
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbConn, DbErr, EntityTrait, InsertResult, QueryFilter,
    QuerySelect, RuntimeErr, Set,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

#[cfg(feature = "process")]
use crate::entities::employee;
#[cfg(feature = "process")]
use crate::entities::prelude::Employee as Epl;
use crate::methods::{Address, ContactInformation, Email, History, Id, MobileNumber, Name};
use crate::Session;

#[cfg(feature = "process")]
use crate::methods::convert_addr_to_geo;

#[cfg(feature = "types")]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Employee {
    pub id: Id,
    pub rid: String,
    pub name: Name,
    pub auth: EmployeeAuth,
    pub contact: ContactInformation,
    pub clock_history: Vec<History<Attendance>>,
    pub level: Vec<Access<Action>>,
}

impl From<EmployeeInput> for Employee {
    fn from(value: EmployeeInput) -> Self {
        let id = Uuid::new_v4().to_string();
        Employee {
            id,
            rid: value.rid.to_string(),
            name: value.name,
            auth: EmployeeAuth {
                hash: String::new(),
            },
            contact: value.contact,
            clock_history: value.clock_history,
            level: value.level,
        }
    }
}

#[cfg(feature = "types")]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Access<T> {
    pub action: T,
    pub authority: i32,
}

#[cfg(feature = "types")]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Action {
    CreateCustomer,
    DeleteCustomer,
    ModifyCustomer,
    FetchCustomer,

    CreateEmployee,
    DeleteEmployee,
    ModifyEmployee,
    FetchEmployee,

    CreateTransaction,
    DeleteTransaction,
    ModifyTransaction,
    FetchTransaction,

    CreateProduct,
    DeleteProduct,
    ModifyProduct,

    CreateStockAdjustmentIntent,
    ClearStockAdjustmentIntent,

    FetchProduct,
    CreateStore,
    DeleteStore,
    ModifyStore,
    FetchStore,

    CreateSupplier,
    DeleteSupplier,
    ModifySupplier,
    FetchSupplier,

    CreateKiosk,
    DeleteKiosk,
    ModifyKiosk,
    ModifyKioskPreferences,
    FetchKiosk,

    AccessAdminPanel,
    SuperUserDo,
    GenerateTemplateContent,
    FetchGeoLocation,
}

#[cfg(feature = "types")]
/// Stores a password hash, signed as a key using the users login ID.
/// Upon logging in using a client portal, the pre-sign object is signed using the provided ID - if the hash matches that which is given, authentication can be approved.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EmployeeAuth {
    pub hash: String,
}

#[cfg(feature = "types")]
#[derive(Serialize, Deserialize, Clone)]
pub struct EmployeeInput {
    pub name: Name,
    pub rid: i32,
    pub contact: ContactInformation,
    pub password: String,
    pub clock_history: Vec<History<Attendance>>,
    pub level: Vec<Access<Action>>,
}

impl Display for Employee {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let clock_history: String = self
            .clock_history
            .iter()
            .map(|f| {
                format!(
                    "{}: {} ({})\n",
                    f.timestamp.format("%d/%m/%Y %H:%M"),
                    f.item.track_type.to_string(),
                    f.item.kiosk
                )
            })
            .collect();

        write!(
            f,
            "{} {} ({:?})\n{}\n({}) {} {}\n\n[Clock History]\n{}
            ",
            self.name.first,
            self.name.last,
            self.level,
            self.id,
            self.contact.mobile.number,
            if self.contact.mobile.valid {
                "VALID"
            } else {
                "INVALID"
            },
            self.contact.email.full,
            clock_history
        )
    }
}

#[cfg(feature = "process")]
use argon2::{self, Config};

#[cfg(feature = "methods")]
impl Employee {
    pub async fn insert(
        empl: EmployeeInput,
        db: &DbConn,
        session: Session,
        static_rid: Option<i32>,
    ) -> Result<InsertResult<employee::ActiveModel>, DbErr> {
        let id = Uuid::new_v4().to_string();
        let mut rid = alea::i32_in_range(0, 9999);

        if static_rid.is_some() {
            rid = static_rid.unwrap();
        }

        let password = empl.password;
        let salt = b"randomsalt";
        let config = Config::default();
        let hash = argon2::hash_encoded(password.as_bytes(), salt, &config).unwrap();

        let insert_crud = employee::ActiveModel {
            id: Set(id),
            rid: Set(format!("{:0>#4}", rid)),
            name: Set(json!(empl.name)),
            auth: Set(json!(EmployeeAuth { hash })),
            contact: Set(json!(empl.contact)),
            clock_history: Set(json!(empl.clock_history)),
            level: Set(json!(empl.level)),
            tenant_id: Set(session.tenant_id),
        };

        match Epl::insert(insert_crud).exec(db).await {
            Ok(res) => Ok(res),
            Err(err) => Err(err),
        }
    }

    pub async fn verify(
        id: &str,
        session: Session,
        pass: &str,
        db: &DbConn,
    ) -> Result<bool, DbErr> {
        let empl = Self::fetch_by_id(id, session, db).await?;
        let is_valid = argon2::verify_encoded(&empl.auth.hash, pass.as_bytes()).unwrap();

        Ok(is_valid)
    }

    pub async fn verify_with_rid(
        rid: &str,
        session: Session,
        pass: &str,
        db: &DbConn,
    ) -> Result<Employee, DbErr> {
        let empl = Self::fetch_by_rid(rid, session, db).await?;

        let mut valid_user: Option<Employee> = None;

        for employee in empl {
            let is_valid = argon2::verify_encoded(&employee.auth.hash, pass.as_bytes()).unwrap();

            if is_valid && valid_user.is_none() {
                valid_user = Some(employee);
            } else if is_valid {
                println!("User with same rid and password exists. Unsure which one to pass - insufficient information.")
            }
        }

        if valid_user.is_some() {
            Ok(valid_user.unwrap())
        } else {
            Err(DbErr::Query(RuntimeErr::Internal(
                "Unable to locate user. No user exists.".to_string(),
            )))
        }
    }

    pub async fn fetch_by_id(id: &str, session: Session, db: &DbConn) -> Result<Employee, DbErr> {
        let empl = Epl::find_by_id(id.to_string())
            .filter(employee::Column::TenantId.eq(session.tenant_id))
            .one(db)
            .await?;

        match empl {
            Some(e) => Ok(Employee {
                id: e.id,
                rid: e.rid,
                name: serde_json::from_value::<Name>(e.name).unwrap(),
                auth: serde_json::from_value::<EmployeeAuth>(e.auth).unwrap(),
                contact: serde_json::from_value::<ContactInformation>(e.contact).unwrap(),
                clock_history: serde_json::from_value::<Vec<History<Attendance>>>(e.clock_history)
                    .unwrap(),
                level: serde_json::from_value::<Vec<Access<Action>>>(e.level).unwrap(),
            }),
            None => Err(DbErr::RecordNotFound(id.to_string())),
        }
    }

    pub async fn fetch_by_rid(
        rid: &str,
        session: Session,
        db: &DbConn,
    ) -> Result<Vec<Employee>, DbErr> {
        let res = employee::Entity::find()
            .having(employee::Column::Rid.contains(rid))
            .filter(employee::Column::TenantId.eq(session.tenant_id))
            .limit(25)
            .all(db)
            .await?;

        let mapped = res
            .iter()
            .map(|e| Employee {
                id: e.id.clone(),
                rid: e.rid.clone(),
                name: serde_json::from_value::<Name>(e.name.clone()).unwrap(),
                auth: serde_json::from_value::<EmployeeAuth>(e.auth.clone()).unwrap(),
                contact: serde_json::from_value::<ContactInformation>(e.contact.clone()).unwrap(),
                clock_history: serde_json::from_value::<Vec<History<Attendance>>>(
                    e.clock_history.clone(),
                )
                .unwrap(),
                level: serde_json::from_value::<Vec<Access<Action>>>(e.level.clone()).unwrap(),
            })
            .collect();

        Ok(mapped)
    }

    pub async fn fetch_by_name(
        name: &str,
        session: Session,
        db: &DbConn,
    ) -> Result<Vec<Employee>, DbErr> {
        let res = employee::Entity::find()
            .having(employee::Column::Name.contains(name))
            .filter(employee::Column::TenantId.eq(session.tenant_id))
            .limit(25)
            .all(db)
            .await?;

        let mapped = res
            .iter()
            .map(|e| Employee {
                id: e.id.clone(),
                rid: e.rid.clone(),
                name: serde_json::from_value::<Name>(e.name.clone()).unwrap(),
                auth: serde_json::from_value::<EmployeeAuth>(e.auth.clone()).unwrap(),
                contact: serde_json::from_value::<ContactInformation>(e.contact.clone()).unwrap(),
                clock_history: serde_json::from_value::<Vec<History<Attendance>>>(
                    e.clock_history.clone(),
                )
                .unwrap(),
                level: serde_json::from_value::<Vec<Access<Action>>>(e.level.clone()).unwrap(),
            })
            .collect();

        Ok(mapped)
    }

    pub async fn fetch_by_name_exact(
        name: serde_json::Value,
        session: Session,
        db: &DbConn,
    ) -> Result<Vec<Employee>, DbErr> {
        let res = employee::Entity::find()
            .having(employee::Column::Name.eq(name))
            .filter(employee::Column::TenantId.eq(session.tenant_id))
            .limit(25)
            .all(db)
            .await?;

        let mapped = res
            .iter()
            .map(|e| Employee {
                id: e.id.clone(),
                rid: e.rid.clone(),
                name: serde_json::from_value::<Name>(e.name.clone()).unwrap(),
                auth: serde_json::from_value::<EmployeeAuth>(e.auth.clone()).unwrap(),
                contact: serde_json::from_value::<ContactInformation>(e.contact.clone()).unwrap(),
                clock_history: serde_json::from_value::<Vec<History<Attendance>>>(
                    e.clock_history.clone(),
                )
                .unwrap(),
                level: serde_json::from_value::<Vec<Access<Action>>>(e.level.clone()).unwrap(),
            })
            .collect();

        Ok(mapped)
    }

    pub async fn fetch_by_level(
        level: i32,
        session: Session,
        db: &DbConn,
    ) -> Result<Vec<Employee>, DbErr> {
        let res = employee::Entity::find()
            .having(employee::Column::Level.eq(level))
            .filter(employee::Column::TenantId.eq(session.tenant_id))
            .limit(25)
            .all(db)
            .await?;

        let mapped = res
            .iter()
            .map(|e| Employee {
                id: e.id.clone(),
                rid: e.rid.clone(),
                name: serde_json::from_value::<Name>(e.name.clone()).unwrap(),
                auth: serde_json::from_value::<EmployeeAuth>(e.auth.clone()).unwrap(),
                contact: serde_json::from_value::<ContactInformation>(e.contact.clone()).unwrap(),
                clock_history: serde_json::from_value::<Vec<History<Attendance>>>(
                    e.clock_history.clone(),
                )
                .unwrap(),
                level: serde_json::from_value::<Vec<Access<Action>>>(e.level.clone()).unwrap(),
            })
            .collect();

        Ok(mapped)
    }

    pub async fn update_no_geom(
        empl: Employee,
        session: Session,
        id: &str,
        db: &DbConn,
    ) -> Result<Employee, DbErr> {
        employee::ActiveModel {
            id: Set(id.to_string()),
            rid: Set(empl.rid),
            name: Set(json!(empl.name)),
            auth: Set(json!(empl.auth)),
            clock_history: Set(json!(empl.clock_history)),
            level: Set(json!(empl.level)),
            ..Default::default()
        }
        .update(db)
        .await?;

        Self::fetch_by_id(id, session, db).await
    }

    pub async fn update(
        empl: Employee,
        session: Session,
        id: &str,
        db: &DbConn,
    ) -> Result<Employee, DbErr> {
        let addr = convert_addr_to_geo(&format!(
            "{} {} {} {}",
            empl.contact.address.street,
            empl.contact.address.street2,
            empl.contact.address.po_code,
            empl.contact.address.city
        ));

        match addr {
            Ok(ad) => {
                let mut new_contact = empl.contact;
                new_contact.address = ad;

                employee::ActiveModel {
                    id: Set(id.to_string()),
                    rid: Set(empl.rid),
                    name: Set(json!(empl.name)),
                    auth: Set(json!(empl.auth)),
                    contact: Set(json!(new_contact)),
                    clock_history: Set(json!(empl.clock_history)),
                    level: Set(json!(empl.level)),
                    tenant_id: Set(session.clone().tenant_id),
                }
                .update(db)
                .await?;

                Self::fetch_by_id(id, session, db).await
            }
            Err(_) => Err(DbErr::Query(RuntimeErr::Internal(
                "Invalid address format".to_string(),
            ))),
        }
    }

    pub async fn generate(db: &DbConn, session: Session) -> Result<Employee, DbErr> {
        // Create Transaction
        let empl = example_employee();

        // Insert & Fetch Transaction
        match Employee::insert(empl.clone(), db, session.clone(), Some(empl.rid)).await {
            Ok(data) => match Employee::fetch_by_id(&data.last_insert_id, session, db).await {
                Ok(res) => Ok(res),
                Err(e) => Err(e),
            },
            Err(e) => Err(e),
        }
    }
}

#[cfg(feature = "types")]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Attendance {
    pub track_type: TrackType,
    pub kiosk: Id,
}

#[cfg(feature = "types")]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum TrackType {
    In,
    Out,
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
        rid: 1232,
        name: Name {
            first: "Carl".to_string(),
            middle: "".to_string(),
            last: "Kennith".to_string(),
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
                lat: -36.915500,
                lon: 174.838740,
            },
        },
        clock_history: vec![
            History::<Attendance> {
                item: Attendance {
                    track_type: TrackType::In,
                    kiosk: "5".to_string(),
                },
                reason: "".to_string(),
                timestamp: Utc::now(),
            },
            History::<Attendance> {
                item: Attendance {
                    track_type: TrackType::Out,
                    kiosk: "6".to_string(),
                },
                reason: "".to_string(),
                timestamp: Utc::now(),
            },
            History::<Attendance> {
                item: Attendance {
                    track_type: TrackType::In,
                    kiosk: "1".to_string(),
                },
                reason: "".to_string(),
                timestamp: Utc::now(),
            },
            History::<Attendance> {
                item: Attendance {
                    track_type: TrackType::Out,
                    kiosk: "3".to_string(),
                },
                reason: "".to_string(),
                timestamp: Utc::now(),
            },
            History::<Attendance> {
                item: Attendance {
                    track_type: TrackType::In,
                    kiosk: "4".to_string(),
                },
                reason: "".to_string(),
                timestamp: Utc::now(),
            },
            History::<Attendance> {
                item: Attendance {
                    track_type: TrackType::Out,
                    kiosk: "4".to_string(),
                },
                reason: "Left Early".to_string(),
                timestamp: Utc::now(),
            },
            History::<Attendance> {
                item: Attendance {
                    track_type: TrackType::In,
                    kiosk: "4".to_string(),
                },
                reason: "".to_string(),
                timestamp: Utc::now(),
            },
            History::<Attendance> {
                item: Attendance {
                    track_type: TrackType::Out,
                    kiosk: "5".to_string(),
                },
                reason: "".to_string(),
                timestamp: Utc::now(),
            },
        ],
        level: vec![
            Access {
                action: Action::FetchProduct,
                authority: 1,
            },
            Access {
                action: Action::FetchCustomer,
                authority: 1,
            },
            Access {
                action: Action::FetchEmployee,
                authority: 1,
            },
            Access {
                action: Action::FetchTransaction,
                authority: 1,
            },
            Access {
                action: Action::FetchStore,
                authority: 1,
            },
            Access {
                action: Action::ModifyProduct,
                authority: 1,
            },
            Access {
                action: Action::ModifyCustomer,
                authority: 1,
            },
            Access {
                action: Action::ModifyEmployee,
                authority: 0,
            },
            Access {
                action: Action::DeleteTransaction,
                authority: 1,
            },
            Access {
                action: Action::ModifyTransaction,
                authority: 1,
            },
            Access {
                action: Action::ModifyStore,
                authority: 1,
            },
            Access {
                action: Action::CreateProduct,
                authority: 1,
            },
            Access {
                action: Action::CreateCustomer,
                authority: 1,
            },
            Access {
                action: Action::CreateEmployee,
                authority: 0,
            },
            Access {
                action: Action::CreateTransaction,
                authority: 1,
            },
            Access {
                action: Action::CreateStore,
                authority: 0,
            },
            Access {
                action: Action::FetchGeoLocation,
                authority: 1,
            },
        ],
    }
}
