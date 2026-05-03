use sqlitex::{sqlitex, Connection};

#[sqlitex("schema.sql")]
struct App {
    add_user: sql!("INSERT INTO users (username) VALUES (?)"),
    get_all: sql!("SELECT * FROM users"),
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let conn = Connection::open("test.db")?;
    let mut db = App::new(conn);

    // init is auto generated when we connect to an external sql file.
    // by running this, it will run all the sql queries on that file, which in this case is `schema.sql`
    db.init()?;

    let _ = db.add_user("Charlie");

    let users = db.get_all()?.all()?;

    for user in users {
        println!("ID: {}, Name: {}", user.id, user.username);
    }

    // prints out:
    //
    // ID: 1, Name: admin
    // ID: 2, Name: guest
    // ID: 3, Name: Charlie

    Ok(())
}
