
CREATE TABLE user (
    id INTEGER PRIMARY KEY,
    pubkey VARCHAR
);
CREATE UNIQUE INDEX user_u ON user(pubkey);

CREATE TABLE version (
    id INTEGER PRIMARY KEY,
    major INTEGER,
    minor INTEGER,
    patch INTEGER,
    target VARCHAR
);
CREATE UNIQUE INDEX version_u ON version(major, minor, patch, target);

CREATE TABLE sysinfo (
    id INTEGER PRIMARY KEY,
    name VARCHAR,
    host_name VARCHAR,
    kernel_version VARCHAR,
    os_version VARCHAR,
    cpu_name VARCHAR,
    cpu_vendor VARCHAR,
    cpu_brand VARCHAR,
    memory VARCHAR
);
CREATE UNIQUE INDEX sysinfo_u ON sysinfo(
    name,
    host_name,
    kernel_version,
    os_version,
    cpu_name, 
    cpu_vendor,
    cpu_brand,
    memory
);

CREATE TABLE user_version_sys (
    id INTEGER PRIMARY KEY,
    user_id INTEGER,
    version_id INTEGER,
    sysinfo_id INTEGER
);
CREATE UNIQUE INDEX user_version_sys_u ON user_version_sys(user_id, version_id, sysinfo_id);

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

INSERT INTO log_level VALUES (0, 'TRACE'), (1, 'DEBUG'), (2, 'INFO'), (3, 'WARN'), (4, 'ERROR');

CREATE TABLE logs (
    id INTEGER PRIMARY KEY,
    session_id INTEGER,
    level_id INTEGER,
    target VARCHAR,
    message VARCHAR
);
