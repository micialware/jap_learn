use crate::repetitions::CardSetSettings;
use rusqlite::Connection;
use crate::lang::SetOrderMode;

pub fn load_sets(connection: &Connection) -> Vec<CardSetSettings> {
    let mut stmt = connection.prepare("SELECT id, name, forward, backward, filter FROM card_set").unwrap();
    let iter = stmt.query_map([], |row| {
        Ok(CardSetSettings {
            id: row.get(0)?,
            name: row.get(1)?,
            forward: row.get(2)?,
            backward: row.get(3)?,
            filter: row.get(4)?,
            count: None,
            worst_words_list: None,
            open_mode: SetOrderMode::Default,
        })
    }).unwrap();

    let mut buffer = vec![];
    for word in iter {
        buffer.push(word.unwrap());
    }

    buffer
}

pub fn add_set(set: &mut CardSetSettings, connection: &Connection) {
    let index = connection
        .query_row(
            "INSERT INTO card_set (name, forward, backward, filter) VALUES (?1, ?2, ?3, ?4) RETURNING id",
            (
                &set.name,
                &set.forward,
                &set.backward,
                &set.filter,
            ),
            |row| row.get(0)
        )
        .unwrap_or_else(|e| {println!("{}", e); 0});

    set.id = index;
}

pub fn update_card_set(set: &mut CardSetSettings, connection: &Connection){
    if set.id == 0 {
        add_set(set, &connection);
    }

    else {
        connection
            .execute(
                "UPDATE card_set SET name = ?1, forward = ?2, backward = ?3, filter = ?4 WHERE id = ?5",
                (
                    &set.name,
                    &set.forward,
                    &set.backward,
                    &set.filter,
                    &set.id
                ),
            )
            .unwrap_or_else(|e| {println!("{}", e); 0});
    }
}

pub fn delete_set(set: &CardSetSettings, connection: &Connection) {
    if set.id == 0 {
        return;
    }
    connection
        .execute("DELETE FROM card_set WHERE id = ?1", (&set.id,))
        .unwrap_or_else(|e| {
            println!("{}", e);
            0
        });
}