PRAGMA foreign_keys = 1;
PRAGMA user_version = 6;
CREATE TABLE entries
(
    id       INTEGER PRIMARY KEY,
    remote   TEXT    NOT NULL,
    category TEXT    NOT NULL,
    package  TEXT    NOT NULL,
    desc     TEXT    NOT NULL,
    type     INTEGER NOT NULL,
    version  TEXT    NOT NULL,
    author   TEXT    NOT NULL,
    flags    INTEGER DEFAULT 0,
    UNIQUE (remote, category, package)
);
CREATE TABLE files
(
    id    INTEGER PRIMARY KEY,
    entry INTEGER     NOT NULL,
    path  TEXT UNIQUE NOT NULL,
    main  INTEGER     NOT NULL,
    type  INTEGER     NOT NULL,
    FOREIGN KEY (entry) REFERENCES entries (id)
);
INSERT INTO entries (id, remote, category, package, desc, type, version, author, flags) VALUES (1, 'My Repository', 'Example', 'Hello World.lua', 'Print Hello World', 1, '1.0.1', 'cfillion', 0);
INSERT INTO files (id, entry, path, main, type) VALUES (1, 1, 'Scripts/My Repository/Example/Hello World.lua', 2, 0);
