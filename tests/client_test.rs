use std::time::{Duration, Instant};

#[cfg(feature = "process")]
#[tokio::test]
async fn main() {
    let mut durations: Vec<u128> = vec![]; 

    let client = reqwest::Client::new();

    match tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_millis(10));

        for i in 0..1000 {
            interval.tick().await;

            {
                let start = Instant::now();
                match client.post("http://127.0.0.1:8000/customer/")
                    .body("{\r\n    \"name\": {\r\n        \"first\": \"Carl\",\r\n        \"middle\": \"\",\r\n        \"last\": \"Kennith\"\r\n    },\r\n    \"contact\": {\r\n        \"name\": \"Carl Kennith\",\r\n        \"mobile\": {\r\n            \"region_code\": \"+64\",\r\n            \"root\": \"021212120\"\r\n        },\r\n        \"email\": {\r\n            \"root\": \"carl\",\r\n            \"domain\": \"kennith.com\",\r\n            \"full\": \"carl@kennith.com\"\r\n        },\r\n        \"landline\": \"\",\r\n        \"address\": {\r\n            \"street\": \"9 Carbine Road\",\r\n            \"street2\": \"\",\r\n            \"city\": \"Auckland\",\r\n            \"country\": \"New Zealand\",\r\n            \"po_code\": \"100\"\r\n        }\r\n    },\r\n    \"order_history\": [\r\n        {\r\n            \"id\": \"20b86d5a-9378-41ae-a01a-1da546ec5f80\",\r\n            \"destination\": {\r\n                \"code\": \"001\",\r\n                \"contact\": {\r\n                    \"name\": \"Torpedo7\",\r\n                    \"mobile\": {\r\n                        \"region_code\": \"+64\",\r\n                        \"root\": \"021212120\"\r\n                    },\r\n                    \"email\": {\r\n                        \"root\": \"order\",\r\n                        \"domain\": \"torpedo7.com\",\r\n                        \"full\": \"order@torpedo7.com\"\r\n                    },\r\n                    \"landline\": \"\",\r\n                    \"address\": {\r\n                        \"street\": \"9 Carbine Road\",\r\n                        \"street2\": \"\",\r\n                        \"city\": \"Auckland\",\r\n                        \"country\": \"New Zealand\",\r\n                        \"po_code\": \"100\"\r\n                    }\r\n                }\r\n            },\r\n            \"origin\": {\r\n                \"code\": \"002\",\r\n                \"contact\": {\r\n                    \"name\": \"Torpedo7\",\r\n                    \"mobile\": {\r\n                        \"region_code\": \"+64\",\r\n                        \"root\": \"021212120\"\r\n                    },\r\n                    \"email\": {\r\n                        \"root\": \"order\",\r\n                        \"domain\": \"torpedo7.com\",\r\n                        \"full\": \"order@torpedo7.com\"\r\n                    },\r\n                    \"landline\": \"\",\r\n                    \"address\": {\r\n                        \"street\": \"9 Carbine Road\",\r\n                        \"street2\": \"\",\r\n                        \"city\": \"Auckland\",\r\n                        \"country\": \"New Zealand\",\r\n                        \"po_code\": \"100\"\r\n                    }\r\n                }\r\n            },\r\n            \"products\": [\r\n                {\r\n                    \"product_code\": \"132522\",\r\n                    \"variant\": [\r\n                        \"22\"\r\n                    ],\r\n                    \"discount\": {\r\n                        \"Absolute\": 0\r\n                    },\r\n                    \"product_cost\": 15,\r\n                    \"quantity\": 5\r\n                },\r\n                {\r\n                    \"product_code\": \"132522\",\r\n                    \"variant\": [\r\n                        \"23\"\r\n                    ],\r\n                    \"discount\": {\r\n                        \"Absolute\": 0\r\n                    },\r\n                    \"product_cost\": 15,\r\n                    \"quantity\": 5\r\n                }\r\n            ],\r\n            \"status\": [\r\n                {\r\n                    \"status\": {\r\n                        \"Transit\": {\r\n                            \"shipping_company\": {\r\n                                \"name\": \"Torpedo7\",\r\n                                \"mobile\": {\r\n                                    \"region_code\": \"+64\",\r\n                                    \"root\": \"021212120\"\r\n                                },\r\n                                \"email\": {\r\n                                    \"root\": \"order\",\r\n                                    \"domain\": \"torpedo7.com\",\r\n                                    \"full\": \"order@torpedo7.com\"\r\n                                },\r\n                                \"landline\": \"\",\r\n                                \"address\": {\r\n                                    \"street\": \"9 Carbine Road\",\r\n                                    \"street2\": \"\",\r\n                                    \"city\": \"Auckland\",\r\n                                    \"country\": \"New Zealand\",\r\n                                    \"po_code\": \"100\"\r\n                                }\r\n                            },\r\n                            \"query_url\": \"https:www.fedex.com/fedextrack/?trknbr=\",\r\n                            \"tracking_code\": \"1523123\",\r\n                            \"assigned_products\": [\r\n                                \"132522-22\"\r\n                            ]\r\n                        }\r\n                    },\r\n                    \"assigned_products\": [\r\n                        \"132522-22\"\r\n                    ]\r\n                }\r\n            ],\r\n            \"status_history\": [\r\n                {\r\n                    \"timestamp\": \"2022-11-27T06:54:11.665676300Z\",\r\n                    \"status\": \"Queued\"\r\n                }\r\n            ],\r\n            \"order_notes\": [\r\n                {\r\n                    \"message\": \"Order Shipped from Depot\",\r\n                    \"timestamp\": \"2022-11-27T06:54:11.665644300Z\"\r\n                }\r\n            ],\r\n            \"reference\": \"TOR-19592\",\r\n            \"creation_date\": \"2022-11-27T06:54:11.665651300Z\",\r\n            \"discount\": {\r\n                \"Absolute\": 0\r\n            }\r\n        }\r\n    ],\r\n    \"customer_notes\": [],\r\n    \"balance\": 0\r\n}")
                    .send()
                    .await {
                    Ok(_) => {},
                    Err(err) => panic!("Panic: {}", err),
                }

                let duration = start.elapsed();
                println!("{}:{:?}ms", i, duration);
                durations.push(duration.as_millis());
            }
        }
        
    }).await {
        Ok(_) => {},
        Err(_) => panic!("Panic"),
    }

    // let avg_duration = durations.iter().sum::<u128>() as f32 / durations.len() as f32;
    assert_eq!("A", "A");
}