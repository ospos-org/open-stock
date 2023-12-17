use chrono::{Days, Duration, Utc};
use uuid::Uuid;
use crate::{Address, ContactInformation, CustomerType, DiscountValue, Email, History, Location, MobileNumber, Note, Order, OrderStatus, OrderStatusAssignment, Payment, PaymentAction, PaymentProcessor, PaymentStatus, PickStatus, Price, ProductInstance, ProductPurchase, TransactionCustomer, TransactionInit, TransactionType, TransitInformation};

pub fn example_transaction(customer_id: &str) -> TransactionInit {
    let torpedo7 = ContactInformation {
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
    };

    let order = Order {
        destination: Location {
            store_code: "001".into(),
            store_id: "628f74d7-de00-4956-a5b6-2031e0c72128".to_string(),
            contact: torpedo7.clone(),
        },
        order_type: crate::methods::OrderType::Shipment,
        origin: Location {
            store_code: "002".into(),
            store_id: "c4a1d88b-e8a0-4dcd-ade2-1eea82254816".to_string(),
            contact: torpedo7.clone(),
        },
        products: vec![
            ProductPurchase {
                product_name: "Torpedo7 Nippers Kids Kayak & Paddle".to_string(),
                product_variant_name: "1.83m Beaches".to_string(),
                id: "PDT-KAYAK-PURCHASE-ID-1".to_string(),
                product_sku: "".into(),
                product_code: "54897443288214".into(),
                discount: DiscountValue::Absolute(0),
                product_cost: 399.99,
                quantity: 1.0,
                transaction_type: TransactionType::Out,
                tags: vec!["Tee".into(), "Cotton".into(), "Organic".into()],
                instances: vec![ProductInstance {
                    id: "def".to_string(),
                    fulfillment_status: crate::FulfillmentStatus {
                        pick_status: PickStatus::Pending,
                        pick_history: vec![],
                        last_updated: Utc::now(),
                        notes: vec![],
                    },
                }],
            },
            ProductPurchase {
                product_name: "Torpedo7 Kids Voyager II Paddle Vest".to_string(),
                product_variant_name: "Small Red (4-6y)".to_string(),
                id: "PDT-LIFEJACKET-PURCHASE-ID-1".to_string(),
                product_sku: "".into(),
                product_code: "51891265958214".into(),
                discount: DiscountValue::Absolute(0),
                product_cost: 139.99,
                quantity: 1.0,
                transaction_type: TransactionType::Out,
                tags: vec!["Tee".into(), "Cotton".into(), "Organic".into()],
                instances: vec![ProductInstance {
                    id: "def".to_string(),
                    fulfillment_status: crate::FulfillmentStatus {
                        pick_status: PickStatus::Pending,
                        pick_history: vec![],
                        last_updated: Utc::now(),
                        notes: vec![],
                    },
                }],
            },
        ],
        previous_failed_fulfillment_attempts: vec![],
        status: OrderStatusAssignment {
            // status: OrderStatus::Transit(
            //     TransitInformation {
            //         shipping_company: torpedo7.clone(),
            //         query_url: "https://www.fedex.com/fedextrack/?trknbr=".into(),
            //         tracking_code: "1523123".into(),
            //         assigned_products: vec!["132522-22".to_string()]
            //     }
            // )
            status: OrderStatus::Fulfilled(Utc::now()),
            assigned_products: vec!["132522-22".to_string()],
            timestamp: Utc::now(),
        },
        order_history: vec![],
        order_notes: vec![Note {
            message: "Order shipped from warehouse.".into(),
            timestamp: Utc::now(),
            author: Uuid::new_v4().to_string(),
        }],
        reference: "TOR-19592".into(),
        creation_date: Utc::now(),
        id: Uuid::new_v4().to_string(),
        status_history: vec![
            History::<OrderStatusAssignment> {
                item: OrderStatusAssignment {
                    status: OrderStatus::Queued(Utc::now()),
                    timestamp: Utc::now(),
                    assigned_products: vec!["PDT-KAYAK-PURCHASE-ID-1".to_string()],
                },
                timestamp: Utc::now(),
                reason: "Order Placed".to_string(),
            },
            History::<OrderStatusAssignment> {
                item: OrderStatusAssignment {
                    status: OrderStatus::Processing(
                        Utc::now().checked_add_signed(Duration::hours(1)).unwrap(),
                    ),
                    timestamp: Utc::now().checked_add_signed(Duration::hours(1)).unwrap(),
                    assigned_products: vec!["PDT-KAYAK-PURCHASE-ID-1".to_string()],
                },
                timestamp: Utc::now().checked_add_signed(Duration::hours(1)).unwrap(),
                reason: "Order received by store crew.".to_string(),
            },
            History::<OrderStatusAssignment> {
                item: OrderStatusAssignment {
                    status: OrderStatus::Transit(Box::new(TransitInformation {
                        shipping_company: torpedo7,
                        query_url: "https://www.fedex.com/fedextrack/?trknbr=".into(),
                        tracking_code: "1523123".into(),
                        assigned_products: vec!["132522-22".to_string()],
                    })),
                    timestamp: Utc::now().checked_add_signed(Duration::hours(2)).unwrap(),
                    assigned_products: vec!["132522-22".to_string()],
                },
                timestamp: Utc::now().checked_add_signed(Duration::hours(2)).unwrap(),
                reason: "Order shipped from warehouse.".to_string(),
            },
            History::<OrderStatusAssignment> {
                item: OrderStatusAssignment {
                    status: OrderStatus::Fulfilled(
                        Utc::now().checked_add_days(Days::new(2)).unwrap(),
                    ),
                    timestamp: Utc::now().checked_add_days(Days::new(2)).unwrap(),
                    assigned_products: vec!["132522-22".to_string()],
                },
                timestamp: Utc::now().checked_add_days(Days::new(2)).unwrap(),
                reason: "Item Delivered".to_string(),
            },
        ],
        discount: DiscountValue::Absolute(0),
    };

    TransactionInit {
        customer: TransactionCustomer {
            customer_id: customer_id.into(),
            customer_type: CustomerType::Individual,
        },
        transaction_type: TransactionType::In,
        products: vec![order],
        order_total: 115_i64,
        payment: vec![Payment {
            id: Uuid::new_v4().to_string(),
            payment_method: crate::methods::PaymentMethod::Card,
            fulfillment_date: Utc::now(),
            amount: Price {
                quantity: 115.00,
                currency: "NZD".to_string(),
            },
            processing_fee: Price {
                quantity: 0.10,
                currency: "NZD".to_string(),
            },
            status: PaymentStatus::Unfulfilled(String::from(
                "Unable to fulfil payment requirements - insufficient funds.",
            )),
            processor: PaymentProcessor {
                location: "001".to_string(),
                employee: "EMPLOYEE_ID".to_string(),
                software_version: "k0.5.2".to_string(),
                token: Uuid::new_v4().to_string(),
            },
            order_ids: vec![Uuid::new_v4().to_string()],
            delay_action: PaymentAction::Cancel,
            delay_duration: "PT12H".to_string(),
        }],
        order_date: Utc::now(),
        order_notes: vec![Note {
            message: "Order packaged from warehouse.".into(),
            timestamp: Utc::now(),
            author: Uuid::new_v4().to_string(),
        }],
        // order_history: vec![History { item: ProductExchange { method_type: TransactionType::Out, product_code: "132522".into(), variant: vec!["22".into()], quantity: 1 }, reason: "Faulty Product".into(), timestamp: Utc::now() }],
        kiosk: "...".into(),
    }
}
