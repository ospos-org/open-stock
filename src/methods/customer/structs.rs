use std::fmt::Display;
use chrono::{DateTime, Utc};
use schemars::JsonSchema;

#[cfg(feature = "process")]
use crate::entities::customer;
#[cfg(feature = "process")]
use crate::entities::prelude::Customer as Cust;
#[cfg(feature = "process")]
use crate::methods::convert_addr_to_geo;
use crate::methods::{Address, ContactInformation, Id, NoteList};
#[cfg(feature = "process")]
use sea_orm::QueryFilter;
#[cfg(feature = "process")]
use sea_orm::{
    sea_query::{Expr, Func},
    ActiveModelTrait, ColumnTrait, DbBackend, DbConn, EntityTrait, FromQueryResult,
    InsertResult, JsonValue, QuerySelect, RuntimeErr, Set, Statement,
};
use sea_orm::{DbErr, DeleteResult, QueryOrder};
use serde::{Deserialize, Serialize};
use serde_json::json;
use validator::Validate;
use crate::entities::customer::ActiveModel;
use crate::{ContactInformationInput, methods::Error, Session};

#[cfg(feature = "types")]
#[derive(Serialize, Deserialize, Clone, JsonSchema, Validate)]
pub struct Customer {
    pub id: Id,
    pub name: String,
    pub contact: ContactInformation,

    pub customer_notes: NoteList,
    pub balance: i64,

    pub special_pricing: String,
    pub accepts_marketing: bool,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>
}

#[cfg(feature = "process")]
#[derive(Serialize, Deserialize, Clone, FromQueryResult, JsonSchema, Validate)]
pub struct CustomerWithTransactions {
    pub id: Id,
    pub name: String,
    pub contact: JsonValue,

    pub customer_notes: JsonValue,
    pub balance: i64,

    pub special_pricing: JsonValue,
    pub accepts_marketing: bool,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    pub transactions: Option<String>,
}

#[cfg(feature = "types")]
#[derive(Serialize, Deserialize, Clone, JsonSchema, Validate)]
pub struct CustomerWithTransactionsOut {
    pub id: Id,
    pub name: String,
    pub contact: ContactInformation,

    pub customer_notes: NoteList,
    pub balance: i64,

    pub special_pricing: String,
    pub accepts_marketing: bool,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    pub transactions: Option<String>,
}

#[cfg(feature = "types")]
#[derive(Serialize, Deserialize, Clone, JsonSchema, Validate)]
pub struct CustomerInput {
    pub name: String,

    pub contact: ContactInformationInput,
    pub customer_notes: NoteList,

    pub special_pricing: String,
    pub balance: i64,

    pub accepts_marketing: bool,
}

#[cfg(feature = "methods")]
impl Customer {
    pub async fn insert(
        cust: CustomerInput,
        session: Session,
        db: &DbConn,
    ) -> Result<InsertResult<ActiveModel>, Error> {
        let insert_crud = cust.into_active(session.tenant_id);
        Cust::insert(insert_crud).exec(db).await.map_err(|v| v.into())
    }

    pub async fn insert_raw(
        cust: Customer,
        session: Session,
        db: &DbConn,
    ) -> Result<InsertResult<ActiveModel>, Error> {
        let insert_crud = cust.into_active(session.tenant_id);
        Cust::insert(insert_crud).exec(db).await.map_err(|v| v.into())
    }

    pub async fn fetch_by_id(id: &str, session: Session, db: &DbConn) -> Result<Customer, Error> {
        match Cust::find_by_id(id.to_string())
            .filter(customer::Column::TenantId.eq(session.tenant_id))
            .one(db)
            .await? {
            Some(c) => Ok(c.into()),
            None => Err(DbErr::RecordNotFound(
                "Unable to find customer record value".to_string(),
            ).into()),
        }
    }

    pub async fn search(
        query: &str,
        session: Session,
        db: &DbConn,
    ) -> Result<Vec<CustomerWithTransactionsOut>, Error> {
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
                created_at: c.created_at,
                updated_at: c.updated_at
            })
            .collect();

        Ok(mapped)
    }

    pub async fn fetch_by_name(
        name: &str,
        session: Session,
        db: &DbConn,
    ) -> Result<Vec<Customer>, Error> {
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

    pub async fn delete(id: &str, session: Session, db: &DbConn) -> Result<DeleteResult, Error> {
        crate::entities::customer::Entity::delete_by_id(id)
            .filter(customer::Column::TenantId.eq(session.tenant_id))
            .exec(db)
            .await
            .map_err(|v| v.into())
    }

    pub async fn fetch_containing_contact(
        value: &str,
        session: Session,
        db: &DbConn,
    ) -> Result<Vec<Customer>, Error> {
        let res = customer::Entity::find()
            .filter(customer::Column::TenantId.eq(session.tenant_id))
            .having(customer::Column::Contact.contains(value))
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
    ) -> Result<Vec<Customer>, Error> {
        Customer::fetch_containing_contact(phone, session, db).await
    }

    pub async fn fetch_by_addr(
        addr: &str,
        session: Session,
        db: &DbConn,
    ) -> Result<Vec<Customer>, Error> {
        Customer::fetch_containing_contact(addr, session, db).await
    }

    pub async fn fetch_recent(
        session: Session,
        db: &DbConn,
    ) -> Result<Vec<Customer>, Error> {
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
    pub async fn generate(session: Session, db: &DbConn) -> Result<Customer, Error> {
        let cust = example_customer();
        // Insert & Fetch Customer
        let r = Customer::insert(cust, session.clone(), db).await.unwrap();
        Customer::fetch_by_id(&r.last_insert_id, session, db).await
    }

    pub async fn update_by_input(
        cust: CustomerInput,
        session: Session,
        id: &str,
        db: &DbConn,
    ) -> Result<Customer, Error> {
        let old_customer = Self::fetch_by_id(id, session.clone(), db).await?;
        let customer = cust.from_existing(old_customer, session.tenant_id.clone());

        Cust::update(customer).exec(db).await?;

        Self::fetch_by_id(id, session, db).await
    }

    pub async fn update(
        cust: Customer,
        session: Session,
        id: &str,
        db: &DbConn,
    ) -> Result<Customer, Error> {
        let addr = convert_addr_to_geo(&format!(
            "{} {} {} {}",
            cust.contact.address.street,
            cust.contact.address.street2,
            cust.contact.address.po_code,
            cust.contact.address.city
        ));

        match addr {
            Ok(ad) => {
                // Derive the default from the provided customer
                let mut model = cust.clone().into_active(session.tenant_id.clone());

                model.contact = Set(
                    json!(ContactInformation {
                        address: ad,
                        ..cust.contact
                    })
                );

                println!("Have active model: {:?}", model);

                model.update(db).await?;

                Self::fetch_by_id(id, session, db).await
            }
            Err(_) => Err(DbErr::Query(RuntimeErr::Internal(
                "Invalid address format".to_string(),
            )).into()),
        }
    }

    pub async fn update_contact_information(
        contact: ContactInformation,
        id: &str,
        session: Session,
        db: &DbConn,
    ) -> Result<Customer, Error> {
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
                let mut model = cust.clone().into_active(session.tenant_id.clone());

                model.contact = Set(
                    json!(ContactInformation {
                        address: ad,
                        ..cust.contact
                    })
                );

                println!("Have active model: {:?}", model);

                model.update(db).await?;

                Self::fetch_by_id(id, session, db).await
            }
            Err(_) => Err(DbErr::Query(RuntimeErr::Internal(
                "Invalid address format".to_string(),
            )).into()),
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
    let customer = ContactInformationInput {
        name: "Carl Kennith".into(),
        mobile: "0212121204".to_string(),
        email: "carl@kennith.com".to_string(),
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
