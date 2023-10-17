use std::fmt::Display;
use chrono::{DateTime, Utc};

#[cfg(feature = "process")]
use crate::entities::prelude::Supplier as Suppl;
#[cfg(feature = "process")]
use crate::entities::supplier;
use crate::Session;

use crate::methods::{ContactInformation, Name, Transaction};

#[cfg(feature = "process")]
use crate::methods::convert_addr_to_geo;

#[cfg(feature = "process")]
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbConn, DbErr, EntityTrait, InsertResult, QueryFilter,
    QuerySelect, RuntimeErr,
};
use sea_orm::ActiveValue::Set;
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;
use crate::methods::supplier::example::example_supplier;

#[cfg(feature = "types")]
#[derive(Serialize, Deserialize, Clone)]
pub struct Supplier {
    pub id: String,
    pub name: Name,

    pub contact: ContactInformation,
    pub transaction_history: Vec<Transaction>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>
}

#[cfg(feature = "types")]
#[derive(Serialize, Deserialize, Clone)]
pub struct SupplierInput {
    pub name: Name,
    pub contact: ContactInformation,
    pub transaction_history: Vec<Transaction>,
}

#[cfg(feature = "methods")]
impl Supplier {
    pub async fn insert(
        suppl: SupplierInput,
        session: Session,
        db: &DbConn,
    ) -> Result<InsertResult<supplier::ActiveModel>, DbErr> {
        let id = Uuid::new_v4().to_string();

        match Suppl::insert(
            suppl.into_active(id, session.tenant_id.clone())
        ).exec(db).await {
            Ok(res) => Ok(res),
            Err(err) => Err(err),
        }
    }

    pub async fn fetch_by_id(id: &str, session: Session, db: &DbConn) -> Result<Supplier, DbErr> {
        let suppl = Suppl::find_by_id(id.to_string())
            .filter(supplier::Column::TenantId.eq(session.tenant_id))
            .one(db)
            .await?;
        let s = suppl.unwrap();

        Ok(s.into())
    }

    pub async fn fetch_by_name(
        name: &str,
        session: Session,
        db: &DbConn,
    ) -> Result<Vec<Supplier>, DbErr> {
        let res = supplier::Entity::find()
            .filter(supplier::Column::TenantId.eq(session.tenant_id))
            .having(supplier::Column::Name.contains(name))
            .limit(25)
            .all(db)
            .await?;

        let mapped = res
            .iter()
            .map(|s| s.clone().into())
            .collect();

        Ok(mapped)
    }

    pub async fn fetch_by_phone(
        phone: &str,
        session: Session,
        db: &DbConn,
    ) -> Result<Vec<Supplier>, DbErr> {
        let res = supplier::Entity::find()
            .filter(supplier::Column::TenantId.eq(session.tenant_id))
            .having(supplier::Column::Contact.contains(phone))
            .limit(25)
            .all(db)
            .await?;

        let mapped = res
            .iter()
            .map(|s| s.clone().into())
            .collect();

        Ok(mapped)
    }

    pub async fn fetch_by_addr(
        addr: &str,
        session: Session,
        db: &DbConn,
    ) -> Result<Vec<Supplier>, DbErr> {
        let res = supplier::Entity::find()
            .filter(supplier::Column::TenantId.eq(session.tenant_id))
            .having(supplier::Column::Contact.contains(addr))
            .limit(25)
            .all(db)
            .await?;

        let mapped = res
            .iter()
            .map(|s| s.clone().into())
            .collect();

        Ok(mapped)
    }

    /// Generate and insert a default customer.
    pub async fn generate(session: Session, db: &DbConn) -> Result<Supplier, DbErr> {
        let cust = example_supplier();
        // Insert & Fetch Customer
        let r = Supplier::insert(cust, session.clone(), db).await.unwrap();
        match Supplier::fetch_by_id(&r.last_insert_id, session, db).await {
            Ok(cust) => Ok(cust),
            Err(e) => Err(e),
        }
    }

    pub async fn update(
        suppl: SupplierInput,
        session: Session,
        id: &str,
        db: &DbConn,
    ) -> Result<Supplier, DbErr> {
        let addr = convert_addr_to_geo(&format!(
            "{} {} {} {}",
            suppl.contact.address.street,
            suppl.contact.address.street2,
            suppl.contact.address.po_code,
            suppl.contact.address.city
        ));

        match addr {
            Ok(ad) => {
                let mut new_contact = suppl.contact.clone();
                new_contact.address = ad;

                let mut supplier = suppl.into_active(
                    id.to_string(),session.tenant_id.clone()
                );
                supplier.contact = Set(json!(new_contact));

                supplier.update(db).await?;

                Self::fetch_by_id(id, session, db).await
            }
            Err(_) => Err(DbErr::Query(RuntimeErr::Internal(
                "Invalid address format".to_string(),
            ))),
        }
    }
}

impl Display for Supplier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let order_history: String = self
            .transaction_history
            .iter()
            .map(|f| {
                format!(
                    "{}: {:?}\n",
                    f.order_date.format("%d/%m/%Y %H:%M"),
                    f.transaction_type,
                )
            })
            .collect();

        write!(
            f,
            "{} {} \n{}\n({}) {} {}\n\n[Clock History]\n{}\n
            ",
            self.name.first,
            self.name.last,
            self.id,
            self.contact.mobile.number,
            if self.contact.mobile.valid {
                "VALID"
            } else {
                "ELSE"
            },
            self.contact.email.full,
            order_history,
        )
    }
}
