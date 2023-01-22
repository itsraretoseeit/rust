// ----------------TOML Section

// this is dynamic approach

// fn main() {
//     let config_const_values = {
//         let config_path = std::env::args().nth(1).unwrap();

//         let config_text = std::fs::read_to_string(&config_path).unwrap();

//         config_text.parse::<toml::Value>().unwrap()
//     };

//     println!("Original: {:#?}", config_const_values);

//     println!(
//         "[Postgresql].Database: {}",
//         config_const_values
//             .get("postgresql")
//             .unwrap()
//             .get("database")
//             .unwrap()
//             .as_str()
//             .unwrap()
//     );
// }


// static approach

// use serde_derive::Deserialize;

// #[allow(unused)]
// #[derive(Deserialize)]
// struct Input {
//     xml_file: String,
//     json_file: String,
// }

// #[allow(unused)]
// #[derive(Deserialize)]
// struct Redis {
//     host: String,
// }

// #[allow(unused)]
// #[derive(Deserialize)]
// struct Sqlite {
//     db_file: String,
// }

// #[allow(unused)]
// #[derive(Deserialize)]
// struct Postgresql {
//     username: String,
//     password: String,
//     host: String,
//     port: String,
//     database: String,
// }

// #[allow(unused)]
// #[derive(Deserialize)]
// struct Config {
//     input: Input,
//     redis: Redis,
//     sqlite: Sqlite,
//     postgresql: Postgresql,
// }

// fn main() {
//     let config_const_values: Config = {
//         let config_path = std::env::args().nth(1).unwrap();

//         let config_text = std::fs::read_to_string(&config_path).unwrap();

//         toml::from_str(&config_text).unwrap()
//     };

//     println!(
//         "[postgresql].database: {}",
//         config_const_values.postgresql.database
//     )
// }

//-----------------------JSON SECTION
//--- dynamic
// use serde_json::{Number, Value};

// fn main() {
//     let input_path = std::env::args().nth(1).unwrap();
//     let output_path = std::env::args().nth(2).unwrap();

//     let mut sales_and_products = {
//         let sales_and_products_text = std::fs::read_to_string(&input_path).unwrap();

//         serde_json::from_str::<Value>(&sales_and_products_text).unwrap()
//     };

//     if let Value::Number(n) = &sales_and_products["sales"][1]["quantity"] {
//         sales_and_products["sales"][1]["quantity"] = 
//             Value::Number(Number::from_f64(n.as_f64().unwrap() + 1.5).unwrap());
//     }

//     std::fs::write(
//         output_path,
//         serde_json::to_string_pretty(&sales_and_products).unwrap(),
//     )
//     .unwrap();

//     println!("{}", "Done!")
// }

//--- static

// use serde_derive::{Deserialize, Serialize};

// #[derive(Deserialize, Serialize, Debug)]
// struct Product {
//     id: i32,
//     category: String,
//     name: String,
// }

// #[derive(Deserialize, Serialize, Debug)]
// struct Sale {
//     id: String,
//     product_id: u32,
//     date: i64,
//     quantity: f64,
//     unit: String,
// }

// #[derive(Deserialize, Serialize, Debug)]
// struct SalesAndProducts {
//     products: Vec<Product>,
//     sales: Vec<Sale>,
// }

// fn main() -> Result<(), std::io::Error> {
//     let input_path = std::env::args().nth(1).unwrap();
//     let output_path = std::env::args().nth(2).unwrap();

//     let mut sales_and_products = {
//         let sales_and_products_text = std::fs::read_to_string(&input_path)?;

//         serde_json::from_str::<SalesAndProducts>(&sales_and_products_text).unwrap()
//     };

//     sales_and_products.sales[1].quantity += 1.5;

//     std::fs::write(
//         output_path,
//         serde_json::to_string_pretty(&sales_and_products).unwrap(),
//     )?;
//     Ok(())
// }

//------------------XML

use xml::reader::{EventReader, XmlEvent};

#[derive(Debug, Default)]
struct Product {
    id: u32,
    category: String,
    name: String,
}

#[derive(Debug, Default)]
struct Sale {
    id: String,
    product_id: u32,
    date: i64,
    quantity: f64,
    unit: String,
}

enum LocationItem {
    Other,
    InProduct,
    InSale,
}

enum LocationProduct {
    Other,
    InId,
    InCategory,
    InName,
}

enum LocationSale {
    Other,
    InId,
    InProductId,
    InDate,
    InQuantity,
    InUnit,
}

fn main() {
    let mut location_item = LocationItem::Other;
    let mut location_product = LocationProduct::Other;
    let mut location_sale = LocationSale::Other;
    let pathname = std::env::args().nth(1).unwrap();
    let mut product: Product = Default::default();
    let mut sale: Sale = Default::default();
    let file = std::fs::File::open(pathname).unwrap();
    let file = std::io::BufReader::new(file);
    let parser = EventReader::new(file);
    for event in parser {
        match &location_item {
            LocationItem::Other => match event {
                Ok(XmlEvent::StartElement { ref name, .. }) if name.local_name == "product" => {
                    location_item = LocationItem::InProduct;
                    location_product = LocationProduct::Other;
                    product = Default::default();
                }
                Ok(XmlEvent::StartElement { ref name, .. }) if name.local_name == "sale" => {
                    location_item = LocationItem::InSale;
                    location_sale = LocationSale::Other;
                    sale = Default::default();
                }
                _ => {}
            },
            LocationItem::InProduct => match &location_product {
                LocationProduct::Other => match event {
                    Ok(XmlEvent::StartElement { ref name, .. }) if name.local_name == "id" => {
                        location_product = LocationProduct::InId;
                    }
                    Ok(XmlEvent::StartElement { ref name, .. })
                        if name.local_name == "category" =>
                    {
                        location_product = LocationProduct::InCategory;
                    }
                    Ok(XmlEvent::StartElement { ref name, .. }) if name.local_name == "name" => {
                        location_product = LocationProduct::InName;
                    }
                    Ok(XmlEvent::EndElement { .. }) => {
                        location_item = LocationItem::Other;
                        println!("  Exit product: {:?}", product);
                    }
                    _ => {}
                },
                LocationProduct::InId => match event {
                    Ok(XmlEvent::Characters(characters)) => {
                        product.id = characters.parse::<u32>().unwrap();
                        println!("Got product.id: {}.", characters);
                    }
                    Ok(XmlEvent::EndElement { .. }) => {
                        location_product = LocationProduct::Other;
                    }
                    _ => {}
                },
                LocationProduct::InCategory => match event {
                    Ok(XmlEvent::Characters(characters)) => {
                        product.category = characters.clone();
                        println!("Got product.category: {}.", characters);
                    }
                    Ok(XmlEvent::EndElement { .. }) => {
                        location_product = LocationProduct::Other;
                    }
                    _ => {}
                },
                LocationProduct::InName => match event {
                    Ok(XmlEvent::Characters(characters)) => {
                        product.name = characters.clone();
                        println!("Got product.name: {}.", characters);
                    }
                    Ok(XmlEvent::EndElement { .. }) => {
                        location_product = LocationProduct::Other;
                    }
                    _ => {}
                },
            },
            LocationItem::InSale => match &location_sale {
                LocationSale::Other => match event {
                    Ok(XmlEvent::StartElement { ref name, .. }) if name.local_name == "id" => {
                        location_sale = LocationSale::InId;
                    }
                    Ok(XmlEvent::StartElement { ref name, .. })
                        if name.local_name == "product-id" =>
                    {
                        location_sale = LocationSale::InProductId;
                    }
                    Ok(XmlEvent::StartElement { ref name, .. }) if name.local_name == "date" => {
                        location_sale = LocationSale::InDate;
                    }
                    Ok(XmlEvent::StartElement { ref name, .. })
                        if name.local_name == "quantity" =>
                    {
                        location_sale = LocationSale::InQuantity;
                    }
                    Ok(XmlEvent::StartElement { ref name, .. }) if name.local_name == "unit" => {
                        location_sale = LocationSale::InUnit;
                    }
                    Ok(XmlEvent::EndElement { ref name, .. }) if name.local_name == "sale" => {
                        location_item = LocationItem::Other;
                        println!("  Exit sale: {:?}", sale);
                    }
                    _ => {}
                },
                LocationSale::InId => match event {
                    Ok(XmlEvent::Characters(characters)) => {
                        sale.id = characters.clone();
                        println!("Got sale.id: {}.", characters);
                    }
                    Ok(XmlEvent::EndElement { .. }) => {
                        location_sale = LocationSale::Other;
                    }
                    _ => {}
                },
                LocationSale::InProductId => match event {
                    Ok(XmlEvent::Characters(characters)) => {
                        sale.product_id = characters.parse::<u32>().unwrap();
                        println!("Got sale.product-id: {}.", characters);
                    }
                    Ok(XmlEvent::EndElement { .. }) => {
                        location_sale = LocationSale::Other;
                    }
                    _ => {}
                },
                LocationSale::InDate => match event {
                    Ok(XmlEvent::Characters(characters)) => {
                        sale.date = characters.parse::<i64>().unwrap();
                        println!("Got sale.date: {}.", characters);
                    }
                    Ok(XmlEvent::EndElement { .. }) => {
                        location_sale = LocationSale::Other;
                    }
                    _ => {}
                },
                LocationSale::InQuantity => match event {
                    Ok(XmlEvent::Characters(characters)) => {
                        sale.quantity = characters.parse::<f64>().unwrap();
                        println!("Got sale.quantity: {}.", characters);
                    }
                    Ok(XmlEvent::EndElement { .. }) => {
                        location_sale = LocationSale::Other;
                    }
                    _ => {}
                },
                LocationSale::InUnit => match event {
                    Ok(XmlEvent::Characters(characters)) => {
                        sale.unit = characters.clone();
                        println!("Got sale.unit: {}.", characters);
                    }
                    Ok(XmlEvent::EndElement { .. }) => {
                        location_sale = LocationSale::Other;
                    }
                    _ => {}
                },
            },
        }
    }
}