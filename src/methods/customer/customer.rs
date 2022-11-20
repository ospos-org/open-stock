use crate::methods::{Name, ContactInformation, OrderList, NoteList};

pub struct Customer {
    pub id: String,
    pub name: Name,
    pub contact: ContactInformation,
    pub order_history: OrderList,
    pub customer_notes: NoteList,
    pub balance: i128,
}