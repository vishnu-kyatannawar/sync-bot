use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use chrono::Local;

pub struct Logger {
    log_file: PathBuf,
}

impl Logger {
    pub fn new() -> anyhow::Result<Self> {
        // Create logs directory in the project root
        let log_dir = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?
            .join("projects/personal/sync-bot/logs");
        
        fs::create_dir_all(&log_dir)?;
        
        // Create log file with timestamp
        let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S");
        let log_file = log_dir.join(format!("sync-bot_{}.log", timestamp));
        
        // Create the file
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_file)?;
        
        Ok(Self { log_file })
    }
    
    pub fn log(&self, level: &str, message: &str) {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        let log_line = format!("[{}] [{}] {}\n", timestamp, level, message);
        
        // Write to file
        if let Ok(mut file) = OpenOptions::new().append(true).open(&self.log_file) {
            let _ = file.write_all(log_line.as_bytes());
        }
        
        // Also print to console
        match level {
            "ERROR" => eprintln!("{}", log_line.trim()),
            "WARN" => eprintln!("{}", log_line.trim()),
            _ => println!("{}", log_line.trim()),
        }
    }
    
    pub fn info(&self, message: &str) {
        self.log("INFO", message);
    }
    
    pub fn warn(&self, message: &str) {
        self.log("WARN", message);
    }
    
    pub fn error(&self, message: &str) {
        self.log("ERROR", message);
    }
    
    pub fn debug(&self, message: &str) {
        self.log("DEBUG", message);
    }
}

// Global logger instance
static mut LOGGER: Option<Logger> = None;

pub fn init_logger() -> anyhow::Result<()> {
    unsafe {
        LOGGER = Some(Logger::new()?);
        if let Some(logger) = &LOGGER {
            logger.info("=== Sync Bot Started ===");
            logger.info(&format!("Log file: {:?}", logger.log_file));
        }
    }
    Ok(())
}

pub fn log_info(message: &str) {
    unsafe {
        if let Some(logger) = &LOGGER {
            logger.info(message);
        }
    }
}

pub fn log_warn(message: &str) {
    unsafe {
        if let Some(logger) = &LOGGER {
            logger.warn(message);
        }
    }
}

pub fn log_error(message: &str) {
    unsafe {
        if let Some(logger) = &LOGGER {
            logger.error(message);
        }
    }
}

pub fn log_debug(message: &str) {
    unsafe {
        if let Some(logger) = &LOGGER {
            logger.debug(message);
        }
    }
}
