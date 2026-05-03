//! nothing much to comment on and is mostly self explanatory

use sqlitex::{Connection, sqlitex};

#[sqlitex("test.db")]
struct App {
    get_all: sql!("SELECT * FROM users"),
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let conn = Connection::open("test.db")?;
    let mut db = App::new(conn);

    for user in db.get_all()?.all()? {
        println!("{} - {}", user.id, user.username);
    }

        //prints out:
        //
        // 1 - admin
        // 2 - guest
        // 3 - Charlie
    Ok(())
}

