//! Database module - synchronous rusqlite implementation
//! Uses sqlc-generated queries from the queries module

use anyhow::{Context, Result};
use deadpool_sqlite::rusqlite::Connection;

/// Initialize database tables if they don't exist
pub fn init_database(conn: &Connection) -> Result<()> {
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS config (
            admins TEXT NOT NULL DEFAULT '{}',
            treasury_notifications_chat_id INTEGER,
            assembly_minutes_chat_id INTEGER
        );
        "#,
        [],
    )
    .context("Failed to create config table")?;

    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS festAttendee (
            legalName TEXT NOT NULL,
            codeName TEXT NOT NULL,
            hasPaid INTEGER NOT NULL DEFAULT 0,
            assistsTo TEXT NOT NULL,
            allergies TEXT,
            diet INTEGER NOT NULL
        );
        "#,
        [],
    )
    .context("Failed to create festAttendee table")?;

    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS shopSell (
            item TEXT NOT NULL,
            price INTEGER NOT NULL
        );
        "#,
        [],
    )
    .context("Failed to create shopSell table")?;

    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS userHistory (
            userId INTEGER NOT NULL,
            slotId INTEGER NOT NULL,
            message TEXT NOT NULL DEFAULT '',
            answer TEXT NOT NULL DEFAULT '',
            createdAt DATETIME NOT NULL,
            PRIMARY KEY (userId, slotId)
        );
        "#,
        [],
    )
    .context("Failed to create userHistory table")?;

    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS historyPointers (
            userId INTEGER PRIMARY KEY,
            lastSlot INTEGER NOT NULL DEFAULT 0
        );
        "#,
        [],
    )
    .context("Failed to create historyPointers table")?;

    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS associates (
            nickName TEXT PRIMARY KEY,
            email TEXT NOT NULL
        );
        "#,
        [],
    )
    .context("Failed to create associates table")?;

    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS treasuryUpdates (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            description TEXT NOT NULL,
            amount REAL NOT NULL,
            createdAt DATETIME NOT NULL,
            announced INTEGER NOT NULL DEFAULT 0
        );
        "#,
        [],
    )
    .context("Failed to create treasuryUpdates table")?;

    Ok(())
}

/// Open a database connection
pub fn open_db(path: &std::path::Path) -> Result<Connection> {
    let conn = Connection::open(path).context("Failed to open database")?;
    init_database(&conn)?;
    Ok(conn)
}
