create table words
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
);