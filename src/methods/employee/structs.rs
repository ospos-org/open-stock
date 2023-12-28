use std::fmt::{self, Display};

use chrono::{DateTime, Utc};
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

#[derive(Deserialize, Serialize, Clone, JsonSchema, Validate)]
pub struct Auth {
    pub pass: String,
    pub kiosk_id: String,
    pub tenant_id: String,
}

#[cfg(feature = "types")]
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
pub enum AccountType {
    FrontLine,
    Managerial
}

#[derive(Serialize, Deserialize, Clone, JsonSchema, Validate)]
pub struct LogRequest {
    pub kiosk: String,
    pub reason: String,
    pub in_or_out: String,
}

#[cfg(feature = "types")]
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema, Validate)]
pub struct Employee {
    pub id: Id,
    pub rid: String,
    pub name: Name,

    pub auth: EmployeeAuth,
    pub contact: ContactInformation,
    pub clock_history: Vec<History<Attendance>>,

    pub level: Vec<Access<Action>>,
    pub account_type: AccountType,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>
}

#[cfg(feature = "types")]
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema, Validate)]
pub struct Access<T> {
    pub action: T,
    pub authority: i32,
}

use enum_iterator::{all, Sequence};


#[cfg(feature = "types")]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Sequence, JsonSchema)]
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
pub fn all_actions() -> Vec<Access<Action>> {
    all::<Action>().map(| x | Access {
        action: x,
        authority: 1
    }).collect::<Vec<_>>()
}

#[cfg(feature = "types")]
/// Stores a password hash, signed as a key using the users login ID.
/// Upon logging in using a client portal, the pre-sign object is signed using the provided ID -
/// if the hash matches that which is given, authentication can be approved.
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema, Validate)]
pub struct EmployeeAuth {
    pub hash: String,
}

#[cfg(feature = "types")]
#[derive(Serialize, Deserialize, Clone, JsonSchema, Validate)]
pub struct EmployeeInput {
    pub name: Name,
    pub rid: i32,
    pub contact: ContactInformation,
    pub password: String,
    pub clock_history: Vec<History<Attendance>>,
    pub level: Vec<Access<Action>>,
    pub account_type: AccountType
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
use rand::Rng;
use schemars::JsonSchema;
use validator::Validate;

#[cfg(feature = "methods")]
impl Employee {
    pub async fn insert(
        empl: EmployeeInput,
        db: &DbConn,
        session: Session,
        static_rid: Option<i32>,
        static_id: Option<String>,
    ) -> Result<InsertResult<employee::ActiveModel>, DbErr> {
        let id = static_id.map_or(Uuid::new_v4().to_string(), |x| x);
        let mut rid = rand::thread_rng().gen_range(0..9999);

        if static_rid.is_some() {
            rid = static_rid.unwrap();
        }

        let password = empl.password.clone();
        let salt = b"randomsalt";
        let config = Config::original();
        let hash = argon2::hash_encoded(password.as_bytes(), salt, &config).unwrap();

        let insert_crud = empl.into_active(id, rid, session.tenant_id, hash);

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
        let employee = Self::fetch_by_id(id, session, db).await?;

        let is_valid = argon2::verify_encoded(
            &employee.auth.hash, pass.as_bytes()
        ).unwrap();

        Ok(is_valid)
    }

    pub async fn verify_with_rid(
        rid: &str,
        session: Session,
        pass: &str,
        db: &DbConn,
    ) -> Result<Employee, DbErr> {
        let employee = Self::fetch_by_rid(rid, session, db).await?;

        let mut valid_user: Option<Employee> = None;

        for employee in employee {
            println!("Validating employee");

            let is_valid = argon2::verify_encoded(
                &employee.auth.hash, pass.as_bytes()
            ).map_err(|e| DbErr::RecordNotFound(e.to_string()))?;

            println!("Found employee is {}", if is_valid { "Valid" } else { "Invalid" });

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
            Some(e) => Ok(e.into()),
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
            .map(|e| e.clone().into())
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
            .map(|e| e.clone().into())
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
            .map(|e| e.clone().into())
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
            .map(|e| e.clone().into())
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

                // A hand-written conversion is used here,
                // as it is more explicit and less generalisable.
                //
                // When given time, re-write the track for the `update` API
                // to remove this as a limitation as this may become future
                // technical debt.
                employee::ActiveModel {
                    id: Set(id.to_string()),
                    rid: Set(empl.rid),
                    name: Set(json!(empl.name)),
                    auth: Set(json!(empl.auth)),
                    contact: Set(json!(new_contact)),
                    clock_history: Set(json!(empl.clock_history)),
                    level: Set(json!(empl.level)),
                    tenant_id: Set(session.clone().tenant_id),
                    account_type: Set(json!(empl.account_type)),
                    created_at: Set(empl.created_at.naive_utc()),
                    updated_at: Set(empl.updated_at.naive_utc())
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
        let empl = example_employee();
        match Employee::insert(empl.clone(), db, session.clone(), Some(empl.rid), None).await {
            Ok(data) => match Employee::fetch_by_id(&data.last_insert_id, session, db).await {
                Ok(res) => Ok(res),
                Err(e) => Err(e),
            },
            Err(e) => Err(e),
        }
    }
}

#[cfg(feature = "types")]
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema, Validate)]
pub struct Attendance {
    pub track_type: TrackType,
    pub kiosk: Id,
}

#[cfg(feature = "types")]
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
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
        account_type: AccountType::FrontLine,
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
