use anyhow::Context;
use rusqlite::Connection;
use std::fs;

pub struct Snippet {
  pub id: i32,
  pub command: String,
  pub created_at: String,
}

fn db_path() -> anyhow::Result<std::path::PathBuf> {
  let base = dirs::data_local_dir().context("could not find local data directory")?;
  let dir = base.join("snip");
  fs::create_dir_all(&dir)?;
  Ok(dir.join("snip.db"))
}

pub fn init() -> anyhow::Result<Connection> {
  let path = db_path()?;
  let conn = Connection::open(&path)?;

  conn.execute_batch("
    CREATE TABLE IF NOT EXISTS snippets (
            id         INTEGER PRIMARY KEY AUTOINCREMENT,
            command    TEXT NOT NULL UNIQUE,
            created_at TEXT DEFAULT (datetime('now'))
        );")?;
  Ok(conn)
}

pub fn save(conn: &Connection, command: &str) -> anyhow::Result<()> {
  conn.execute(
        "INSERT OR IGNORE INTO snippets (command) VALUES (?1)",
        rusqlite::params![command],
    )
    .map(|_| ())
    .map_err(|e| anyhow::anyhow!(e))
}

pub fn list(conn: &Connection) -> anyhow::Result<Vec<Snippet>> {
    let mut stmt = conn.prepare(
        "SELECT id, command, created_at FROM snippets ORDER BY created_at DESC",
    )?;
    stmt.query_map([], |row| {
        Ok(Snippet {
            id: row.get(0)?,
            command: row.get(1)?,
            created_at: row.get(2)?,
        })
    })
    .map_err(|e| anyhow::anyhow!(e))?
    .collect::<Result<Vec<_>, rusqlite::Error>>()  // explicit error type
    .map_err(|e| anyhow::anyhow!(e))               // then convert to anyhow
}

pub fn search(conn: &Connection, query: &str) -> anyhow::Result<Vec<Snippet>> {
  let q = query.to_lowercase();
  list(conn).map(|snippets| {
    snippets
    .into_iter()
    .filter(|s| s.command.to_lowercase().contains(&q))
    .collect()
  })
}

pub fn delete(conn: &Connection, id: i32) -> anyhow::Result<()> {
  conn.execute(
    "DELETE FROM snippets WHERE id = ?1", rusqlite::params![id],
  )
  .map_err(|e| anyhow::anyhow!(e))
  .and_then(|affected| match affected {
    0 => Err(anyhow::anyhow!("no snippet found with id {}", id)),
    _ => Ok(())
  })
}

pub fn drop_db() -> anyhow::Result<()> {
  let path = db_path()?;
  if path.exists() {
    fs::remove_file(&path)?;
    println!("Database dropped: {}", path.display());
  } else {
    println!("No database found at {}", path.display());
  }
  Ok(())
}

