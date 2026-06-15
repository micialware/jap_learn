create table card_stats
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
);