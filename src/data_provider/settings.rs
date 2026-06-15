use rusqlite::Connection;

pub fn get_setting(key: String, connection: &Connection) -> Option<String> {
    let mut stmt = connection.prepare("SELECT value FROM settings WHERE id = ?1").unwrap();
    let iter = stmt.query_map((key,), |row| {
        row.get(0)
    }).unwrap();

    for row in iter {
        if let Ok(value) = row {
            return Some(value);
        }
        return None;
    }
    None
}

pub fn set_setting(key: String, value: String, connection: &Connection) {
    let current = get_settings_list(connection);
    if current.contains(&key) {
        update_settings(key, value, connection);
    }else {
        create_settings(key, value, connection);
    }
}

pub fn delete_settings(key: String, connection: &Connection) {
    connection
        .execute("DELETE FROM settings WHERE id = ?1", (&key,))
        .unwrap_or_else(|e| {
            println!("{}", e);
            0
        });
}

fn create_settings(key: String, value: String, connection: &Connection) {
    connection
        .execute(
            "INSERT into settings (id, value) VALUES (?1, ?2)",
            (key, value),
        )
        .unwrap_or_else(|e| {
            println!("{}", e);
            0
        });
}

fn update_settings(key: String, value: String, connection: &Connection) {
    connection
        .execute(
            "update settings
set value = ?2
where id = ?1",
            (key, value),
        )
        .unwrap_or_else(|e| {
            println!("{}", e);
            0
        });}

fn get_settings_list(connection: &Connection) -> Vec<String> {
    let mut stmt = connection.prepare("SELECT id FROM settings").unwrap();
    let iter = stmt.query_map((), |row| {
        row.get(0)
    }).unwrap();
    iter.map(|row| { row.unwrap() }).collect()
}

