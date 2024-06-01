
CREATE TABLE user (
    id INTEGER PRIMARY KEY,
    pubkey VARCHAR
);

CREATE TABLE version (
    id INTEGER PRIMARY KEY,
    major INTEGER,
    minor INTEGER,
    patch INTEGER,
    target VARCHAR
);

CREATE TABLE sysinfo (
    id INTEGER PRIMARY KEY,
    sysname VARCHAR,
    nodename VARCHAR,
    release VARCHAR,
    version VARCHAR,
    machine VARCHAR
);

CREATE TABLE user_version_sys (
    id INTEGER PRIMARY KEY,
    user_id INTEGER,
    version_id INTEGER,
    info_id INTEGER
);

CREATE TABLE user_session (
    id INTEGER PRIMARY KEY,
    uvs_id INTEGER,
    started DATETIME,
    ended DATETIME
);

CREATE TABLE log_level (
    id INTEGER PRIMARY KEY,
    name VARCHAR UNIQUE
);

INSERT INTO log_level VALUES (0, 'DEBUG'), (1, 'INFO'), (2, 'WARN'), (3, 'ERROR');

CREATE TABLE logs (
    id INTEGER PRIMARY KEY,
    session_id INTEGER,
    level_id INTEGER,
    target VARCHAR,
    message VARCHAR
);
