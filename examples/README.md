# README

1. These files are meant to be copy pasted into a main.rs file and runned from there.

2. Make sure to add the sqlitex dependency to your Cargo.toml.

3. They are short with comments guiding along the way.

## Important note

The examples and docs uses inline schema (#[sqlitex] with no args) to keep things self-contained.
For real projects, prefer pointing to an external file instead as it is more flexible. Read more this short section [here](https://docs.rs/sqlitex/latest/sqlitex/#connection-methods)
