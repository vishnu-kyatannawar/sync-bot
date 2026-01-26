use std::path::{Path, PathBuf};
use std::fs;
use std::io::Write;
use zip::write::{FileOptions, ZipWriter};
use zip::CompressionMethod;
use chrono::Local;
use anyhow::{Result, Context};

pub fn create_archive(source_dir: &Path, archives_dir: &Path) -> Result<PathBuf> {
    // Create timestamp
    let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S");
    let archive_name = format!("sync-{}.zip", timestamp);
    let archive_path = archives_dir.join(&archive_name);
    
    // Create zip file
    let file = fs::File::create(&archive_path)
        .context("Failed to create archive file")?;
    let mut zip = ZipWriter::new(file);
    
    // Add files to zip
    add_directory_to_zip(&mut zip, source_dir, source_dir, "")
        .context("Failed to add files to archive")?;
    
    zip.finish()
        .context("Failed to finalize archive")?;
    
    // Clean up old archives (keep only last 4)
    cleanup_old_archives(archives_dir)?;
    
    Ok(archive_path)
}

fn add_directory_to_zip(
    zip: &mut ZipWriter<fs::File>,
    base_path: &Path,
    current_path: &Path,
    zip_path: &str,
) -> Result<()> {
    let entries = fs::read_dir(current_path)
        .context("Failed to read directory")?;
    
    for entry in entries {
        let entry = entry.context("Failed to read directory entry")?;
        let path = entry.path();
        let metadata = entry.metadata()
            .context("Failed to get entry metadata")?;
        
        let relative_path = path.strip_prefix(base_path)
            .context("Failed to get relative path")?;
        let zip_entry_path = if zip_path.is_empty() {
            relative_path.to_string_lossy().to_string()
        } else {
            format!("{}/{}", zip_path, relative_path.to_string_lossy())
        };
        
        if metadata.is_file() {
            let mut file = fs::File::open(&path)
                .context("Failed to open file for archiving")?;
            
            zip.start_file(&zip_entry_path, FileOptions::default()
                .compression_method(CompressionMethod::Deflated))?;
            
            std::io::copy(&mut file, zip)
                .context("Failed to write file to archive")?;
        } else if metadata.is_dir() {
            zip.add_directory(&zip_entry_path, FileOptions::default())?;
            add_directory_to_zip(zip, base_path, &path, &zip_entry_path)?;
        }
    }
    
    Ok(())
}

fn cleanup_old_archives(archives_dir: &Path) -> Result<()> {
    let mut archives: Vec<_> = fs::read_dir(archives_dir)
        .context("Failed to read archives directory")?
        .filter_map(|entry| {
            entry.ok().and_then(|e| {
                let path = e.path();
                if path.extension() == Some(std::ffi::OsStr::new("zip")) {
                    e.metadata().ok().and_then(|m| {
                        m.modified().ok().map(|modified| (path, modified))
                    })
                } else {
                    None
                }
            })
        })
        .collect();
    
    // Sort by modification time (newest first)
    archives.sort_by(|a, b| b.1.cmp(&a.1));
    
    // Keep only last 4, delete the rest
    if archives.len() > 4 {
        for (path, _) in archives.iter().skip(4) {
            fs::remove_file(path)
                .context("Failed to remove old archive")?;
        }
    }
    
    Ok(())
}

pub fn get_archive_count(archives_dir: &Path) -> Result<usize> {
    if !archives_dir.exists() {
        return Ok(0);
    }
    
    let count = fs::read_dir(archives_dir)
        .context("Failed to read archives directory")?
        .filter(|entry| {
            entry.as_ref()
                .map(|e| {
                    e.path().extension() == Some(std::ffi::OsStr::new("zip"))
                })
                .unwrap_or(false)
        })
        .count();
    
    Ok(count)
}
