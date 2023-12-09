use std::fmt::Display;
use chrono::{DateTime, Utc};
use schemars::JsonSchema;

#[cfg(feature = "process")]
use crate::entities::customer;
#[cfg(feature = "process")]
use crate::entities::prelude::Customer as Cust;
#[cfg(feature = "process")]
use crate::methods::convert_addr_to_geo;
use crate::methods::{Address, ContactInformation, Email, Id, MobileNumber, NoteList};
#[cfg(feature = "process")]
use sea_orm::QueryFilter;
#[cfg(feature = "process")]
use sea_orm::{
    sea_query::{Expr, Func},
    ActiveModelTrait, ColumnTrait, DbBackend, DbConn, DbErr, EntityTrait, FromQueryResult,
    InsertResult, JsonValue, QuerySelect, RuntimeErr, Set, Statement,
};
use sea_orm::QueryOrder;
use serde::{Deserialize, Serialize};
use serde_json::json;
use crate::entities::customer::ActiveModel;
use crate::Session;

#[cfg(feature = "types")]
#[derive(Serialize, Deserialize, Clone, JsonSchema)]
pub struct Customer {
    pub id: Id,
    pub name: String,
    pub contact: ContactInformation,

    pub customer_notes: NoteList,
    pub balance: u32,

    pub special_pricing: String,
    pub accepts_marketing: bool,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>
}

#[cfg(feature = "process")]
#[derive(Serialize, Deserialize, Clone, FromQueryResult, JsonSchema)]
pub struct CustomerWithTransactions {
    pub id: Id,
    pub name: String,

    pub contact: JsonValue,
    pub customer_notes: JsonValue,

    pub balance: u32,
    pub special_pricing: JsonValue,

    pub transactions: Option<String>,
    pub accepts_marketing: bool,
}

#[cfg(feature = "types")]
#[derive(Serialize, Deserialize, Clone, JsonSchema)]
pub struct CustomerWithTransactionsOut {
    pub id: Id,
    pub name: String,

    pub contact: ContactInformation,
    pub customer_notes: NoteList,

    pub balance: u32,
    pub special_pricing: String,

    pub transactions: Option<String>,
    pub accepts_marketing: bool,
}

#[cfg(feature = "types")]
#[derive(Serialize, Deserialize, Clone, JsonSchema)]
pub struct CustomerInput {
    pub name: String,

    pub contact: ContactInformation,
    pub customer_notes: NoteList,

    pub special_pricing: String,
    pub balance: u32,

    pub accepts_marketing: bool,
}

#[cfg(feature = "methods")]
impl Customer {
    pub async fn insert(
        cust: CustomerInput,
        session: Session,
        db: &DbConn,
    ) -> Result<InsertResult<ActiveModel>, DbErr> {
        let insert_crud = cust.into_active(session.tenant_id);

        match Cust::insert(insert_crud).exec(db).await {
            Ok(res) => Ok(res),
            Err(err) => Err(err),
        }
    }

    pub async fn insert_raw(
        cust: Customer,
        session: Session,
        db: &DbConn,
    ) -> Result<InsertResult<ActiveModel>, DbErr> {
        let insert_crud = cust.into_active(session.tenant_id);

        match Cust::insert(insert_crud).exec(db).await {
            Ok(res) => Ok(res),
            Err(err) => Err(err),
        }
    }

    pub async fn fetch_by_id(id: &str, session: Session, db: &DbConn) -> Result<Customer, DbErr> {
        let cust = Cust::find_by_id(id.to_string())
            .filter(customer::Column::TenantId.eq(session.tenant_id))
            .one(db)
            .await?;

        match cust {
            Some(c) => Ok(c.into()),
            None => Err(DbErr::RecordNotFound(
                "Unable to find customer record value".to_string(),
            )),
        }
    }

    pub async fn search(
        query: &str,
        session: Session,
        db: &DbConn,
    ) -> Result<Vec<CustomerWithTransactionsOut>, DbErr> {
        let as_str: Vec<CustomerWithTransactions> = CustomerWithTransactions::find_by_statement(Statement::from_sql_and_values(
                DbBackend::MySql,
                &format!("SELECT Customer.*, GROUP_CONCAT(`Transactions`.`id`) as transactions
                FROM Customer
                LEFT JOIN Transactions ON (REPLACE(JSON_EXTRACT(Transactions.customer, '$.customer_id'), '\"', '')) = Customer.id
                WHERE (LOWER(Customer.name) LIKE '%{}%' OR Customer.contact LIKE '%{}%') AND Customer.tenant_id = '{}'
                GROUP BY Customer.id
                LIMIT 25",
                query, query, session.tenant_id),
                vec![]
            ))
            .all(db)
            .await?;

        let mapped = as_str
            .iter()
            .map(|c| CustomerWithTransactionsOut {
                id: c.id.clone(),
                name: c.name.clone(),
                contact: serde_json::from_value::<ContactInformation>(c.contact.clone()).unwrap(),
                customer_notes: serde_json::from_value::<NoteList>(c.customer_notes.clone())
                    .unwrap(),
                special_pricing: serde_json::from_value::<String>(c.special_pricing.clone())
                    .unwrap(),
                balance: c.balance,
                transactions: c.transactions.clone(),
                accepts_marketing: c.accepts_marketing,
            })
            .collect();

        Ok(mapped)
    }

    pub async fn fetch_by_name(
        name: &str,
        session: Session,
        db: &DbConn,
    ) -> Result<Vec<Customer>, DbErr> {
        let res = customer::Entity::find()
            .filter(customer::Column::TenantId.eq(session.tenant_id))
            .having(
                Expr::expr(Func::lower(Expr::col(customer::Column::Name)))
                    .like(format!("%{}%", name)),
            )
            .limit(25)
            .all(db)
            .await?;

        let mapped: Vec<Customer> = res
            .iter()
            .map(|c| c.into())
            .collect();

        Ok(mapped)
    }

    pub async fn fetch_by_phone(
        phone: &str,
        session: Session,
        db: &DbConn,
    ) -> Result<Vec<Customer>, DbErr> {
        let res = customer::Entity::find()
            .filter(customer::Column::TenantId.eq(session.tenant_id))
            .having(customer::Column::Contact.contains(phone))
            .limit(25)
            .all(db)
            .await?;

        let mapped: Vec<Customer> = res
            .iter()
            .map(|c| c.into())
            .collect();

        Ok(mapped)
    }

    pub async fn fetch_by_addr(
        addr: &str,
        session: Session,
        db: &DbConn,
    ) -> Result<Vec<Customer>, DbErr> {
        let res = customer::Entity::find()
            .filter(customer::Column::TenantId.eq(session.tenant_id))
            .having(customer::Column::Contact.contains(addr))
            .limit(25)
            .all(db)
            .await?;

        let mapped: Vec<Customer> = res
            .iter()
            .map(|c| c.into())
            .collect();

        Ok(mapped)
    }

    pub async fn fetch_recent(
        session: Session,
        db: &DbConn,
    ) -> Result<Vec<Customer>, DbErr> {
        let res = customer::Entity::find()
            .filter(customer::Column::TenantId.eq(session.tenant_id))
            .order_by_desc(customer::Column::UpdatedAt)
            .limit(25)
            .all(db)
            .await?;

        let mapped: Vec<Customer> = res
            .iter()
            .map(|c| c.into())
            .collect();

        Ok(mapped)
    }

    /// Generate and insert a default customer.
    pub async fn generate(session: Session, db: &DbConn) -> Result<Customer, DbErr> {
        let cust = example_customer();
        // Insert & Fetch Customer
        let r = Customer::insert(cust, session.clone(), db).await.unwrap();
        match Customer::fetch_by_id(&r.last_insert_id, session, db).await {
            Ok(cust) => Ok(cust),
            Err(e) => Err(e),
        }
    }

    pub async fn update(
        cust: CustomerInput,
        session: Session,
        id: &str,
        db: &DbConn,
    ) -> Result<Customer, DbErr> {
        let addr = convert_addr_to_geo(&format!(
            "{} {} {} {}",
            cust.contact.address.street,
            cust.contact.address.street2,
            cust.contact.address.po_code,
            cust.contact.address.city
        ));

        // !impl Validate form input/ contact information.

        match addr {
            Ok(ad) => {
                let mut model = cust.clone().into_active(id.to_string());

                let mut new_contact = cust.contact.clone();
                new_contact.address = ad;

                model.contact = Set(json!(new_contact));

                model.update(db)
                    .await?;

                Self::fetch_by_id(id, session, db).await
            }
            Err(_) => Err(DbErr::Query(RuntimeErr::Internal(
                "Invalid address format".to_string(),
            ))),
        }
    }

    pub async fn update_contact_information(
        contact: ContactInformation,
        id: &str,
        session: Session,
        db: &DbConn,
    ) -> Result<Customer, DbErr> {
        let cust = Self::fetch_by_id(id, session.clone(), db).await?;
        // Get geo location for new contact information...
        let addr = convert_addr_to_geo(&format!(
            "{} {} {} {}",
            contact.address.street,
            contact.address.street2,
            contact.address.po_code,
            contact.address.city
        ));

        match addr {
            Ok(ad) => {
                let mut model = cust.clone().into_active(id.to_string());

                let mut new_contact = cust.contact.clone();
                new_contact.address = ad;

                model.contact = Set(json!(new_contact));

                model.update(db)
                    .await?;

                Self::fetch_by_id(id, session, db).await
            }
            Err(_) => Err(DbErr::Query(RuntimeErr::Internal(
                "Invalid address format".to_string(),
            ))),
        }
    }
}

impl Display for Customer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let customer_notes: String = self
            .customer_notes
            .iter()
            .map(|f| format!("{}: {}\n", f.timestamp.format("%d/%m/%Y %H:%M"), f.message,))
            .collect();

        write!(
            f,
            "{} (${})\n{}\n({}) {} {}\n\n[Notes]\n{}
            ",
            self.name,
            self.balance,
            self.id,
            self.contact.mobile.number,
            if self.contact.mobile.valid {
                "VALID"
            } else {
                "INVALID"
            },
            self.contact.email.full,
            customer_notes
        )
    }
}

pub fn example_customer() -> CustomerInput {
    let customer = ContactInformation {
        name: "Carl Kennith".into(),
        mobile: MobileNumber::from("0212121204".to_string()),
        email: Email::from("carl@kennith.com".to_string()),
        landline: "".into(),
        address: Address {
            street: "54 Arney Crescent".into(),
            street2: "Remuera".into(),
            city: "Auckland".into(),
            country: "New Zealand".into(),
            po_code: "1050".into(),
            lat: -36.869870,
            lon: 174.790520,
        },
    };

    CustomerInput {
        name: "Carl Kennith".into(),
        contact: customer,
        special_pricing: "".into(),
        customer_notes: vec![],
        balance: 0,
        accepts_marketing: true,
    }
}
