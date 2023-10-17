use crate::{Address, ContactInformation, Email, MobileNumber, Store};

pub fn example_stores() -> Vec<Store> {
    vec![
        Store {
            id: "628f74d7-de00-4956-a5b6-2031e0c72128".to_string(),
            name: "Mt Wellington".to_string(),
            contact: ContactInformation {
                name: "Torpedo7 Mt Wellington".into(),
                mobile: MobileNumber {
                    number: "+6421212120".into(),
                    valid: true,
                },
                email: Email {
                    root: "order".into(),
                    domain: "torpedo7.com".into(),
                    full: "order@torpedo7.com".into(),
                },
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
            },
            code: "001".to_string(),
            created_at: Default::default(),
            updated_at: Default::default(),
        },
        Store {
            id: "c4a1d88b-e8a0-4dcd-ade2-1eea82254816".to_string(),
            name: "Westfield".to_string(),
            contact: ContactInformation {
                name: "Torpedo7 Westfield".into(),
                mobile: MobileNumber {
                    number: "+6421212120".into(),
                    valid: true,
                },
                email: Email {
                    root: "order".into(),
                    domain: "torpedo7.com".into(),
                    full: "order@torpedo7.com".into(),
                },
                landline: "".into(),
                address: Address {
                    street: "309 Broadway, Westfield Shopping Centre".into(),
                    street2: "Newmarket".into(),
                    city: "Auckland".into(),
                    country: "New Zealand".into(),
                    po_code: "1023".into(),
                    lat: -36.871820,
                    lon: 174.776730,
                },
            },
            code: "002".to_string(),
            created_at: Default::default(),
            updated_at: Default::default(),
        },
        Store {
            id: "a91509fa-2783-43ae-8c3c-5d5bc5cb6c95".to_string(),
            name: "Albany".to_string(),
            contact: ContactInformation {
                name: "Torpedo7 Albany".into(),
                mobile: MobileNumber {
                    number: "+6421212120".into(),
                    valid: true,
                },
                email: Email {
                    root: "order".into(),
                    domain: "torpedo7.com".into(),
                    full: "order@torpedo7.com".into(),
                },
                landline: "".into(),
                address: Address {
                    street: "6 Mercari Way".into(),
                    street2: "Albany".into(),
                    city: "Auckland".into(),
                    country: "New Zealand".into(),
                    po_code: "0632".into(),
                    lat: -36.7323515,
                    lon: 174.7082982,
                },
            },
            code: "003".to_string(),
            created_at: Default::default(),
            updated_at: Default::default(),
        },
    ]
}
