use crate::methods::{Id, Name, ContactInformation, History};


pub struct Employee {
    pub id: Id,
    pub name: Name,
    pub contact: ContactInformation,
    pub clock_history: Vec<History<Attendance>>
}

pub struct Attendance {
    pub track_type: TrackType,
    pub till: Id
}

pub enum TrackType {
    In, Out
}