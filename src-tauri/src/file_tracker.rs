use rusqlite::{Connection, Result as SqlResult};
use std::path::{Path, PathBuf};
use sha2::{Sha256, Digest};
use std::fs;
use std::io::Read;
use std::time::SystemTime;
use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct FileMetadata {
    pub path: String,
    pub hash: String,
    pub size: u64,
    pub modified: u64,
    pub last_synced: Option<u64>,
}

pub fn get_database_path() -> Result<PathBuf> {
    let data_dir = crate::config::get_data_dir()?;
    Ok(data_dir.join("sync_bot.db"))
}

pub fn init_database() -> Result<()> {
    let db_path = get_database_path()?;
    let conn = Connection::open(&db_path)
        .context("Failed to open database")?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS file_metadata (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            path TEXT NOT NULL UNIQUE,
            hash TEXT NOT NULL,
            size INTEGER NOT NULL,
            modified INTEGER NOT NULL,
            last_synced INTEGER,
            created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
        )",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_path ON file_metadata(path)",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS tracked_paths (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            path TEXT NOT NULL UNIQUE,
            is_directory INTEGER NOT NULL,
            created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
        )",
        [],
    )?;

    Ok(())
}

pub fn get_connection() -> Result<Connection> {
    let db_path = get_database_path()?;
    Connection::open(&db_path)
        .context("Failed to open database connection")
}

pub fn calculate_file_hash(path: &Path) -> Result<String> {
    let mut file = fs::File::open(path)
        .context("Failed to open file for hashing")?;
    
    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; 8192];
    
    loop {
        let bytes_read = file.read(&mut buffer)
            .context("Failed to read file for hashing")?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }
    
    let hash = hasher.finalize();
    Ok(format!("{:x}", hash))
}

pub fn get_file_metadata(path: &Path) -> Result<(u64, u64)> {
    let metadata = fs::metadata(path)
        .context("Failed to get file metadata")?;
    
    let size = metadata.len();
    let modified = metadata
        .modified()
        .context("Failed to get modification time")?
        .duration_since(SystemTime::UNIX_EPOCH)
        .context("Invalid modification time")?
        .as_secs();
    
    Ok((size, modified))
}

pub fn get_stored_metadata(conn: &Connection, path: &str) -> SqlResult<Option<FileMetadata>> {
    let mut stmt = conn.prepare(
        "SELECT path, hash, size, modified, last_synced FROM file_metadata WHERE path = ?"
    )?;
    
    let mut rows = stmt.query_map([path], |row| {
        Ok(FileMetadata {
            path: row.get(0)?,
            hash: row.get(1)?,
            size: row.get(2)?,
            modified: row.get(3)?,
            last_synced: row.get(4)?,
        })
    })?;
    
    if let Some(row) = rows.next() {
        row.map(Some)
    } else {
        Ok(None)
    }
}

pub fn update_file_metadata(conn: &Connection, metadata: &FileMetadata) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO file_metadata (path, hash, size, modified, last_synced)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![
            metadata.path,
            metadata.hash,
            metadata.size,
            metadata.modified,
            metadata.last_synced,
        ],
    )?;
    Ok(())
}

pub fn has_file_changed(path: &Path) -> Result<bool> {
    let path_str = path.to_string_lossy().to_string();
    let conn = get_connection()?;
    
    // Get current file metadata
    let (current_size, current_modified) = get_file_metadata(path)?;
    
    // Check if we have stored metadata
    if let Some(stored) = get_stored_metadata(&conn, &path_str)? {
        // Quick check: if size or mtime changed, file might have changed
        if stored.size != current_size || stored.modified != current_modified {
            // Calculate hash to be sure
            let current_hash = calculate_file_hash(path)?;
            return Ok(current_hash != stored.hash);
        }
        // Size and mtime match, assume unchanged (optimization)
        return Ok(false);
    }
    
    // No stored metadata, file is new/changed
    Ok(true)
}

pub fn get_all_files_to_sync() -> Result<Vec<PathBuf>> {
    let conn = get_connection()?;
    let mut stmt = conn.prepare(
        "SELECT path FROM tracked_paths"
    )?;
    
    let rows = stmt.query_map([], |row| {
        Ok(PathBuf::from(row.get::<_, String>(0)?))
    })?;
    
    let mut files = Vec::new();
    for row in rows {
        let path = row?;
        collect_files_recursive(&path, &mut files)?;
    }
    
    Ok(files)
}

fn collect_files_recursive(path: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    if path.is_file() {
        files.push(path.to_path_buf());
    } else if path.is_dir() {
        let entries = fs::read_dir(path)
            .context("Failed to read directory")?;
        
        for entry in entries {
            let entry = entry.context("Failed to read directory entry")?;
            let entry_path = entry.path();
            collect_files_recursive(&entry_path, files)?;
        }
    }
    
    Ok(())
}

pub fn mark_file_synced(path: &Path) -> Result<()> {
    let path_str = path.to_string_lossy().to_string();
    let (size, modified) = get_file_metadata(path)?;
    let hash = calculate_file_hash(path)?;
    let last_synced = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .context("Invalid system time")?
        .as_secs();
    
    let conn = get_connection()?;
    let metadata = FileMetadata {
        path: path_str,
        hash,
        size,
        modified,
        last_synced: Some(last_synced),
    };
    
    update_file_metadata(&conn, &metadata)?;
    Ok(())
}

pub fn add_tracked_path(path: &Path) -> Result<()> {
    let conn = get_connection()?;
    let path_str = path.to_string_lossy().to_string();
    let is_directory = path.is_dir();
    
    conn.execute(
        "INSERT OR IGNORE INTO tracked_paths (path, is_directory) VALUES (?1, ?2)",
        rusqlite::params![path_str, is_directory as i32],
    )?;
    
    Ok(())
}

pub fn remove_tracked_path(path: &Path) -> Result<()> {
    let conn = get_connection()?;
    let path_str = path.to_string_lossy().to_string();
    
    conn.execute(
        "DELETE FROM tracked_paths WHERE path = ?1",
        rusqlite::params![path_str],
    )?;
    
    Ok(())
}

pub fn get_tracked_paths() -> Result<Vec<String>> {
    let conn = get_connection()?;
    let mut stmt = conn.prepare("SELECT path FROM tracked_paths ORDER BY path")?;
    
    let rows = stmt.query_map([], |row| {
        Ok(row.get::<_, String>(0)?)
    })?;
    
    let mut paths = Vec::new();
    for row in rows {
        paths.push(row?);
    }
    
    Ok(paths)
}
