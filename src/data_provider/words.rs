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
        "create table settings
(
    id text primary key,
    value text
);",
        (),
    )
        .unwrap_or_else(|e| 0);

    conn.execute(
        "create table card_set
(
    id       INTEGER
        primary key autoincrement,
    name     TEXT not null,
    forward  TEXT not null,
    backward TEXT not null,
    filter   TEXT not null
);",
        (),
    )
        .unwrap_or_else(|e| 0);
    conn.execute(
        "create table word_group
(
    id   INTEGER
        primary key autoincrement,
    name TEXT not null
);",
        (),
    )
        .unwrap_or_else(|e| 0);
    conn.execute(
        "create table words
(
    id       INTEGER
        primary key autoincrement,
    key      TEXT              not null,
    value    TEXT              not null,
    tags     TEXT              not null,
    more     TEXT,
    group_id integer default 1 not null
        constraint words_word_group_id_fk
            references word_group
            on update cascade on delete cascade
);",
        (),
    )
        .unwrap_or_else(|e| 0);
    conn.execute(
        "create table card_stats
(
    id          INTEGER
        primary key autoincrement,
    word_id     INTEGER           not null
        references words
            on delete cascade,
    set_id      TEXT              not null
        references card_set
            on delete cascade,
    score       INTEGER default 1 not null,
    last_opened integer           not null
);",
        (),
    )
        .unwrap_or_else(|e| 0);
    conn.execute(
        "insert into word_group (name)
values (\"Слова\");",
        (),
    )
        .unwrap_or_else(|e| 0);
}

pub fn add_word(word: &mut WordData, connection: &Connection) {
    let index = connection
        .query_row(
            "INSERT INTO words (key, value, tags, more, group_id) VALUES (?1, ?2, ?3, ?4, ?5\
            ) RETURNING id",
            (
                &word.key,
                &word.value,
                &word.tags,
                serde_json::to_string(&word.additional).unwrap(),
                &word.group_id,
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
            let additional: String = row.get(4)?;
            Ok(WordData {
                id: row.get(0)?,
                key: row.get(1)?,
                value: row.get(2)?,
                tags: row.get(3)?,
                additional: serde_json::from_str::<HashMap<String, String>>(&additional).unwrap(),
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

pub fn add_group(group: &mut WordGroup, connection: &Connection) {
    let index = connection
        .query_row(
            "INSERT INTO word_group (name) VALUES (?1) RETURNING id",
            (&group.name,),
            |row| row.get(0),
        )
        .unwrap_or_else(|e| {
            println!("{}", e);
            0
        });

    group.id = index;
}

pub fn update_group(group: &mut WordGroup, connection: &Connection) {
    if group.id == 0 {
        add_group(group, &connection);
    } else {
        connection
            .execute(
                "UPDATE word_group SET name = ?1 WHERE id = ?5",
                (&group.name, &group.id),
            )
            .unwrap_or_else(|e| {
                println!("{}", e);
                0
            });
    }
}

pub fn delete_group(group: &WordGroup, connection: &Connection) {
    if group.id == 0 {
        return;
    }
    connection
        .execute("DELETE FROM word_group WHERE id = ?1", (&group.id,))
        .unwrap_or_else(|e| {
            println!("{}", e);
            0
        });
}
