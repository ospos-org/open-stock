use crate::{Address, ContactInformation, Email, MobileNumber, Name, SupplierInput};

pub fn example_supplier() -> SupplierInput {
    let customer = ContactInformation {
        name: "Carl Kennith".into(),
        mobile: MobileNumber::from("021212120".to_string()),
        email: Email::from("carl@kennith.com".to_string()),
        landline: "".into(),
        address: Address {
            street: "315-375 Mount Wellington Highway".into(),
            street2: "Mount Wellington".into(),
            city: "Auckland".into(),
            country: "New Zealand".into(),
            po_code: "1060".into(),
            lat: -36.915501,
            lon: 174.838745,
        },
    };

    SupplierInput {
        name: Name {
            first: "OJI".into(),
            middle: "Fibre".into(),
            last: "Solutions".into(),
        },
        contact: customer,
        transaction_history: vec![],
    }
}
