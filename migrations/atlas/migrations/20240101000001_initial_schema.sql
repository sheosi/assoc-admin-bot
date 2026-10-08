-- Initial schema migration
-- Created: 2024-01-01

-- Create config table
CREATE TABLE IF NOT EXISTS config (
    admins TEXT NOT NULL DEFAULT '{}',
    treasury_notifications_chat_id INTEGER,
    assembly_minutes_chat_id INTEGER
);

-- Create festAttendee table
CREATE TABLE IF NOT EXISTS festAttendee (
    legalName TEXT NOT NULL,
    codeName TEXT NOT NULL,
    hasPaid INTEGER NOT NULL DEFAULT 0,
    assistsTo TEXT NOT NULL,
    allergies TEXT,
    diet INTEGER NOT NULL
);

-- Create shopSell table
CREATE TABLE IF NOT EXISTS shopSell (
    item TEXT NOT NULL,
    price INTEGER NOT NULL
);

-- Create userHistory table
CREATE TABLE IF NOT EXISTS userHistory (
    userId INTEGER NOT NULL,
    slotId INTEGER NOT NULL,
    message TEXT NOT NULL DEFAULT '',
    answer TEXT NOT NULL DEFAULT '',
    createdAt DATETIME NOT NULL,
    PRIMARY KEY (userId, slotId)
);

-- Create historyPointers table
CREATE TABLE IF NOT EXISTS historyPointers (
    userId INTEGER PRIMARY KEY,
    lastSlot INTEGER NOT NULL DEFAULT 0
);

-- Create associates table
CREATE TABLE IF NOT EXISTS associates (
    nickName TEXT PRIMARY KEY,
    email TEXT NOT NULL
);

-- Create treasuryUpdates table
CREATE TABLE IF NOT EXISTS treasuryUpdates (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    description TEXT NOT NULL,
    amount REAL NOT NULL,
    createdAt DATETIME NOT NULL,
    announced INTEGER NOT NULL DEFAULT 0
);

-- Insert default admin (will be populated by application)
-- This table might be empty initially
