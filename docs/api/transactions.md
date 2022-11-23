The following are the possible transaction queries:

### Fetch by ID
```
GET transaction/<id>
```

Fetches the transaction by the `<id>` tag in the `Transaction` object.


<details>
    <summary>Example Request</summary>

  ### Example Request (Uses dummy data)
  ```
  GET 127.0.0.1:8000/transaction/02f5c824-892f-4838-a2a5-9628e5973a4a
  ```
     
  ### Result
  ```json
 {
    "id": "02f5c824-892f-4838-a2a5-9628e5973a4a",
    "customer": "...",
    "transaction_type": "In",
    "products": [
        {
            "id": "14e9097c-83b6-4356-9939-a77ac4597f66",
            "destination": {
                "code": "001",
                "contact": {
                    "name": "Torpedo7",
                    "mobile": {
                        "region_code": "+64",
                        "root": "021212120"
                    },
                    "email": {
                        "root": "order",
                        "domain": "torpedo7.com",
                        "full": "order@torpedo7.com"
                    },
                    "landline": "",
                    "address": {
                        "street": "9 Carbine Road",
                        "street2": "",
                        "city": "Auckland",
                        "country": "New Zealand",
                        "po_code": "100"
                    }
                }
            },
            "origin": {
                "code": "002",
                "contact": {
                    "name": "Torpedo7",
                    "mobile": {
                        "region_code": "+64",
                        "root": "021212120"
                    },
                    "email": {
                        "root": "order",
                        "domain": "torpedo7.com",
                        "full": "order@torpedo7.com"
                    },
                    "landline": "",
                    "address": {
                        "street": "9 Carbine Road",
                        "street2": "",
                        "city": "Auckland",
                        "country": "New Zealand",
                        "po_code": "100"
                    }
                }
            },
            "products": [
                {
                    "product_code": "132522",
                    "variant": [
                        "22"
                    ],
                    "discount": {
                        "Absolute": 0
                    },
                    "product_cost": 15,
                    "quantity": 5
                },
                {
                    "product_code": "132522",
                    "variant": [
                        "23"
                    ],
                    "discount": {
                        "Absolute": 0
                    },
                    "product_cost": 15,
                    "quantity": 5
                }
            ],
            "status": {
                "Transit": {
                    "shipping_company": {
                        "name": "Torpedo7",
                        "mobile": {
                            "region_code": "+64",
                            "root": "021212120"
                        },
                        "email": {
                            "root": "order",
                            "domain": "torpedo7.com",
                            "full": "order@torpedo7.com"
                        },
                        "landline": "",
                        "address": {
                            "street": "9 Carbine Road",
                            "street2": "",
                            "city": "Auckland",
                            "country": "New Zealand",
                            "po_code": "100"
                        }
                    },
                    "query_url": "https://www.fedex.com/fedextrack/?trknbr=",
                    "tracking_code": "1523123"
                }
            },
            "status_history": [
                {
                    "timestamp": "2022-11-22T07:24:42.526773900Z",
                    "status": "Queued"
                }
            ],
            "order_notes": [
                {
                    "message": "Order Shipped from Depot",
                    "timestamp": "2022-11-22T07:24:42.526635300Z"
                }
            ],
            "reference": "TOR-19592",
            "creation_date": "2022-11-22T07:24:42.526682800Z",
            "discount": {
                "Absolute": 0
            }
        }
    ],
    "order_total": 115,
    "payment": {
        "payment_method": "Card",
        "fulfillment_date": "2022-11-22T07:24:42.526777Z"
    },
    "order_date": "2022-11-22T07:24:43Z",
    "order_notes": [
        {
            "message": "Order packaged from warehouse.",
            "timestamp": "2022-11-22T07:24:42.526777600Z"
        }
    ],
    "order_history": [
        {
            "item": {
                "method_type": "Out",
                "product_code": "132522",
                "variant": [
                    "22"
                ],
                "quantity": 1
            },
            "reason": "Faulty Product",
            "timestamp": "2022-11-22T07:24:42.526778500Z"
        }
    ],
    "salesperson": "...",
    "till": "..."
}
  ```
</details>


### Fetch by Ref
```
GET transaction/ref/<ref>
```
Using the reference of an order number, `stock` will search transactions to find one which contains an order with the reference number `<ref>`, returning a list of possible transactions.

<details>
    <summary>Example Request</summary>

  ### Example Request (Uses dummy data)
  ```
  GET 127.0.0.1:8000/transaction/ref/TOR-19592
  ```
     
  ### Result
  ```json
  [
    {
        "id": "02f5c824-892f-4838-a2a5-9628e5973a4a",
        "customer": "...",
        "transaction_type": "In",
        "products": [
            {
                "id": "14e9097c-83b6-4356-9939-a77ac4597f66",
                "destination": {
                    "code": "001",
                    "contact": {
                        "name": "Torpedo7",
                        "mobile": {
                            "region_code": "+64",
                            "root": "021212120"
                        },
                        "email": {
                            "root": "order",
                            "domain": "torpedo7.com",
                            "full": "order@torpedo7.com"
                        },
                        "landline": "",
                        "address": {
                            "street": "9 Carbine Road",
                            "street2": "",
                            "city": "Auckland",
                            "country": "New Zealand",
                            "po_code": "100"
                        }
                    }
                },
                "origin": {
                    "code": "002",
                    "contact": {
                        "name": "Torpedo7",
                        "mobile": {
                            "region_code": "+64",
                            "root": "021212120"
                        },
                        "email": {
                            "root": "order",
                            "domain": "torpedo7.com",
                            "full": "order@torpedo7.com"
                        },
                        "landline": "",
                        "address": {
                            "street": "9 Carbine Road",
                            "street2": "",
                            "city": "Auckland",
                            "country": "New Zealand",
                            "po_code": "100"
                        }
                    }
                },
                "products": [
                    {
                        "product_code": "132522",
                        "variant": [
                            "22"
                        ],
                        "discount": {
                            "Absolute": 0
                        },
                        "product_cost": 15,
                        "quantity": 5
                    },
                    {
                        "product_code": "132522",
                        "variant": [
                            "23"
                        ],
                        "discount": {
                            "Absolute": 0
                        },
                        "product_cost": 15,
                        "quantity": 5
                    }
                ],
                "status": {
                    "Transit": {
                        "shipping_company": {
                            "name": "Torpedo7",
                            "mobile": {
                                "region_code": "+64",
                                "root": "021212120"
                            },
                            "email": {
                                "root": "order",
                                "domain": "torpedo7.com",
                                "full": "order@torpedo7.com"
                            },
                            "landline": "",
                            "address": {
                                "street": "9 Carbine Road",
                                "street2": "",
                                "city": "Auckland",
                                "country": "New Zealand",
                                "po_code": "100"
                            }
                        },
                        "query_url": "https://www.fedex.com/fedextrack/?trknbr=",
                        "tracking_code": "1523123"
                    }
                },
                "status_history": [
                    {
                        "timestamp": "2022-11-22T07:24:42.526773900Z",
                        "status": "Queued"
                    }
                ],
                "order_notes": [
                    {
                        "message": "Order Shipped from Depot",
                        "timestamp": "2022-11-22T07:24:42.526635300Z"
                    }
                ],
                "reference": "TOR-19592",
                "creation_date": "2022-11-22T07:24:42.526682800Z",
                "discount": {
                    "Absolute": 0
                }
            }
        ],
        "order_total": 115,
        "payment": {
            "payment_method": "Card",
            "fulfillment_date": "2022-11-22T07:24:42.526777Z"
        },
        "order_date": "2022-11-22T07:24:43Z",
        "order_notes": [
            {
                "message": "Order packaged from warehouse.",
                "timestamp": "2022-11-22T07:24:42.526777600Z"
            }
        ],
        "order_history": [
            {
                "item": {
                    "method_type": "Out",
                    "product_code": "132522",
                    "variant": [
                        "22"
                    ],
                    "quantity": 1
                },
                "reason": "Faulty Product",
                "timestamp": "2022-11-22T07:24:42.526778500Z"
            }
        ],
        "salesperson": "...",
        "till": "..."
    }
]
  ```
</details>

### Create/Insert
```
POST transaction
```
Execute a post request to `/transaction` with a `TransactionInput` object. `TransactionInput` is `Transaction` without the `id` property as `id` is generated server-side on the request. 

<details>
    <summary>Example Request</summary>

  ### Example Request (Uses dummy data)
  ```
  POST 127.0.0.1:8000/transaction
  
  {
    "customer": "",
    "transaction_type": "In",
    "products": [],
    "order_total": 0,
    "payment": {
        "payment_method": "Card",
        "fulfillment_date": "2022-11-23T01:49:58.797Z"
    },
    "order_date": "2022-11-23T01:49:58.797Z",
    "order_notes": [],
    "order_history": [],
    "salesperson": "",
    "till": ""
  }
  ```
     
  ### Result
  ```json
  {
    "id": "e9cd8339-d7f4-4a9e-9308-ff414d76bbbf",
    "customer": "",
    "transaction_type": "In",
    "products": [],
    "order_total": 0,
    "payment": {
        "payment_method": "Card",
        "fulfillment_date": "2022-11-23T01:49:58.797Z"
    },
    "order_date": "2022-11-23T01:49:59Z",
    "order_notes": [],
    "order_history": [],
    "salesperson": "",
    "till": ""
 }
  ```
</details>