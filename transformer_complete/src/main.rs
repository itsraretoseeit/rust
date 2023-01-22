use redis::Commands;
use serde_derive::{Deserialize, Serialize};


#[allow(unused)]
#[derive(Debug, Deserialize)]
struct Input {
    json_file: String,
}

#[allow(unused)]
#[derive(Debug,Deserialize)]
struct Redis {
    host: String,
}

#[allow(unused)]
#[derive(Debug, Deserialize)]
struct Sqlite {
    db_file: String,
}

#[allow(unused)]
#[derive(Debug, Deserialize)]
struct Postgresql {
    username: String,
    password: String,
    host: String,
    port: String,
    database: String,
}

#[allow(unused)]
#[derive(Debug, Deserialize)]
struct Config {
    input: Input,
    redis: Redis,
    sqlite: Sqlite,
    postgresql: Postgresql,
}

#[derive(Deserialize, Serialize, Debug, Default)]
struct Product {
    id: i32,
    category: String,
    name: String,
}

#[derive(Deserialize, Serialize, Debug, Default)]
struct Sale {
    id: String,
    product_id: i32,
    date: i64,
    quantity: f64,
    unit: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct SalesAndProducts {
    products: Vec<Product>,
    sales: Vec<Sale>,
}

fn read_json_file(pathname: &str) -> SalesAndProducts {
    serde_json::from_str::<SalesAndProducts>(&std::fs::read_to_string(&pathname).unwrap()).unwrap()
}

fn recreate_sqlite_db(sqlite_config: &Sqlite) -> rusqlite::Result<rusqlite::Connection> {
    use rusqlite::{params, Connection};
    let conn = Connection::open(&sqlite_config.db_file)?;
    let _ = conn.execute("DROP TABLE Sales", params![]);
    let _ = conn.execute("DROP TABLE Products", params![]);
    conn.execute(
        "CREATE TABLE Products (
            id INTEGER PRIMARY KEY,
            category TEXT NOT NULL,
            name TEXT NOT NULL UNIQUE)",
        params![],
    )?;
    conn.execute(
        "CREATE TABLE Sales (
            id TEXT PRIMARY KEY,
            product_id INTEGER NOT NULL REFERENCES Products,
            sale_date BIGINT NOT NULL,
            quantity DOUBLE PRECISION NOT NULL,
            unit TEXT NOT NULL)",
        params![],
    )?;
    Ok(conn)
}

fn write_into_sqlite_db(
    conn: &rusqlite::Connection,
    sales_and_products: &SalesAndProducts,
) -> rusqlite::Result<()> {
    use rusqlite::params;
    for product in &sales_and_products.products {
        conn.execute(
            "INSERT INTO Products (
            id, category, name
            ) VALUES ($1, $2, $3)",
            params![product.id, product.category, product.name],
        )?;
    }
    for sale in &sales_and_products.sales {
        conn.execute(
            "INSERT INTO Sales (
            id, product_id, sale_date, quantity, unit
            ) VALUES ($1, $2, $3, $4, $5)",
            params![
                sale.id,
                sale.product_id,
                sale.date,
                sale.quantity,
                sale.unit,
            ],
        )?;
    }
    Ok(())
}

fn recreate_postgresql_db(
    postgresql_config: &Postgresql,
) -> Result<postgres::Client, postgres::error::Error> {
    use postgres::{Client, NoTls};
    let mut conn = Client::connect(
        &format!(
            "postgres://{}{}{}@{}{}{}{}{}",
            postgresql_config.username,
            if postgresql_config.password.is_empty() {
                ""
            } else {
                ":"
            },
            postgresql_config.password,
            postgresql_config.host,
            if postgresql_config.port.is_empty() {
                ""
            } else {
                ":"
            },
            postgresql_config.port,
            if postgresql_config.database.is_empty() {
                ""
            } else {
                "/"
            },
            postgresql_config.database
        ),
        NoTls,
    )?;
    conn.execute("DROP TABLE Sales", &[])?;
    conn.execute("DROP TABLE Products", &[])?;
    conn.execute(
        "CREATE TABLE Products (
        id INTEGER PRIMARY KEY,
        category TEXT NOT NULL,
        name TEXT NOT NULL UNIQUE)",
        &[],
    )?;
    conn.execute(
        "CREATE TABLE Sales (
        id TEXT PRIMARY KEY,
        product_id INTEGER NOT NULL REFERENCES Products,
        sale_date BIGINT NOT NULL,
        quantity DOUBLE PRECISION NOT NULL,
        unit TEXT NOT NULL)",
        &[],
    )?;
    Ok(conn)
}

fn write_into_postgresql_db(
    conn: &mut postgres::Client,
    sales_and_products: &SalesAndProducts,
) -> Result<(), postgres::error::Error> {
    for product in &sales_and_products.products {
        conn.execute(
            "INSERT INTO Products (
            id, category, name
            ) VALUES ($1, $2, $3)",
            &[&product.id, &product.category, &product.name],
        )?;
    }
    for sale in &sales_and_products.sales {
        conn.execute(
            "INSERT INTO Sales (
            id, product_id, sale_date, quantity, unit
            ) VALUES ($1, $2, $3, $4, $5)",
            &[
                &sale.id,
                &sale.product_id,
                &sale.date,
                &sale.quantity,
                &sale.unit,
            ],
        )?;
    }
    Ok(())
}

fn write_into_redis_store(
    conn: &mut redis::Connection,
    sales_and_products: &SalesAndProducts,
) -> redis::RedisResult<()> {
    for product in &sales_and_products.products {
        conn.set(
            format!("product:{}:category", product.id),
            &product.category,
        )?;
        conn.set(format!("product:{}:name", product.id), &product.name)?;
    }
    for sale in &sales_and_products.sales {
        conn.set(format!("sale:{}:product_id", sale.id), sale.product_id)?;
        conn.set(format!("sale:{}:sale_date", sale.id), sale.date)?;
        conn.set(format!("sale:{}:quantity", sale.id), sale.quantity)?;
        conn.set(format!("sale:{}:unit", sale.id), &sale.unit)?;
    }
    Ok(())
}

fn print_row_count_in_sqlite_db(conn: &rusqlite::Connection) -> rusqlite::Result<()> {
    use rusqlite::params;
    for count in conn
        .prepare("SELECT COUNT(*) FROM Products")?
        .query_map(params![], |row| {
            let c: i64 = row.get(0)?;
            Ok(c)
        })?
    {
        if let Ok(count) = count {
            println!("SQLite #Products={}. ", count);
        }
    }
    for count in conn
        .prepare("SELECT COUNT(*) FROM Sales")?
        .query_map(params![], |row| {
            let c: i64 = row.get(0)?;
            Ok(c)
        })?
    {
        if let Ok(item) = count {
            println!("SQLite #Sales={}. ", item);
        }
    }
    Ok(())
}

fn print_row_count_in_postgresql_db(
    conn: &mut postgres::Client,
) -> Result<(), postgres::error::Error> {
    for row in &conn.query("SELECT COUNT(*) FROM Products", &[])? {
        let count: i64 = row.get(0);
        println!("PostgreSQL #Products={}. ", count);
    }
    for row in &conn.query("SELECT COUNT(*) FROM Sales", &[])? {
        let count: i64 = row.get(0);
        println!("PostgreSQL #Sales={}. ", count);
    }
    Ok(())
}

fn open_redis_store(redis_config: &Redis) -> redis::RedisResult<redis::Connection> {
    Ok(
        redis::Client::open(format!("redis://{}/", redis_config.host).as_str())?
            .get_connection()?,
    )
}

fn main() {
    // Define the config structure by reading the TOML file
    // specified in the command line.
    let config: Config = {
        let config_path = std::env::args().nth(1).unwrap();
        let config_text = std::fs::read_to_string(&config_path).unwrap();
        toml::from_str(&config_text).unwrap()
    };

    let mut sales_and_products = read_json_file(&config.input.json_file);

    let sqlite_conn = recreate_sqlite_db(&config.sqlite).unwrap();
    write_into_sqlite_db(&sqlite_conn, &sales_and_products).unwrap();

    let mut postgresql_conn = recreate_postgresql_db(&config.postgresql).unwrap();
    write_into_postgresql_db(&mut postgresql_conn, &sales_and_products).unwrap();

    let mut redis_conn = open_redis_store(&config.redis).unwrap();
    write_into_redis_store(&mut redis_conn, &sales_and_products).unwrap();

    print_row_count_in_sqlite_db(&sqlite_conn).unwrap();
    print_row_count_in_postgresql_db(&mut postgresql_conn).unwrap();
}
