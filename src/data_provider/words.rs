use crate::dictionary::app_data_dir;
use crate::lang::{WordData, WordGroup};
use rusqlite::Connection;
use std::collections::HashMap;

pub fn create_db() {
    let path = app_data_dir();
    let db_file = path.join("data.db");
    if !db_file.exists() {
        std::fs::File::create(&db_file).unwrap();
        let connection = Connection::open(&db_file).unwrap();
        connection.execute("PRAGMA foreign_keys = ON;", []).unwrap();

        create_tables(&connection);
    }
}

fn create_tables(conn: &Connection) {
    conn.execute(
        "
        CREATE TABLE word_group (
            id   INTEGER PRIMARY KEY AUTOINCREMENT,
            name   TEXT NOT NULL
        );

        insert into word_group (name)
        values (\"Слова\");

      CREATE TABLE words (
            id   INTEGER PRIMARY KEY AUTOINCREMENT,
            key TEXT NOT NULL,
            value TEXT NOT NULL,
            tags TEXT NOT NULL,
            more TEXT,
            group_id INTEGER NOT NULL DEFAULT 1,
            FOREIGN KEY(group_id) REFERENCES word_group(id)
        );

        CREATE TABLE card_stats (
            id   INTEGER PRIMARY KEY AUTOINCREMENT,
            word_id INTEGER NOT NULL,
            set_id TEXT NOT NULL,
            score INTEGER NOT NULL DEFAULT 1,
            last_opened INTEGER NOT NULL,
            FOREIGN KEY (word_id)  REFERENCES words (id) ON DELETE CASCADE,
            FOREIGN KEY (set_id)  REFERENCES card_set (id) ON DELETE CASCADE
        );



        CREATE TABLE card_set (
            id   INTEGER PRIMARY KEY AUTOINCREMENT,
            name   TEXT NOT NULL,
            forward TEXT NOT NULL,
            backward TEXT NOT NULL,
            filter TEXT NOT NULL
        );
        ",
        (),
    )
    .unwrap_or_else(|e| {
        println!("{}", e);
        0
    });
}

pub fn add_word(word: &mut WordData, connection: &Connection) {
    let index = connection
        .query_row(
            "INSERT INTO words (key, value, tags, more) VALUES (?1, ?2, ?3, ?4) RETURNING id",
            (
                &word.key,
                &word.value,
                &word.tags,
                serde_json::to_string(&word.additional).unwrap(),
            ),
            |row| row.get(0),
        )
        .unwrap_or_else(|e| {
            println!("{}", e);
            0
        });

    word.id = index;
}

pub fn update_word(word: &mut WordData, connection: &Connection) {
    if word.id == 0 {
        add_word(word, &connection);
    } else {
        connection
            .execute(
                "UPDATE words SET key = ?1, value = ?2, tags = ?3, more = ?4 WHERE id = ?5",
                (
                    &word.key,
                    &word.value,
                    &word.tags,
                    serde_json::to_string(&word.additional).unwrap(),
                    &word.id,
                ),
            )
            .unwrap_or_else(|e| {
                println!("{}", e);
                0
            });
    }
}

pub fn delete_word(word: &WordData, connection: &Connection) {
    if word.id == 0 {
        return;
    }
    connection
        .execute("DELETE FROM words WHERE id = ?1", (&word.id,))
        .unwrap_or_else(|e| {
            println!("{}", e);
            0
        });
}

pub fn load_words(connection: &Connection) -> Vec<WordData> {
    let mut stmt = connection
        .prepare("SELECT id, key, value, tags, more, group_id FROM words")
        .unwrap();
    let word_iter = stmt
        .query_map([], |row| {
            let addinionals: String = row.get(4)?;
            Ok(WordData {
                id: row.get(0)?,
                key: row.get(1)?,
                value: row.get(2)?,
                tags: row.get(3)?,
                additional: serde_json::from_str::<HashMap<String, String>>(&addinionals).unwrap(),
                group_id: row.get(5)?,
            })
        })
        .unwrap();

    let mut buffer = vec![];
    for word in word_iter {
        buffer.push(word.unwrap());
    }

    buffer
}

pub fn load_word_groups(connection: &Connection) -> Vec<WordGroup> {
    let mut stmt = connection
        .prepare("SELECT id, name FROM word_group")
        .unwrap();
    let group_iter = stmt
        .query_map([], |row| {
            Ok(WordGroup {
                id: row.get(0)?,
                name: row.get(1)?,
            })
        })
        .unwrap();

    let mut buffer = vec![];
    for group in group_iter {
        buffer.push(group.unwrap());
    }

    buffer
}