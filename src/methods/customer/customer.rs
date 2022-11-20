use crate::methods::{Name, ContactInformation, OrderList, NoteList};
use uuid::Uuid;

pub struct Customer {
    pub id: Uuid,
    pub name: Name,
    pub contact: ContactInformation,
    pub order_history: OrderList,
    pub customer_notes: NoteList,
    pub balance: i128,
}