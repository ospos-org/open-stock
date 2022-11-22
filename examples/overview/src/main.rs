use methods::{Address, ContactInformation, Location, MobileNumber, Note, OrderStatus, TransitInformation, Order, Email, Transaction, Employee, Name, Attendance, Product, Customer, Variant, VariantCategory, Stock, Quantity};
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

    // Create Product
    let (pdt, sku) = example_product();
    // Insert & Fetch Product
    Product::insert(pdt, &db).await.unwrap();
    match Product::fetch_by_id(&sku, &db).await {
        Ok(pdt) => {
            println!("Retrieved Product:\n{}", pdt)
        }
        Err(e) => panic!("{}", e)
    }

    // Create Customer
    let (cust, id) = example_customer();
    // Insert & Fetch Customer
    Customer::insert(cust, &db).await.unwrap();
    match Customer::fetch_by_id(&id, &db).await {
        Ok(cust) => {
            println!("Retrieved Customer:\n{}", cust)
        }
        Err(e) => panic!("{}", e)
    }
}

fn example_customer() -> (Customer, String) {
    let id = Uuid::new_v4().to_string();

    let customer = ContactInformation {
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
    };

    (Customer {
        id: id.clone(),
        name: Name { first: "".into(), middle: "".into(), last: "".into() },
        contact: customer.clone(),
        order_history: vec![
            Order {
                destination: Location {
                    code: "001".into(),
                    contact: customer.clone()
                },
                origin: Location {
                    code: "002".into(),
                    contact: customer.clone()
                },
                products: vec![
                    ProductPurchase { product_code:"132522".into(), discount: DiscountValue::Absolute(0), product_cost: 15, variant: vec!["22".into()], quantity: 5 },
                    ProductPurchase { product_code:"132522".into(), discount: DiscountValue::Absolute(0), product_cost: 15, variant: vec!["23".into()], quantity: 5 }
                ],
                status: OrderStatus::Transit(
                    TransitInformation {
                        shipping_company: customer.clone(),
                        query_url: "https://www.fedex.com/fedextrack/?trknbr=".into(),
                        tracking_code: "1523123".into(),
                    }
                ),
                order_notes: vec![Note { message: "Order Shipped from Depot".into(), timestamp: Utc::now() }],
                reference: "TOR-19592".into(),
                creation_date: Utc::now(),
                id: Uuid::new_v4().to_string(),
                status_history: vec![OrderState { status: OrderStatus::Queued, timestamp: Utc::now() }],
                discount: DiscountValue::Absolute(0),
            }
        ],
        customer_notes: vec![],
        balance: 0,
    }, id)
}

fn example_product() -> (Product, String) {
    let sku = "123857".to_string();

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

    (Product {
        name: "Surfboard Fun 7'6\"".to_string(),
        variants: vec![
            VariantCategory { 
                category: "Color".into(), 
                variants: vec![
                    Variant {
                        name:"White".into(), 
                        stock: vec![
                            Stock {
                                store: Location { 
                                    code: "001".into(), 
                                    contact: torpedo7 
                                },
                                quantity: Quantity {
                                    quantity_on_hand: 2,
                                    quantity_on_order: 1,
                                    quantity_on_floor: 1
                                }
                            }
                        ], 
                        images: vec!["https://www.torpedo7.co.nz/images/products/F1S8CN8VAXX_zoom---surfboard-7ft-6in-fun-white.jpg?v=075e26aa5b6847e8bbd2".into(), "https://www.torpedo7.co.nz/images/products/F1S8CN8VAXX_zoom_1---surfboard-7ft-6in-fun-white.jpg?v=075e26aa5b6847e8bbd2".into(), "https://www.torpedo7.co.nz/images/products/F1S8CN8VAXX_zoom_2---surfboard-7ft-6in-fun-white.jpg?v=075e26aa5b6847e8bbd2".into()], 
                        marginal_price: 550, 
                        variant_code: "01".into(), 
                        order_history: vec![] 
                    }
                ]
            }
        ],
        company: "Torq".to_string(),
        sku: sku.clone(),
        loyalty_discount: DiscountValue::Absolute(15),
        images: vec!["https://www.torpedo7.co.nz/images/products/F1S8CN8VAXX_zoom---surfboard-7ft-6in-fun-white.jpg?v=075e26aa5b6847e8bbd2".into(), "https://www.torpedo7.co.nz/images/products/F1S8CN8VAXX_zoom_1---surfboard-7ft-6in-fun-white.jpg?v=075e26aa5b6847e8bbd2".into(), "https://www.torpedo7.co.nz/images/products/F1S8CN8VAXX_zoom_2---surfboard-7ft-6in-fun-white.jpg?v=075e26aa5b6847e8bbd2".into()],
        tags: vec!["Surfboard".into(), "Water".into()],
        description: "This crossover range caters to all levels of surfers in virtually every condition. From waist high mush to overhead and hollow. The versatility of the Mod Fun makes them an excellent choice if you need one board to handle all the conditions where you live and travel.
        Featuring a medium full nose and shallow mid-entry there is enough volume for smaller days and weaker surf. As the surf jumps up, step back and the board transforms. You'll find a board that feels shorter than it's length, delivering predictable handling and performance.  Tri-fin set-up.
        Our fin system is designed by Futures Fins of California - one of the most respected fin systems on the planet.
        Torq TET surfboards all come with fins. The ModFun shapes come with 3 fin boxes and a Thruster fin 
        set offering an even balance of drive and release for all round surfing.".into(),
        specifications: vec![("Difficulty".to_string(), "Expert".to_string()), ("Wave Height".to_string(), "2-6ft".to_string()), ("Dimensions".to_string(), "7'6\" x 21 1/2\" x 2 7/8\"".to_string())],
    }, sku)
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
            History::<Attendance> { item: Attendance { track_type: methods::TrackType::In, till: "5".to_string() }, reason: "".to_string(), timestamp: Utc::now() },
            History::<Attendance> { item: Attendance { track_type: methods::TrackType::Out, till: "6".to_string() }, reason: "".to_string(), timestamp: Utc::now() },
            History::<Attendance> { item: Attendance { track_type: methods::TrackType::In, till: "1".to_string() }, reason: "".to_string(), timestamp: Utc::now() },
            History::<Attendance> { item: Attendance { track_type: methods::TrackType::Out, till: "3".to_string() }, reason: "".to_string(), timestamp: Utc::now() },
            History::<Attendance> { item: Attendance { track_type: methods::TrackType::In, till: "4".to_string() }, reason: "".to_string(), timestamp: Utc::now() },
            History::<Attendance> { item: Attendance { track_type: methods::TrackType::Out, till: "4".to_string() }, reason: "Left Early".to_string(), timestamp: Utc::now() },
            History::<Attendance> { item: Attendance { track_type: methods::TrackType::In, till: "4".to_string() }, reason: "".to_string(), timestamp: Utc::now() },
            History::<Attendance> { item: Attendance { track_type: methods::TrackType::Out, till: "5".to_string() }, reason: "".to_string(), timestamp: Utc::now() },
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
        status_history: vec![OrderState { status: OrderStatus::Queued, timestamp: Utc::now() }],
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
        order_history: vec![History { item: ProductExchange { method_type: TransactionType::Out, product_code: "132522".into(), variant: vec!["22".into()], quantity: 1 }, reason: "Faulty Product".into(), timestamp: Utc::now() }],
        salesperson: "...".into(),
        till: "...".into(),
    };

    (transaction, id)
}