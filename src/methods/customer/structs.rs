use std::fmt::Display;

use crate::entities::prelude::Customer as Cust;
use crate::Session;
use crate::{
    entities::customer,
    methods::{
        convert_addr_to_geo, Address, ContactInformation, Email, Id, MobileNumber, NoteList,
    },
};
use sea_orm::QueryFilter;
use sea_orm::{
    sea_query::{Expr, Func},
    ActiveModelTrait, ColumnTrait, DbBackend, DbConn, DbErr, EntityTrait, FromQueryResult,
    InsertResult, JsonValue, QuerySelect, RuntimeErr, Set, Statement,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

#[cfg(feature = "types")]
#[derive(Serialize, Deserialize, Clone)]
pub struct Customer {
    pub id: Id,
    pub name: String,
    pub contact: ContactInformation,
    pub customer_notes: NoteList,
    pub balance: f32,
    pub special_pricing: String,
    pub accepts_marketing: bool,
}

#[cfg(feature = "types")]
#[derive(Serialize, Deserialize, Clone, FromQueryResult)]
pub struct CustomerWithTransactions {
    pub id: Id,
    pub name: String,
    pub contact: JsonValue,
    pub customer_notes: JsonValue,
    pub balance: f32,
    pub special_pricing: JsonValue,
    pub transactions: Option<String>,
    pub accepts_marketing: bool,
}

#[cfg(feature = "types")]
#[derive(Serialize, Deserialize, Clone)]
pub struct CustomerWithTransactionsOut {
    pub id: Id,
    pub name: String,
    pub contact: ContactInformation,
    pub customer_notes: NoteList,
    pub balance: f32,
    pub special_pricing: String,
    pub transactions: Option<String>,
    pub accepts_marketing: bool,
}

#[cfg(feature = "types")]
#[derive(Serialize, Deserialize, Clone)]
pub struct CustomerInput {
    pub name: String,
    pub contact: ContactInformation,
    pub customer_notes: NoteList,
    pub special_pricing: String,
    pub balance: f32,
    pub accepts_marketing: bool,
}

#[cfg(feature = "methods")]
impl Customer {
    pub async fn insert(
        cust: CustomerInput,
        session: Session,
        db: &DbConn,
    ) -> Result<InsertResult<customer::ActiveModel>, DbErr> {
        let id = Uuid::new_v4().to_string();

        let insert_crud = customer::ActiveModel {
            id: Set(id),
            name: Set(cust.name),
            contact: Set(json!(cust.contact)),
            customer_notes: Set(json!(cust.customer_notes)),
            balance: Set(cust.balance),
            special_pricing: Set(json!(cust.special_pricing)),
            accepts_marketing: Set(cust.accepts_marketing),
            tenant_id: Set(session.tenant_id),
        };

        match Cust::insert(insert_crud).exec(db).await {
            Ok(res) => Ok(res),
            Err(err) => Err(err),
        }
    }

    pub async fn insert_raw(
        cust: Customer,
        session: Session,
        db: &DbConn,
    ) -> Result<InsertResult<customer::ActiveModel>, DbErr> {
        let insert_crud = customer::ActiveModel {
            id: Set(cust.id),
            name: Set(cust.name),
            contact: Set(json!(cust.contact)),
            customer_notes: Set(json!(cust.customer_notes)),
            balance: Set(cust.balance),
            special_pricing: Set(json!(cust.special_pricing)),
            accepts_marketing: Set(cust.accepts_marketing),
            tenant_id: Set(session.tenant_id),
        };

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
            Some(c) => Ok(Customer {
                id: c.id,
                name: c.name,
                contact: serde_json::from_value::<ContactInformation>(c.contact).unwrap(),
                customer_notes: serde_json::from_value::<NoteList>(c.customer_notes).unwrap(),
                special_pricing: serde_json::from_value::<String>(c.special_pricing).unwrap(),
                balance: c.balance,
                accepts_marketing: c.accepts_marketing,
            }),
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

        let mapped = res
            .iter()
            .map(|c| Customer {
                id: c.id.clone(),
                name: c.name.clone(),
                contact: serde_json::from_value::<ContactInformation>(c.contact.clone()).unwrap(),
                customer_notes: serde_json::from_value::<NoteList>(c.customer_notes.clone())
                    .unwrap(),
                special_pricing: serde_json::from_value::<String>(c.special_pricing.clone())
                    .unwrap(),
                balance: c.balance,
                accepts_marketing: c.accepts_marketing,
            })
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

        let mapped = res
            .iter()
            .map(|c| Customer {
                id: c.id.clone(),
                name: c.name.clone(),
                contact: serde_json::from_value::<ContactInformation>(c.contact.clone()).unwrap(),
                customer_notes: serde_json::from_value::<NoteList>(c.customer_notes.clone())
                    .unwrap(),
                special_pricing: serde_json::from_value::<String>(c.special_pricing.clone())
                    .unwrap(),
                balance: c.balance,
                accepts_marketing: c.accepts_marketing,
            })
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

        let mapped = res
            .iter()
            .map(|c| Customer {
                id: c.id.clone(),
                name: c.name.clone(),
                contact: serde_json::from_value::<ContactInformation>(c.contact.clone()).unwrap(),
                customer_notes: serde_json::from_value::<NoteList>(c.customer_notes.clone())
                    .unwrap(),
                special_pricing: serde_json::from_value::<String>(c.special_pricing.clone())
                    .unwrap(),
                balance: c.balance,
                accepts_marketing: c.accepts_marketing,
            })
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
                let mut new_contact = cust.contact;
                new_contact.address = ad;

                customer::ActiveModel {
                    id: Set(id.to_string()),
                    name: Set(cust.name),
                    contact: Set(json!(new_contact)),
                    customer_notes: Set(json!(cust.customer_notes)),
                    special_pricing: Set(json!(cust.special_pricing)),
                    balance: Set(cust.balance),
                    accepts_marketing: Set(cust.accepts_marketing),
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

    pub async fn update_contact_information(
        contact: ContactInformation,
        id: &str,
        session: Session,
        db: &DbConn,
    ) -> Result<Customer, DbErr> {
        let customer = Self::fetch_by_id(id, session.clone(), db).await?;
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
                let mut new_contact = contact;
                new_contact.address = ad;

                customer::ActiveModel {
                    id: Set(customer.id),
                    name: Set(customer.name),
                    contact: Set(json!(new_contact)),
                    customer_notes: Set(json!(customer.customer_notes)),
                    special_pricing: Set(json!(customer.special_pricing)),
                    balance: Set(customer.balance),
                    accepts_marketing: Set(customer.accepts_marketing),
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
        balance: 0.0,
        accepts_marketing: true,
    }
}
