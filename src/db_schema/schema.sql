-- schema.sql
 CREATE TABLE config (
   admins TEXT NOT NULL DEFAULT '{}',
   treasury_notifications_chat_id INTEGER,
   assembly_minutes_chat_id INTEGER
 );

CREATE TABLE festAttendee (
    legalName TEXT NOT NULL,
    codeName TEXT NOT NULL,
    hasPaid INTEGER NOT NULL DEFAULT 0,
    assistsTo TEXT NOT NULL,
    allergies TEXT,
    diet INTEGER NOT NULL
);

CREATE TABLE shopSell (
    item TEXT NOT NULL,
    price INTEGER NOT NULL
);


CREATE TABLE userHistory (
    userId INTEGER NOT NULL,
    slotId INTEGER NOT NULL, -- 1 to 5
    message TEXT NOT NULL DEFAULT '',
    answer TEXT NOT NULL DEFAULT '',
    createdAt DATETIME NOT NULL,
    PRIMARY KEY (userId, slotId)
);

-- We also need a way to know which slot is the 'newest' for each user
CREATE TABLE  historyPointers (
    userId INTEGER PRIMARY KEY,
    lastSlot INTEGER NOT NULL DEFAULT 0 -- Stores 1, 2, 3, 4, or 5
);

-- Associates table
CREATE TABLE associates (
    nickName TEXT PRIMARY KEY,
    email TEXT NOT NULL
);

-- Treasury updates tracking
CREATE TABLE treasuryUpdates (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    description TEXT NOT NULL,
    amount REAL NOT NULL,
    createdAt DATETIME NOT NULL,
    announced INTEGER NOT NULL DEFAULT 0 -- 0 = not announced, 1 = announced
);
