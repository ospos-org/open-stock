use std::{fs::File, thread, time::Duration, net::{TcpStream, TcpListener}, io::prelude::*};
use methods::{Address, ContactInformation, Location, MobileNumber, Note, OrderStatus, TransitInformation, Order, Email, Transaction};

use uuid::Uuid;
use chrono::Utc;
use lib::ThreadPool;

use crate::methods::{ProductPurchase, DiscountValue, Payment, NoteList, History, OrderState, ProductExchange};

mod lib;
mod methods;

fn main() {
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
        id: Uuid::new_v4(),
        status_history: vec![OrderState { status: OrderStatus::Queued, date: Utc::now() }],
        discount: DiscountValue::Absolute(0),
    };
    
    let transaction = Transaction {
        id: Uuid::new_v4(),
        customer: "...".into(),
        transaction_type: methods::TransactionType::In,
        products: vec![order],
        order_total: 115,
        payment: Payment {
            payment_method: methods::PaymentMethod::Card,
            fulfillment_date: Utc::now(),
        },
        order_date: Utc::now(),
        order_notes: vec![Note { message: "Order packaged from warehouse.".into(), timestamp: Utc::now() }],
        order_history: vec![History { method_type: methods::TransactionType::Out, item: ProductExchange { product_code: "132522".into(), variant: vec!["22".into()], quantity: 1 }, reason: "Faulty Product".into() }],
        salesperson: "...".into(),
        till: "...".into(),
    };

    println!("Authored transaction of {:?}", transaction);

    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(4);
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        pool.execute(||{
            handle_connection(stream);
        });
    }
}

fn handle_connection(mut stream: TcpStream){
    let mut buffer = [0;1024];
    stream.read(&mut buffer).unwrap();

    let get = b"GET / HTTP/1.1\r\n";
    let sleep = b"GET /sleep HTTP/1.1\r\n";
    let (status_line, _filename) = if buffer.starts_with(get){
        ("HTTP/1.1 200 OK\r\n\r\n", "hello.html")}
    else if buffer.starts_with(sleep){
        thread::sleep(Duration::from_secs(5));
        ("HTTP/1.1 200 OK\r\n\r\n","hello.html")
    }
    else {
        ("HTTP/1.1 404 NOT FOUND\r\n\r\n", "404.html")
    };

    let mut file = File::open("hello.html").unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    let response = format!("{}{}", status_line, contents);
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}