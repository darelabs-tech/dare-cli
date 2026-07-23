//! Versioned GraphRAG schema migrations (explicit only — ADR-006).

use dare_core::{CoreError, CoreResult};
use rusqlite::Connection;

/// Current schema version after successful migrate (baseline TS 3.18.1 + migrations table).
pub const CURRENT_SCHEMA_VERSION: u32 = 1;

/// Exact DDL from `@dewtech/dare-cli@3.18.1` `SCHEMA_SQL`.
pub const SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS nodes (
  id TEXT PRIMARY KEY,
  type TEXT NOT NULL,
  label TEXT NOT NULL,
  description TEXT,
  vector BLOB,
  metadata TEXT DEFAULT '{}',
  created_at TEXT DEFAULT (datetime('now')),
  updated_at TEXT DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS edges (
  id TEXT PRIMARY KEY,
  source_id TEXT NOT NULL,
  target_id TEXT NOT NULL,
  type TEXT NOT NULL,
  weight REAL DEFAULT 1.0,
  metadata TEXT DEFAULT '{}'
);

CREATE INDEX IF NOT EXISTS idx_nodes_type ON nodes(type);
CREATE INDEX IF NOT EXISTS idx_edges_source ON edges(source_id);
CREATE INDEX IF NOT EXISTS idx_edges_target ON edges(target_id);
CREATE INDEX IF NOT EXISTS idx_edges_type ON edges(type);
"#;

const MIGRATIONS_DDL: &str = r#"
CREATE TABLE IF NOT EXISTS dare_schema_migrations (
  version INTEGER PRIMARY KEY,
  applied_at TEXT NOT NULL
);
"#;

fn table_exists(conn: &Connection, name: &str) -> CoreResult<bool> {
    let mut stmt = conn
        .prepare("SELECT 1 FROM sqlite_master WHERE type='table' AND name=?1 LIMIT 1")
        .map_err(|e| CoreError::io(e.to_string()))?;
    let exists = stmt
        .exists([name])
        .map_err(|e| CoreError::io(e.to_string()))?;
    Ok(exists)
}

fn column_names(conn: &Connection, table: &str) -> CoreResult<Vec<String>> {
    let mut stmt = conn
        .prepare(&format!("PRAGMA table_info({table})"))
        .map_err(|e| CoreError::io(e.to_string()))?;
    let rows = stmt
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(|e| CoreError::io(e.to_string()))?;
    let mut names = Vec::new();
    for r in rows {
        names.push(r.map_err(|e| CoreError::io(e.to_string()))?);
    }
    Ok(names)
}

/// Detect schema version without mutating the database.
///
/// - missing `nodes` → treat as empty / needs migrate to create (reported as 0)
/// - `nodes` without `vector` → 0
/// - max(`dare_schema_migrations.version`) if table present
/// - else full baseline schema → 1 (legacy TS DB without migrations table)
pub fn detect_sqlite_schema_version(conn: &Connection) -> CoreResult<u32> {
    if !table_exists(conn, "nodes")? {
        return Ok(0);
    }
    let cols = column_names(conn, "nodes")?;
    if !cols.iter().any(|c| c == "vector") {
        return Ok(0);
    }
    if table_exists(conn, "dare_schema_migrations")? {
        let v: Option<i64> = conn
            .query_row(
                "SELECT MAX(version) FROM dare_schema_migrations",
                [],
                |row| row.get::<_, Option<i64>>(0),
            )
            .map_err(|e| CoreError::io(e.to_string()))?;
        return Ok(v.unwrap_or(1).max(1) as u32);
    }
    Ok(1)
}

/// Apply migrations forward-only to reach [`CURRENT_SCHEMA_VERSION`].
pub fn migrate_sqlite(conn: &Connection) -> CoreResult<u32> {
    let version = detect_sqlite_schema_version(conn)?;
    if version >= CURRENT_SCHEMA_VERSION && table_exists(conn, "dare_schema_migrations")? {
        return Ok(version);
    }

    if version == 0 {
        if table_exists(conn, "nodes")? {
            let cols = column_names(conn, "nodes")?;
            if !cols.iter().any(|c| c == "vector") {
                conn.execute("ALTER TABLE nodes ADD COLUMN vector BLOB", [])
                    .map_err(|e| CoreError::io(e.to_string()))?;
            }
            if !table_exists(conn, "edges")? {
                conn.execute_batch(SCHEMA_SQL)
                    .map_err(|e| CoreError::io(e.to_string()))?;
            }
        } else {
            conn.execute_batch(SCHEMA_SQL)
                .map_err(|e| CoreError::io(e.to_string()))?;
        }
    }

    conn.execute_batch(MIGRATIONS_DDL)
        .map_err(|e| CoreError::io(e.to_string()))?;

    let already: bool = conn
        .query_row(
            "SELECT 1 FROM dare_schema_migrations WHERE version = ?1",
            [CURRENT_SCHEMA_VERSION],
            |_| Ok(true),
        )
        .unwrap_or(false);
    if !already {
        conn.execute(
            "INSERT INTO dare_schema_migrations(version, applied_at) VALUES (?1, datetime('now'))",
            [CURRENT_SCHEMA_VERSION],
        )
        .map_err(|e| CoreError::io(e.to_string()))?;
    }

    Ok(CURRENT_SCHEMA_VERSION)
}

/// Create baseline schema for a brand-new database **without** recording migrations
/// (caller should still call migrate for the version table when desired).
pub fn ensure_baseline_schema(conn: &Connection) -> CoreResult<()> {
    conn.execute_batch(SCHEMA_SQL)
        .map_err(|e| CoreError::io(e.to_string()))?;
    Ok(())
}
