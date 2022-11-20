use crate::methods::stml::Order;
use chrono::{Utc, DateTime};

use super::{TransactionType, ProductExchange};

pub struct Name {
    pub first: String,
    pub middle: String,
    pub last: String
}

#[derive(Clone, Debug)]
pub struct ContactInformation {
    pub name: String,
    pub mobile: MobileNumber,
    pub email: Email,
    pub landline: String,
    pub address: Address
}

#[derive(Clone, Debug)]
pub struct MobileNumber {
    pub region_code: String,
    pub root: String
}

impl MobileNumber {
    pub fn from(number: String) -> Self {
        MobileNumber { region_code: "+64".into(), root: number }
    }
}

pub type OrderList = Vec<Order>;
pub type NoteList = Vec<Note>;
pub type HistoryList = Vec<History>;

#[derive(Debug)]
pub struct History {
    pub method_type: TransactionType,
    pub item: ProductExchange,
    pub reason: String
}

#[derive(Clone, Debug)]
pub struct Email {
    root: String,
    domain: String,
    full: String
}

impl Email {
    pub fn from(email: String) -> Self {
        let split = email.split("@");
        let col = split.collect::<Vec<&str>>();

        let root = match col.get(0) {
            Some(root) => {
                *root
            },
            None => "",
        };

        let domain = match col.get(1) {
            Some(domain) => {
                *domain
            },
            None => "",
        };

        Email {
            root: root.into(),
            domain: domain.into(),
            full: email
        }
    }
}



#[derive(Debug)]
pub struct Note {
    pub message: String,
    pub timestamp: DateTime<Utc>
}

#[derive(Clone, Debug)]
pub struct Address {
    pub street: String,
    pub street2: String,
    pub city: String,
    pub country: String,
    pub po_code: String
}

#[derive(Debug)]
pub struct Location {
    pub code: String,
    // Address is stored in the contact information.
    pub contact: ContactInformation
}

pub type Url = String;

pub type TagList = Vec<Tag>;
pub type Tag = String;
pub type Id = String;