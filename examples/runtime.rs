use sqlitex::Connection;
use sqlitex::traits::dynamic::Value;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let conn = Connection::open_memory()?;

    // Use execute for write statements (CREATE, INSERT, UPDATE, DELETE, etc.)
    // Chaining of multiple sql queries via `;` are not allowed
    conn.execute(
        "CREATE TABLE products (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            price REAL,
            in_stock INTEGER
        )",
    )?;

    // _rows_affected variable is the number of rows modified, which in this case is an insert of 3 rows
    let _rows_affected = conn.execute(
        "INSERT INTO products (name, price, in_stock) VALUES
        ('Laptop', 999.99, 1),
        ('Mouse', 25.50, 1),
        ('Keyboard', 75.00, 0)",
    )?;

    // Use query for running SELECT statements
    // Chaining of multiple sql queries via `;` are not allowed
    let results = conn.query("SELECT * FROM products")?;

    // results.column_names is a vec of all the col names defined in the create table
    // which in this case is ["id", "name", "price", "in_stock"]
    println!("All column names: {:?}", results.column_names);

    // row_result is an iterator. Columns are accessed by index, matching the SELECT order.
    // Match on the Value enum to handle each column's type
    for row_result in results {
        let row = row_result?;

        let id = match row[0] {
            Value::Integer(i) => i,
            _ => 0,
        };
        let name = row[1].as_string();

        let price = match row[2] {
            Value::Real(f) => f,
            Value::Null => 0.0,
            _ => 0.0,
        };
        let in_stock = match row[3] {
            Value::Integer(i) => i != 0,
            Value::Null => false,
            _ => false,
        };

        println!("id={id}, name={name}, price={price}, in_stock={in_stock}");
    }

    // u can use helper functions like first() or all() to get a vector of rows.
    let _first_row = conn
        .query("SELECT name, price FROM products WHERE id = 1")?
        .first()?; // or .all()? for all rows


    Ok(())
}
