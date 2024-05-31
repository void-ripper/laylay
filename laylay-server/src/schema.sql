
CREATE TABLE devices (
    id INTEGER PRIMARY KEY
);

CREATE TABLE log_level (
    id INTEGER PRIMARY KEY,
    name VARCHAR UNIQUE
);

INSERT INTO log_level VALUES (0, 'DEBUG'), (1, 'INFO'), (2, 'WARN'), (3, 'ERROR');

CREATE TABLE logs (
    id INTEGER PRIMARY KEY,
    level_id INTEGER,
    target VARCHAR,
    message VARCHAR
);