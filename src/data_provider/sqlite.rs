use crate::dictionary::app_data_dir;
use rusqlite::Connection;

pub fn create_db() {
    let path = app_data_dir();
    let db_file = path.join("data.db");
    if !db_file.exists() {
        std::fs::File::create(&db_file).unwrap();
        let connection = Connection::open(&db_file).unwrap();
        connection.execute("PRAGMA foreign_keys = ON;", []).unwrap();

        create_tables(&connection);
    }else {
        let connection = Connection::open(&db_file).unwrap();

        ensure_db_schema(&connection);
    }
}

fn ensure_db_schema(conn: &Connection) {

}

fn create_tables(conn: &Connection) {
    make_card_set(conn).unwrap();
    make_settings(conn).unwrap();
    make_word_group(conn).unwrap();
    make_words(conn).unwrap();
    make_card_stats(conn).unwrap();
}

fn make_settings(conn: &Connection) -> Result<(), rusqlite::Error> {
    let query = include_str!("../../sql/settings.sql");
    conn.execute(query, ())?;
    Ok(())
}

fn make_card_set(conn: &Connection) -> Result<(), rusqlite::Error> {
    let query = include_str!("../../sql/card_set.sql");
    conn.execute(query, ())?;
    Ok(())
}

fn make_word_group(conn: &Connection) -> Result<(), rusqlite::Error> {
    let query = include_str!("../../sql/word_group.sql");
    conn.execute(query, ())?;
    conn.execute("insert into word_group (name) values (\"Слова\");", ())?;
    Ok(())
}
fn make_words(conn: &Connection) -> Result<(), rusqlite::Error> {
    let query = include_str!("../../sql/words.sql");
    conn.execute(query, ())?;
    Ok(())
}
fn make_card_stats(conn: &Connection) -> Result<(), rusqlite::Error> {
    let query = include_str!("../../sql/card_stats.sql");
    conn.execute(query, ())?;
    Ok(())
}
