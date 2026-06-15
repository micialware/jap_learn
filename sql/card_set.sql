create table card_set
(
    id       INTEGER
        primary key autoincrement,
    name     TEXT not null,
    forward  TEXT not null,
    backward TEXT not null,
    filter   TEXT not null
);