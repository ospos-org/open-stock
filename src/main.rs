use methods::{Address, ContactInformation, Location, MobileNumber, Note, OrderStatus, TransitInformation, Order, Email, Transaction, Employee, Name, Attendance};
use crate::{methods::{ProductPurchase, DiscountValue, Payment, History, OrderState, ProductExchange}, entities::sea_orm_active_enums::TransactionType};

use sea_orm::Database;
use uuid::Uuid;
use chrono::Utc;

mod methods;
mod entities;

use dotenv::dotenv;
use std::env;

#[async_std::main]
async fn main() {
    dotenv().ok();

    let database_url = match env::var("DATABASE_URL") {
        Ok(url) => url,
        Err(err) => {
            panic!("Was unable to initialize, could not determine the database url. Reason: {}", err)
        },
    };

    let db = Database::connect(database_url) 
        .await
        .unwrap();

    // Create Transaction
    let (tsn, id) = example_transaction();
    // Insert & Fetch Transaction
    Transaction::insert(tsn, &db).await.unwrap();
    match Transaction::fetch_by_id(&id, &db).await {
        Ok(ts) => {
            println!("Retrieved Transaction:\n{}", ts);
        }
        Err(e) => panic!("{}", e)
    }

    // Create Employee
    let (empl, id) = example_employee();
    // Insert & Fetch Employee
    Employee::insert(empl, &db).await.unwrap();
    match Employee::fetch_by_id(&id, &db).await {
        Ok(emp) => {
            println!("Retrieved Employee:\n{}", emp);
        }
        Err(e) => panic!("{}", e)
    }

}

fn example_employee() -> (Employee, String) {
    let id = Uuid::new_v4().to_string();

    let employee = Employee {
        id: id.clone(),
        name: Name {
            first: "Carl".to_string(),
            middle: "".to_string(),
            last: "Kennith".to_string()
        },
        contact: ContactInformation {
            name: "Carl Kennith".into(),
            mobile: MobileNumber::from("021212120".to_string()),
            email: Email::from("carl@kennith.com".to_string()),
            landline: "".into(),
            address: Address {
                street: "9 Carbine Road".into(),
                street2: "".into(),
                city: "Auckland".into(),
                country: "New Zealand".into(),
                po_code: "100".into(),
            },
        },
        clock_history: vec![
            History::<Attendance> { item: Attendance { track_type: methods::TrackType::In, till: "5".to_string() }, reason: "".to_string(), date: Utc::now() },
            History::<Attendance> { item: Attendance { track_type: methods::TrackType::Out, till: "6".to_string() }, reason: "".to_string(), date: Utc::now() },
            History::<Attendance> { item: Attendance { track_type: methods::TrackType::In, till: "1".to_string() }, reason: "".to_string(), date: Utc::now() },
            History::<Attendance> { item: Attendance { track_type: methods::TrackType::Out, till: "3".to_string() }, reason: "".to_string(), date: Utc::now() },
            History::<Attendance> { item: Attendance { track_type: methods::TrackType::In, till: "4".to_string() }, reason: "".to_string(), date: Utc::now() },
            History::<Attendance> { item: Attendance { track_type: methods::TrackType::Out, till: "4".to_string() }, reason: "Left Early".to_string(), date: Utc::now() },
            History::<Attendance> { item: Attendance { track_type: methods::TrackType::In, till: "4".to_string() }, reason: "".to_string(), date: Utc::now() },
            History::<Attendance> { item: Attendance { track_type: methods::TrackType::Out, till: "5".to_string() }, reason: "".to_string(), date: Utc::now() },
        ],
        level: 2,
    };

    (employee, id)
}

fn example_transaction() -> (Transaction, String) {
    let torpedo7 = ContactInformation {
        name: "Torpedo7".into(),
        mobile: MobileNumber::from("021212120".to_string()),
        email: Email::from("order@torpedo7.com".to_string()),
        landline: "".into(),
        address: Address {
            street: "9 Carbine Road".into(),
            street2: "".into(),
            city: "Auckland".into(),
            country: "New Zealand".into(),
            po_code: "100".into(),
        },
    };

    let order = Order {
        destination: Location {
            code: "001".into(),
            contact: torpedo7.clone()
        },
        origin: Location {
            code: "002".into(),
            contact: torpedo7.clone()
        },
        products: vec![
            ProductPurchase { product_code:"132522".into(), discount: DiscountValue::Absolute(0), product_cost: 15, variant: vec!["22".into()], quantity: 5 },
            ProductPurchase { product_code:"132522".into(), discount: DiscountValue::Absolute(0), product_cost: 15, variant: vec!["23".into()], quantity: 5 }
        ],
        status: OrderStatus::Transit(
            TransitInformation {
                shipping_company: torpedo7.clone(),
                query_url: "https://www.fedex.com/fedextrack/?trknbr=".into(),
                tracking_code: "1523123".into(),
            }
        ),
        order_notes: vec![Note { message: "Order Shipped from Depot".into(), timestamp: Utc::now() }],
        reference: "TOR-19592".into(),
        creation_date: Utc::now(),
        id: Uuid::new_v4().to_string(),
        status_history: vec![OrderState { status: OrderStatus::Queued, date: Utc::now() }],
        discount: DiscountValue::Absolute(0),
    };

    let id = Uuid::new_v4().to_string();
    
    let transaction = Transaction {
        id: id.clone(),
        customer: "...".into(),
        transaction_type: TransactionType::In,
        products: vec![order],
        order_total: 115,
        payment: Payment {
            payment_method: methods::PaymentMethod::Card,
            fulfillment_date: Utc::now(),
        },
        order_date: Utc::now(),
        order_notes: vec![Note { message: "Order packaged from warehouse.".into(), timestamp: Utc::now() }],
        order_history: vec![History { item: ProductExchange { method_type: TransactionType::Out, product_code: "132522".into(), variant: vec!["22".into()], quantity: 1 }, reason: "Faulty Product".into(), date: Utc::now() }],
        salesperson: "...".into(),
        till: "...".into(),
    };

    (transaction, id)
}