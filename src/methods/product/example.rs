use crate::{
    Address, ContactInformation, DiscountValue, Email, Location, MobileNumber, Product,
    ProductIdentification, ProductVisibility, Quantity, Stock, StockInformation, Variant,
    VariantCategory, VariantInformation,
};

pub fn example_products() -> Vec<Product> {
    let mt_wellington = Location {
        store_code: "001".into(),
        store_id: "628f74d7-de00-4956-a5b6-2031e0c72128".to_string(),
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
    };

    let westfield = Location {
        store_code: "002".into(),
        store_id: "c4a1d88b-e8a0-4dcd-ade2-1eea82254816".to_string(),
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
    };

    let albany = Location {
        store_code: "003".into(),
        store_id: "a91509fa-2783-43ae-8c3c-5d5bc5cb6c95".to_string(),
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
    };

    vec![
        Product {
            name: "Explore Graphic Tee".into(),
            company: "Torpedo7".into(),
            identification: ProductIdentification::default(),
            visible: ProductVisibility::ShowWhenInStock,
            created_at: Default::default(),
            name_long: String::new(),
            description_long: String::new(),
            variant_groups: vec![
                VariantCategory {
                    category: "Colour".into(),
                    variants: vec![
                        Variant {
                            name: "White".into(),
                            images: vec![
                                "https://www.torpedo7.co.nz/images/products/T7TEO23YBHT_zoom---men-s-ecopulse-short-sleeve-explore-graphic-t-shirt-blanc-du-blanc.jpg?v=845eb9a5288642009c05".into(),
                            ],
                            marginal_price: 550.00,
                            variant_code: "01".into(),
                            order_history: vec![],
                        },
                        Variant {
                            name: "Black".into(),
                            images: vec![
                                "https://www.torpedo7.co.nz/images/products/T7TEO23YEAA_zoom---men-s-ecopulse-short-sleeve-explore-graphic-t-shirt-black.jpg?v=845eb9a5288642009c05".into(),
                            ],
                            marginal_price: 550.00,
                            variant_code: "02".into(),
                            order_history: vec![],
                        },
                        Variant {
                            name: "Hot Sauce".into(),
                            images: vec![
                                "https://www.torpedo7.co.nz/images/products/T7TEO23YDHS_zoom---men-s-ecopulse-short-sleeve-explore-graphic-t-shirts-hot-sauce.jpg?v=845eb9a5288642009c05".into(),
                            ],
                            marginal_price: 550.00,
                            variant_code: "03".into(),
                            order_history: vec![],
                        },
                        Variant {
                            name: "Tourmaline".into(),
                            images: vec![
                                "https://www.torpedo7.co.nz/images/products/T7TEO23YCJZ_zoom---men-s-ecopulse-short-sleeve-explore-graphic-t-shirt-tourmaline.jpg?v=845eb9a5288642009c05".into(),
                            ],
                            marginal_price: 550.00,
                            variant_code: "04".into(),
                            order_history: vec![],
                        },
                        Variant {
                            name: "Navy Blazer".into(),
                            images: vec![
                                "https://www.torpedo7.co.nz/images/products/T7TEO23YIWJ_zoom---men-s-ecopulse-short-sleeve-organic-chest-print-t-shirt-navy-blazer.jpg?v=845eb9a5288642009c05".into(),
                            ],
                            marginal_price: 550.00,
                            variant_code: "05".into(),
                            order_history: vec![],
                        }
                    ]
                },
                VariantCategory {
                    category: "Size".into(),
                    variants: vec![
                        Variant {
                            name: "Small".into(),
                            images: vec![
                                "https://www.torpedo7.co.nz/images/products/T7TEO23YIWJ_zoom---men-s-ecopulse-short-sleeve-organic-chest-print-t-shirt-navy-blazer.jpg?v=845eb9a5288642009c05".into()
                            ],
                            marginal_price: 550.00,
                            variant_code: "21".into(),
                            order_history: vec![],
                        },
                        Variant {
                            name: "Medium".into(),
                            images: vec![
                                "https://www.torpedo7.co.nz/images/products/T7TEO23YIWJ_zoom---men-s-ecopulse-short-sleeve-organic-chest-print-t-shirt-navy-blazer.jpg?v=845eb9a5288642009c05".into(),
                            ],
                            marginal_price: 550.00,
                            variant_code: "22".into(),
                            order_history: vec![],
                        },
                        Variant {
                            name: "Large".into(),
                            images: vec![
                                "https://www.torpedo7.co.nz/images/products/T7TEO23YIWJ_zoom---men-s-ecopulse-short-sleeve-organic-chest-print-t-shirt-navy-blazer.jpg?v=845eb9a5288642009c05".into(),
                            ],
                            marginal_price: 550.00,
                            variant_code: "23".into(),
                            order_history: vec![],
                        },
                        Variant {
                            name: "Extra Large".into(),
                            images: vec![
                                "https://www.torpedo7.co.nz/images/products/T7TEO23YIWJ_zoom---men-s-ecopulse-short-sleeve-organic-chest-print-t-shirt-navy-blazer.jpg?v=845eb9a5288642009c05".into()
                            ],
                            marginal_price: 550.00,
                            variant_code: "24".into(),
                            order_history: vec![],
                        }
                    ]
                }
            ],
            variants: vec![
                VariantInformation {
                    id: "SM-BLK-ITM".to_string(),
                    name: "Small Black".into(),
                    buy_min: 1.0,
                    buy_max: -1.0,
                    identification: ProductIdentification::default(),
                    stock_tracking: true,
                    stock: vec![
                        Stock {
                            store: mt_wellington.clone(),
                            quantity: Quantity {
                                quantity_sellable: 0.0,
                                quantity_on_order: 0.0,
                                quantity_unsellable: 0.0,
                                quantity_allocated: 0.0
                            }
                        },
                        Stock {
                            store: westfield.clone(),
                            quantity: Quantity {
                                quantity_sellable: 4.0,
                                quantity_on_order: 2.0,
                                quantity_unsellable: 2.0,
                                quantity_allocated: 0.0
                            }
                        },
                        Stock {
                            store: albany.clone(),
                            quantity: Quantity {
                                quantity_sellable: 1.0,
                                quantity_on_order: 0.0,
                                quantity_unsellable: 2.0,
                                quantity_allocated: 0.0
                            }
                        },
                    ],
                    images: vec![
                        "https://www.torpedo7.co.nz/images/products/T7TEO23YEAA_zoom---men-s-ecopulse-short-sleeve-explore-graphic-t-shirt-black.jpg?v=845eb9a5288642009c05".into()
                    ],
                    marginal_price: 10.99,
                    retail_price: 44.99,
                    variant_code: vec!["02".into(), "21".into()],
                    order_history: vec![],
                    barcode: "51890723908812".into(),
                    stock_information: StockInformation {
                        stock_group: "RANDOM".into(),
                        sales_group: "RANDOM".into(),
                        value_stream: "RANDOM".into(),
                        brand: "SELLER_GROUP".into(),
                        tax_code: "GSL".into(),
                        weight: "5.6".into(),
                        volume: "0.123".into(),
                        max_volume: "6.00".into(),
                        back_order: false,
                        discontinued: false,
                        non_diminishing: false,
                        shippable: true,
                        min_stock_before_alert: 2.0,
                        min_stock_level: 0.0,
                        colli: String::new(),
                        size_x: 0.0,
                        size_y: 0.0,
                        size_z: 0.0,
                        size_x_unit: "m".to_string(),
                        size_y_unit: "m".to_string(),
                        size_z_unit: "m".to_string(),
                        size_override_unit: "m".to_string()
                    },
                    loyalty_discount: DiscountValue::Absolute(15)
                },
                VariantInformation {
                    id: "M-BLK-ITM".to_string(),
                    name: "Medium Black".into(),
                    buy_min: 1.0,
                    buy_max: -1.0,
                    identification: ProductIdentification::default(),
                    stock_tracking: true,
                    stock: vec![
                        Stock {
                            store: mt_wellington.clone(),
                            quantity: Quantity {
                                quantity_sellable: 7.0,
                                quantity_unsellable: 2.0,
                                quantity_on_order: 4.0,
                                quantity_allocated: 0.0
                            }
                        },
                        Stock {
                            store: westfield.clone(),
                            quantity: Quantity {
                                quantity_sellable: 0.0,
                                quantity_on_order: 1.0,
                                quantity_unsellable: 0.0,
                                quantity_allocated: 0.0
                            }
                        },
                        Stock {
                            store: albany.clone(),
                            quantity: Quantity {
                                quantity_sellable: 1.0,
                                quantity_on_order: 0.0,
                                quantity_unsellable: 2.0,
                                quantity_allocated: 0.0
                            }
                        },
                    ],
                    images: vec![
                        "https://www.torpedo7.co.nz/images/products/T7TEO23YEAA_zoom---men-s-ecopulse-short-sleeve-explore-graphic-t-shirt-black.jpg?v=845eb9a5288642009c05".into()
                    ],
                    marginal_price: 12.49,
                    retail_price: 46.99,
                    variant_code: vec!["02".into(), "22".into()],
                    order_history: vec![],
                    barcode: "51150723152813".into(),
                    stock_information: StockInformation {
                        stock_group: "RANDOM".into(),
                        sales_group: "RANDOM".into(),
                        value_stream: "RANDOM".into(),
                        brand: "SELLER_GROUP".into(),
                        tax_code: "GSL".into(),
                        weight: "5.6".into(),
                        volume: "0.123".into(),
                        max_volume: "6.00".into(),
                        back_order: false,
                        discontinued: false,
                        non_diminishing: false,
                        shippable: true,
                        min_stock_before_alert: 2.0,
                        min_stock_level: 0.0,
                        colli: String::new(),
                        size_x: 0.0,
                        size_y: 0.0,
                        size_z: 0.0,
                        size_x_unit: "m".to_string(),
                        size_y_unit: "m".to_string(),
                        size_z_unit: "m".to_string(),
                        size_override_unit: "m".to_string()
                    },
                    loyalty_discount: DiscountValue::Absolute(25)
                },
                VariantInformation {
                    id: "LG-WHT-ITM".to_string(),
                    name: "Large White".into(),
                    buy_min: 1.0,
                    buy_max: -1.0,
                    identification: ProductIdentification::default(),
                    stock_tracking: true,
                    stock: vec![
                        Stock {
                            store: mt_wellington.clone(),
                            quantity: Quantity {
                                quantity_sellable: 1.0,
                                quantity_on_order: 0.0,
                                quantity_unsellable: 2.0,
                                quantity_allocated: 0.0
                            }
                        },
                        Stock {
                            store: westfield.clone(),
                            quantity: Quantity {
                                quantity_sellable: 3.0,
                                quantity_on_order: 0.0,
                                quantity_unsellable: 1.0,
                                quantity_allocated: 0.0
                            }
                        },
                        Stock {
                            store: albany.clone(),
                            quantity: Quantity {
                                quantity_sellable: 1.0,
                                quantity_on_order: 0.0,
                                quantity_unsellable: 2.0,
                                quantity_allocated: 0.0
                            }
                        },
                    ],
                    images: vec![
                        "https://www.torpedo7.co.nz/images/products/T7TEO23YBHT_zoom---men-s-ecopulse-short-sleeve-explore-graphic-t-shirt-blanc-du-blanc.jpg?v=845eb9a5288642009c05".into()
                    ],
                    variant_code: vec!["01".into(), "23".into()],
                    order_history: vec![],
                    marginal_price: 16.09,
                    retail_price: 49.99,
                    barcode: "51150723159173".into(),
                    stock_information: StockInformation {
                        stock_group: "RANDOM".into(),
                        sales_group: "RANDOM".into(),
                        value_stream: "RANDOM".into(),
                        brand: "SELLER_GROUP".into(),
                        tax_code: "GSL".into(),
                        weight: "5.6".into(),
                        volume: "0.123".into(),
                        max_volume: "6.00".into(),
                        back_order: false,
                        discontinued: false,
                        non_diminishing: false,
                        shippable: true,
                        min_stock_before_alert: 2.0,
                        min_stock_level: 0.0,
                        colli: String::new(),
                        size_x: 0.0,
                        size_y: 0.0,
                        size_z: 0.0,
                        size_x_unit: "m".to_string(),
                        size_y_unit: "m".to_string(),
                        size_z_unit: "m".to_string(),
                        size_override_unit: "m".to_string()
                    },
                    loyalty_discount: DiscountValue::Absolute(5)
                },
            ],
            sku: 123456.to_string(),
            images: vec![
                "https://www.torpedo7.co.nz/images/products/T7TEOQR5NDD_zoom---men-s-short-sleeve-explore-graphic-tee-ochre-rose.jpg".into()
            ],
            tags: vec![
                "Tee".into(),
                "Cotton".into(),
                "Organic".into()
            ],
            description: "Made with organically grown cotton to reflect our love of the planet and the people on it.".into(),
            specifications: vec![
                ("".into(), "Soft cotton tee".into()),
                ("".into(), "100% Organically Grown Cotton. Uses Less Water. No pesticides used on crops. Supports Regenerative Agriculture".into()),
                ("".into(), "Composition: 100% Organic cotton".into())
            ],
            updated_at: Default::default(),
        },
        Product {
            name: "Nippers Kids Kayak & Paddle".into(),
            company: "Torpedo7".into(),
            identification: ProductIdentification::default(),
            visible: ProductVisibility::ShowWhenInStock,
            created_at: Default::default(),
            name_long: String::new(),
            description_long: String::new(),
            variant_groups: vec![
                VariantCategory {
                    category: "Colour".into(),
                    variants: vec![
                        Variant {
                            name: "Beaches".into(),
                            images: vec![
                                "https://www.torpedo7.co.nz/images/products/T7KKK23NB0Z_zoom---2023-nippers-kids-kayak---paddle-1-83m-beaches.jpg?v=99f4b292748848b5b1d6".into(),
                            ],
                            marginal_price: 399.99,
                            variant_code: "01".into(),
                            order_history: vec![],
                        },
                        Variant {
                            name: "Tropics".into(),
                            images: vec![
                                "https://www.torpedo7.co.nz/images/products/T7KKK23NTOY_zoom---2023-nippers-kids-kayak---paddle-1-83m-tropics.jpg?v=99f4b292748848b5b1d6".into(),
                            ],
                            marginal_price: 399.99,
                            variant_code: "02".into(),
                            order_history: vec![],
                        }
                    ]
                },
                VariantCategory {
                    category: "Size".into(),
                    variants: vec![
                        Variant {
                            name: "1.83m".into(),
                            images: vec![
                                "https://www.torpedo7.co.nz/images/products/T7KKK23NTOY_zoom---2023-nippers-kids-kayak---paddle-1-83m-tropics.jpg?v=99f4b292748848b5b1d6".into()
                            ],
                            marginal_price: 399.99,
                            variant_code: "21".into(),
                            order_history: vec![],
                        }
                    ]
                }
            ],
            variants: vec![
                VariantInformation {
                    id: "1.83-BEACHES".to_string(),
                    name: "1.83m Beaches".into(),
                    buy_min: 1.0,
                    buy_max: -1.0,
                    identification: ProductIdentification::default(),
                    stock_tracking: true,
                    stock: vec![
                        Stock {
                            store: mt_wellington.clone(),
                            quantity: Quantity {
                                quantity_sellable: 4.0,
                                quantity_on_order: 0.0,
                                quantity_unsellable: 2.0,
                                quantity_allocated: 0.0
                            }
                        },
                        Stock {
                            store: westfield.clone(),
                            quantity: Quantity {
                                quantity_sellable: 1.0,
                                quantity_on_order: 1.0,
                                quantity_unsellable: 0.0,
                                quantity_allocated: 0.0
                            }
                        },
                        Stock {
                            store: albany.clone(),
                            quantity: Quantity {
                                quantity_sellable: 0.0,
                                quantity_on_order: 2.0,
                                quantity_unsellable: 1.0,
                                quantity_allocated: 0.0
                            }
                        },
                    ],
                    images: vec![
                        "https://www.torpedo7.co.nz/images/products/T7KKK23NB0Z_zoom---2023-nippers-kids-kayak---paddle-1-83m-beaches.jpg".into()
                    ],
                    marginal_price: 85.99,
                    retail_price: 399.99,
                    variant_code: vec!["01".into(), "21".into()],
                    order_history: vec![],
                    barcode: "51891743988214".into(),
                    stock_information: StockInformation {
                        stock_group: "RANDOM".into(),
                        sales_group: "RANDOM".into(),
                        value_stream: "RANDOM".into(),
                        brand: "SELLER_GROUP".into(),
                        tax_code: "GSL".into(),
                        weight: "5.6".into(),
                        volume: "0.123".into(),
                        max_volume: "6.00".into(),
                        back_order: false,
                        discontinued: false,
                        non_diminishing: false,
                        shippable: true,
                        min_stock_before_alert: 2.0,
                        min_stock_level: 0.0,
                        colli: String::new(),
                        size_x: 0.0,
                        size_y: 0.0,
                        size_z: 0.0,
                        size_x_unit: "m".to_string(),
                        size_y_unit: "m".to_string(),
                        size_z_unit: "m".to_string(),
                        size_override_unit: "m".to_string()
                    },
                    loyalty_discount: DiscountValue::Absolute(15)
                },
                VariantInformation {
                    id: "1.83-TROPICS".to_string(),
                    name: "1.83m Tropics".into(),
                    buy_min: 1.0,
                    buy_max: -1.0,
                    identification: ProductIdentification::default(),
                    stock_tracking: true,
                    stock: vec![
                        Stock {
                            store: mt_wellington.clone(),
                            quantity: Quantity {
                                quantity_sellable: 7.0,
                                quantity_unsellable: 2.0,
                                quantity_on_order: 4.0,
                                quantity_allocated: 0.0
                            }
                        },
                        Stock {
                            store: westfield.clone(),
                            quantity: Quantity {
                                quantity_sellable: 0.0,
                                quantity_on_order: 1.0,
                                quantity_unsellable: 0.0,
                                quantity_allocated: 0.0
                            }
                        },
                        Stock {
                            store: albany.clone(),
                            quantity: Quantity {
                                quantity_sellable: 1.0,
                                quantity_on_order: 0.0,
                                quantity_unsellable: 2.0,
                                quantity_allocated: 0.0
                            }
                        },
                    ],
                    images: vec![
                        "https://www.torpedo7.co.nz/images/products/T7KKK23NB0Z_zoom---2023-nippers-kids-kayak---paddle-1-83m-beaches.jpg".into()
                    ],
                    marginal_price: 85.99,
                    retail_price: 399.99,
                    variant_code: vec!["02".into(), "21".into()],
                    order_history: vec![],
                    barcode: "54897443288214".into(),
                    stock_information: StockInformation {
                        stock_group: "RANDOM".into(),
                        sales_group: "RANDOM".into(),
                        value_stream: "RANDOM".into(),
                        brand: "SELLER_GROUP".into(),
                        tax_code: "GSL".into(),
                        weight: "5.6".into(),
                        volume: "0.123".into(),
                        max_volume: "6.00".into(),
                        back_order: false,
                        discontinued: false,
                        non_diminishing: false,
                        shippable: true,
                        min_stock_before_alert: 2.0,
                        min_stock_level: 0.0,
                        colli: String::new(),
                        size_x: 0.0,
                        size_y: 0.0,
                        size_z: 0.0,
                        size_x_unit: "m".to_string(),
                        size_y_unit: "m".to_string(),
                        size_z_unit: "m".to_string(),
                        size_override_unit: "m".to_string()
                    },
                    loyalty_discount: DiscountValue::Absolute(25)
                },
            ],
            sku: 654321.to_string(),
            images: vec![
                "https://www.torpedo7.co.nz/images/products/T7KKK23NB0Z_zoom---2023-nippers-kids-kayak---paddle-1-83m-beaches.jpg".into()
            ],
            tags: vec![
                "Kayak".into(),
                "Kids".into(),
                "Recreational".into(),
                "Water".into()
            ],
            description: "The Nippers Kayak is the ideal size for kid's wanting their own independence. The compact, lightweight design with a V shaped hull and centre fin allows the kayak to track and manoeuver easily. They’ll also enjoy the multiple foot holds to accommodate various heights and a lightweight, flexible paddle so they can push through water without too much resistance. With the Nippers Kayak from Torpedo7, there are no excuses for a bad day with mum and dad.".into(),
            specifications: vec![
                ("Length".into(), "183cm".into()),
                ("Width".into(), "70cm".into()),
                ("Height".into(), "23cm".into()),
                ("Gross Weight".into(), "9kg".into()),
                ("Weight Capacity".into(), "50kg".into())
            ],
            updated_at: Default::default(),
        },
        Product {
            name: "Kids Voyager II Paddle Vest".into(),
            company: "Torpedo7".into(),
            identification: ProductIdentification::default(),
            visible: ProductVisibility::ShowWhenInStock,
            created_at: Default::default(),
            name_long: String::new(),
            description_long: String::new(),
            variant_groups: vec![
                VariantCategory {
                    category: "Colour".into(),
                    variants: vec![
                        Variant {
                            name: "Red".into(),
                            images: vec![
                                "https://www.torpedo7.co.nz/images/products/T7LJJ22DFRA_zoom---kids-voyager-ii-paddle-vest-red.jpg?v=99f4b292748848b5b1d6".into(),
                            ],
                            marginal_price: 139.99,
                            variant_code: "01".into(),
                            order_history: vec![],
                        }
                    ]
                },
                VariantCategory {
                    category: "Size".into(),
                    variants: vec![
                        Variant {
                            name: "4-6".into(),
                            images: vec![
                                "https://www.torpedo7.co.nz/images/products/T7LJJ22DFRA_zoom---kids-voyager-ii-paddle-vest-red.jpg?v=99f4b292748848b5b1d6".into()
                            ],
                            marginal_price: 139.99,
                            variant_code: "21".into(),
                            order_history: vec![],
                        },
                        Variant {
                            name: "8-10".into(),
                            images: vec![
                                "https://www.torpedo7.co.nz/images/products/T7LJJ22DFRA_zoom---kids-voyager-ii-paddle-vest-red.jpg?v=99f4b292748848b5b1d6".into()
                            ],
                            marginal_price: 139.99,
                            variant_code: "22".into(),
                            order_history: vec![],
                        },
                        Variant {
                            name: "12-14".into(),
                            images: vec![
                                "https://www.torpedo7.co.nz/images/products/T7LJJ22DFRA_zoom---kids-voyager-ii-paddle-vest-red.jpg?v=99f4b292748848b5b1d6".into()
                            ],
                            marginal_price: 139.99,
                            variant_code: "23".into(),
                            order_history: vec![],
                        },
                    ]
                }
            ],
            variants: vec![
                VariantInformation {
                    id: "S-RED".to_string(),
                    name: "Small Red (4-6y)".into(),
                    buy_min: 1.0,
                    buy_max: -1.0,
                    identification: ProductIdentification::default(),
                    stock_tracking: true,
                    stock: vec![
                        Stock {
                            store: mt_wellington.clone(),
                            quantity: Quantity {
                                quantity_sellable: 4.0,
                                quantity_on_order: 0.0,
                                quantity_unsellable: 2.0,
                                quantity_allocated: 0.0
                            }
                        },
                        Stock {
                            store: westfield.clone(),
                            quantity: Quantity {
                                quantity_sellable: 1.0,
                                quantity_on_order: 1.0,
                                quantity_unsellable: 0.0,
                                quantity_allocated: 0.0
                            }
                        },
                        Stock {
                            store: albany.clone(),
                            quantity: Quantity {
                                quantity_sellable: 0.0,
                                quantity_on_order: 2.0,
                                quantity_unsellable: 1.0,
                                quantity_allocated: 0.0
                            }
                        },
                    ],
                    images: vec![
                        "https://www.torpedo7.co.nz/images/products/T7LJJ22DFRA_zoom---kids-voyager-ii-paddle-vest-red.jpg?v=99f4b292748848b5b1d6".into()
                    ],
                    marginal_price: 45.99,
                    retail_price: 139.99,
                    variant_code: vec!["01".into(), "21".into()],
                    order_history: vec![],
                    barcode: "51891265958214".into(),
                    stock_information: StockInformation {
                        stock_group: "RANDOM".into(),
                        sales_group: "RANDOM".into(),
                        value_stream: "RANDOM".into(),
                        brand: "SELLER_GROUP".into(),
                        tax_code: "GSL".into(),
                        weight: "5.6".into(),
                        volume: "0.123".into(),
                        max_volume: "6.00".into(),
                        back_order: false,
                        discontinued: false,
                        non_diminishing: false,
                        shippable: true,
                        min_stock_before_alert: 2.0,
                        min_stock_level: 0.0,
                        colli: String::new(),
                        size_x: 0.0,
                        size_y: 0.0,
                        size_z: 0.0,
                        size_x_unit: "m".to_string(),
                        size_y_unit: "m".to_string(),
                        size_z_unit: "m".to_string(),
                        size_override_unit: "m".to_string()
                    },
                    loyalty_discount: DiscountValue::Absolute(15)
                },
                VariantInformation {
                    id: "M-RED".to_string(),
                    name: "Medium Red (8-10y)".into(),
                    buy_min: 1.0,
                    buy_max: -1.0,
                    identification: ProductIdentification::default(),
                    stock_tracking: true,
                    stock: vec![
                        Stock {
                            store: mt_wellington.clone(),
                            quantity: Quantity {
                                quantity_sellable: 4.0,
                                quantity_on_order: 0.0,
                                quantity_unsellable: 2.0,
                                quantity_allocated: 0.0
                            }
                        },
                        Stock {
                            store: westfield.clone(),
                            quantity: Quantity {
                                quantity_sellable: 1.0,
                                quantity_on_order: 1.0,
                                quantity_unsellable: 0.0,
                                quantity_allocated: 0.0
                            }
                        },
                        Stock {
                            store: albany.clone(),
                            quantity: Quantity {
                                quantity_sellable: 0.0,
                                quantity_on_order: 2.0,
                                quantity_unsellable: 1.0,
                                quantity_allocated: 0.0
                            }
                        },
                    ],
                    images: vec![
                        "https://www.torpedo7.co.nz/images/products/T7LJJ22DFRA_zoom---kids-voyager-ii-paddle-vest-red.jpg?v=99f4b292748848b5b1d6".into()
                    ],
                    marginal_price: 45.99,
                    retail_price: 139.99,
                    variant_code: vec!["01".into(), "22".into()],
                    order_history: vec![],
                    barcode: "51893261953216".into(),
                    stock_information: StockInformation {
                        stock_group: "RANDOM".into(),
                        sales_group: "RANDOM".into(),
                        value_stream: "RANDOM".into(),
                        brand: "SELLER_GROUP".into(),
                        tax_code: "GSL".into(),
                        weight: "5.6".into(),
                        volume: "0.123".into(),
                        max_volume: "6.00".into(),
                        back_order: false,
                        discontinued: false,
                        non_diminishing: false,
                        shippable: true,
                        min_stock_before_alert: 2.0,
                        min_stock_level: 0.0,
                        colli: String::new(),
                        size_x: 0.0,
                        size_y: 0.0,
                        size_z: 0.0,
                        size_x_unit: "m".to_string(),
                        size_y_unit: "m".to_string(),
                        size_z_unit: "m".to_string(),
                        size_override_unit: "m".to_string()
                    },
                    loyalty_discount: DiscountValue::Absolute(15)
                },
                VariantInformation {
                    id: "L-RED".to_string(),
                    name: "Large Red (12-14y)".into(),
                    buy_min: 1.0,
                    buy_max: -1.0,
                    identification: ProductIdentification::default(),
                    stock_tracking: true,
                    stock: vec![
                        Stock {
                            store: mt_wellington,
                            quantity: Quantity {
                                quantity_sellable: 4.0,
                                quantity_on_order: 0.0,
                                quantity_unsellable: 2.0,
                                quantity_allocated: 0.0
                            }
                        },
                        Stock {
                            store: westfield,
                            quantity: Quantity {
                                quantity_sellable: 1.0,
                                quantity_on_order: 1.0,
                                quantity_unsellable: 0.0,
                                quantity_allocated: 0.0
                            }
                        },
                        Stock {
                            store: albany,
                            quantity: Quantity {
                                quantity_sellable: 0.0,
                                quantity_on_order: 2.0,
                                quantity_unsellable: 1.0,
                                quantity_allocated: 0.0
                            }
                        },
                    ],
                    images: vec![
                        "https://www.torpedo7.co.nz/images/products/T7LJJ22DFRA_zoom---kids-voyager-ii-paddle-vest-red.jpg?v=99f4b292748848b5b1d6".into()
                    ],
                    marginal_price: 45.99,
                    retail_price: 139.99,
                    variant_code: vec!["01".into(), "23".into()],
                    order_history: vec![],
                    barcode: "52496265958214".into(),
                    stock_information: StockInformation {
                        stock_group: "RANDOM".into(),
                        sales_group: "RANDOM".into(),
                        value_stream: "RANDOM".into(),
                        brand: "SELLER_GROUP".into(),
                        tax_code: "GSL".into(),
                        weight: "5.6".into(),
                        volume: "0.123".into(),
                        max_volume: "6.00".into(),
                        back_order: false,
                        discontinued: false,
                        non_diminishing: false,
                        shippable: true,
                        min_stock_before_alert: 2.0,
                        min_stock_level: 0.0,
                        colli: String::new(),
                        size_x: 0.0,
                        size_y: 0.0,
                        size_z: 0.0,
                        size_x_unit: "m".to_string(),
                        size_y_unit: "m".to_string(),
                        size_z_unit: "m".to_string(),
                        size_override_unit: "m".to_string()
                    },
                    loyalty_discount: DiscountValue::Absolute(15)
                },
            ],
            sku: 162534.to_string(),
            images: vec![
                "https://www.torpedo7.co.nz/images/products/T7LJJ22DFRA_zoom---kids-voyager-ii-paddle-vest-red.jpg".into()
            ],
            tags: vec![
                "Kayak".into(),
                "Kids".into(),
                "Recreational".into(),
                "Water".into()
            ],
            description: "The Nippers Kayak is the ideal size for kid's wanting their own independence.  The compact, lightweight design with a V shaped hull and centre fin allows the kayak to track and manoeuvre easily. They’ll also enjoy the multiple foot holds to accommodate various heights and a lightweight, flexible paddle so they can push through water without too much resistance. With the Nippers Kayak from Torpedo7, there are no excuses for a bad day with mum and dad.".into(),
            specifications: vec![
                ("Length".into(), "183cm".into()),
                ("Width".into(), "70cm".into()),
                ("Height".into(), "23cm".into()),
                ("Gross Weight".into(), "9kg".into()),
                ("Weight Capacity".into(), "50kg".into())
            ],
            updated_at: Default::default(),
        }
    ]
}
