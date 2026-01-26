use std::path::Path;
use std::fs;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use anyhow::{Result, Context};

const GOOGLE_OAUTH_AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const GOOGLE_OAUTH_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const GOOGLE_DRIVE_API_BASE: &str = "https://www.googleapis.com/drive/v3";

// Standard loopback URI for desktop apps
pub const REDIRECT_PORT: u16 = 14242;
pub const REDIRECT_URI: &str = "http://localhost:14242";

// Default placeholders
const DEFAULT_CLIENT_ID: &str = "YOUR_CLIENT_ID";
const DEFAULT_CLIENT_SECRET: &str = "YOUR_CLIENT_SECRET";

#[derive(Debug, Serialize, Deserialize)]
struct TokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: Option<u64>,
    token_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct DriveFile {
    id: String,
    name: String,
    #[serde(rename = "mimeType")]
    mime_type: String,
    parents: Option<Vec<String>>,
}

pub struct DriveSync {
    client: Client,
    access_token: Option<String>,
    refresh_token: Option<String>,
}

impl DriveSync {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            access_token: None,
            refresh_token: None,
        }
    }

    pub fn get_auth_url() -> Result<String> {
        let config = crate::config::load_config()?;
        let client_id = config.client_id
            .or_else(|| std::env::var("GOOGLE_CLIENT_ID").ok())
            .unwrap_or_else(|| DEFAULT_CLIENT_ID.to_string());
        
        if client_id == DEFAULT_CLIENT_ID {
            anyhow::bail!("Please set Google Client ID in settings");
        }
        
        let scopes = "https://www.googleapis.com/auth/drive.file";
        let redirect_uri = urlencoding::encode(REDIRECT_URI);
        let scope = urlencoding::encode(scopes);
        
        let url = format!(
            "{}?client_id={}&redirect_uri={}&response_type=code&scope={}&access_type=offline&prompt=consent",
            GOOGLE_OAUTH_AUTH_URL, client_id, redirect_uri, scope
        );
        
        Ok(url)
    }
    
    pub async fn handle_oauth_callback(&mut self, code: &str) -> Result<()> {
        self.exchange_code_for_token(code).await
    }

    pub async fn exchange_code_for_token(&mut self, code: &str) -> Result<()> {
        let config = crate::config::load_config()?;
        let client_id = config.client_id
            .or_else(|| std::env::var("GOOGLE_CLIENT_ID").ok())
            .unwrap_or_else(|| DEFAULT_CLIENT_ID.to_string());
        let client_secret = config.client_secret
            .or_else(|| std::env::var("GOOGLE_CLIENT_SECRET").ok())
            .unwrap_or_else(|| DEFAULT_CLIENT_SECRET.to_string());
        
        let params = [
            ("code", code),
            ("client_id", &client_id),
            ("client_secret", &client_secret),
            ("redirect_uri", REDIRECT_URI),
            ("grant_type", "authorization_code"),
        ];
        
        let response = self.client
            .post(GOOGLE_OAUTH_TOKEN_URL)
            .form(&params)
            .send()
            .await
            .context("Failed to exchange code for token")?;
        
        let token_data: TokenResponse = response.json().await
            .context("Failed to parse token response")?;
        
        self.access_token = Some(token_data.access_token);
        self.refresh_token = token_data.refresh_token;
        
        // Store tokens securely (in production, use Tauri's secure storage)
        self.save_tokens()?;
        
        Ok(())
    }

    pub async fn refresh_access_token(&mut self) -> Result<()> {
        let refresh_token = self.refresh_token.as_ref()
            .context("No refresh token available")?;
        
        let config = crate::config::load_config()?;
        let client_id = config.client_id
            .or_else(|| std::env::var("GOOGLE_CLIENT_ID").ok())
            .unwrap_or_else(|| DEFAULT_CLIENT_ID.to_string());
        let client_secret = config.client_secret
            .or_else(|| std::env::var("GOOGLE_CLIENT_SECRET").ok())
            .unwrap_or_else(|| DEFAULT_CLIENT_SECRET.to_string());
        
        let params = [
            ("refresh_token", refresh_token),
            ("client_id", &client_id),
            ("client_secret", &client_secret),
            ("grant_type", &"refresh_token".to_string()),
        ];
        
        let response = self.client
            .post(GOOGLE_OAUTH_TOKEN_URL)
            .form(&params)
            .send()
            .await
            .context("Failed to refresh token")?;
        
        let token_data: TokenResponse = response.json().await
            .context("Failed to parse token response")?;
        
        self.access_token = Some(token_data.access_token);
        self.save_tokens()?;
        
        Ok(())
    }

    pub async fn ensure_authenticated(&mut self) -> Result<()> {
        if self.access_token.is_none() {
            self.load_tokens()?;
        }
        
        if self.access_token.is_none() {
            anyhow::bail!("Not authenticated. Please authenticate first.");
        }
        
        // Try a simple API call to check if token is valid
        if let Err(_) = self.test_connection().await {
            // Token might be expired, try to refresh
            if self.refresh_token.is_some() {
                self.refresh_access_token().await?;
            } else {
                anyhow::bail!("Token expired and no refresh token available");
            }
        }
        
        Ok(())
    }

    async fn test_connection(&self) -> Result<()> {
        let token = self.access_token.as_ref()
            .context("No access token")?;
        
        let response = self.client
            .get(&format!("{}/about", GOOGLE_DRIVE_API_BASE))
            .bearer_auth(token)
            .query(&[("fields", "user")])
            .send()
            .await
            .context("Failed to test connection")?;
        
        if !response.status().is_success() {
            anyhow::bail!("Connection test failed");
        }
        
        Ok(())
    }

    pub async fn find_or_create_folder(&mut self, folder_name: &str) -> Result<String> {
        self.ensure_authenticated().await?;
        let token = self.access_token.as_ref().unwrap();
        
        // Search for existing folder
        let query = format!("name='{}' and mimeType='application/vnd.google-apps.folder' and trashed=false", folder_name);
        let response = self.client
            .get(&format!("{}/files", GOOGLE_DRIVE_API_BASE))
            .bearer_auth(token)
            .query(&[("q", &query), ("fields", &"files(id,name)".to_string())])
            .send()
            .await
            .context("Failed to search for folder")?;
        
        let data: serde_json::Value = response.json().await
            .context("Failed to parse folder search response")?;
        
        if let Some(files) = data.get("files").and_then(|f| f.as_array()) {
            if let Some(first) = files.first() {
                if let Some(id) = first.get("id").and_then(|i| i.as_str()) {
                    return Ok(id.to_string());
                }
            }
        }
        
        // Folder doesn't exist, create it
        let folder_data = serde_json::json!({
            "name": folder_name,
            "mimeType": "application/vnd.google-apps.folder"
        });
        
        let response = self.client
            .post(&format!("{}/files", GOOGLE_DRIVE_API_BASE))
            .bearer_auth(token)
            .json(&folder_data)
            .send()
            .await
            .context("Failed to create folder")?;
        
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Failed to create folder: {} - {}", status, error_text);
        }
        
        let folder_data: serde_json::Value = response.json().await
            .context("Failed to parse folder creation response")?;
        
        let folder_id = folder_data.get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("No 'id' field in folder creation response: {:?}", folder_data))?;
        
        Ok(folder_id.to_string())
    }

    pub async fn find_or_create_subfolder(&mut self, parent_id: &str, folder_name: &str) -> Result<String> {
        self.ensure_authenticated().await?;
        let token = self.access_token.as_ref().unwrap();
        
        // Search for existing folder in parent
        let query = format!("name='{}' and '{}' in parents and mimeType='application/vnd.google-apps.folder' and trashed=false", 
            folder_name.replace("'", "\\'"), parent_id);
        
        let response = self.client
            .get(&format!("{}/files", GOOGLE_DRIVE_API_BASE))
            .bearer_auth(token)
            .query(&[("q", &query), ("fields", &"files(id,name)".to_string())])
            .send()
            .await
            .context("Failed to search for subfolder")?;
        
        let data: serde_json::Value = response.json().await
            .context("Failed to parse subfolder search response")?;
        
        if let Some(files) = data.get("files").and_then(|f| f.as_array()) {
            if let Some(first) = files.first() {
                if let Some(id) = first.get("id").and_then(|i| i.as_str()) {
                    return Ok(id.to_string());
                }
            }
        }
        
        // Folder doesn't exist, create it
        let folder_data = serde_json::json!({
            "name": folder_name,
            "mimeType": "application/vnd.google-apps.folder",
            "parents": [parent_id]
        });
        
        let response = self.client
            .post(&format!("{}/files", GOOGLE_DRIVE_API_BASE))
            .bearer_auth(token)
            .json(&folder_data)
            .send()
            .await
            .context("Failed to create subfolder")?;
        
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Failed to create subfolder: {} - {}", status, error_text);
        }
        
        let folder_data: serde_json::Value = response.json().await
            .context("Failed to parse subfolder creation response")?;
        
        let folder_id = folder_data.get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("No 'id' field in subfolder creation response: {:?}", folder_data))?;
        
        Ok(folder_id.to_string())
    }

    pub async fn get_folder_id_for_path(&mut self, root_id: &str, relative_path: &Path) -> Result<String> {
        let mut current_id = root_id.to_string();
        
        for component in relative_path.components() {
            if let Some(name) = component.as_os_str().to_str() {
                if name == ".." || name == "/" || name == "." {
                    continue;
                }
                current_id = self.find_or_create_subfolder(&current_id, name).await?;
            }
        }
        
        Ok(current_id)
    }

    pub async fn upload_file(&mut self, file_path: &Path, parent_folder_id: &str) -> Result<String> {
        self.ensure_authenticated().await?;
        let token = self.access_token.as_ref().unwrap();
        
        let file_name = file_path.file_name()
            .and_then(|n| n.to_str())
            .context("Invalid file name")?;
        
        // Determine MIME type based on file extension
        let mime_type = file_path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| match ext.to_lowercase().as_str() {
                "zip" => "application/zip",
                "json" => "application/json",
                "txt" => "text/plain",
                "html" => "text/html",
                "css" => "text/css",
                "js" => "application/javascript",
                "png" => "image/png",
                "jpg" | "jpeg" => "image/jpeg",
                "pdf" => "application/pdf",
                _ => "application/octet-stream",
            })
            .unwrap_or("application/octet-stream");
        
        crate::logger::log_info(&format!("Uploading {} with MIME type: {}", file_name, mime_type));
        
        // Check if file already exists
        let query = format!("name='{}' and '{}' in parents and trashed=false", 
            file_name.replace("'", "\\'"), parent_folder_id);
        let response = self.client
            .get(&format!("{}/files", GOOGLE_DRIVE_API_BASE))
            .bearer_auth(token)
            .query(&[("q", &query), ("fields", &"files(id)".to_string())])
            .send()
            .await
            .context("Failed to check for existing file")?;
        
        let data: serde_json::Value = response.json().await
            .context("Failed to parse file check response")?;
        
        let file_id = if let Some(files) = data.get("files").and_then(|f| f.as_array()) {
            files.first()
                .and_then(|f| f.get("id"))
                .and_then(|i| i.as_str())
                .map(|s| s.to_string())
        } else {
            None
        };
        
        // Read file
        let file_data = fs::read(file_path)
            .context("Failed to read file")?;
        
        // Upload or update file
        if let Some(existing_id) = file_id {
            // Update existing file
            let url = format!("https://www.googleapis.com/upload/drive/v3/files/{}?uploadType=media", existing_id);
            let response = self.client
                .patch(&url)
                .bearer_auth(token)
                .header("Content-Type", mime_type)
                .body(file_data)
                .send()
                .await
                .context("Failed to update file")?;
            
            if !response.status().is_success() {
                let status = response.status();
                let error_text = response.text().await.unwrap_or_default();
                anyhow::bail!("Failed to update file: {} - {}", status, error_text);
            }
            
            Ok(existing_id)
        } else {
            // Create new file using multipart upload
            let metadata = serde_json::json!({
                "name": file_name,
                "parents": [parent_folder_id]
            });
            
            let metadata_part = reqwest::multipart::Part::text(serde_json::to_string(&metadata)?)
                .mime_str("application/json; charset=UTF-8")?;
            
            let file_part = reqwest::multipart::Part::bytes(file_data)
                .file_name(file_name.to_string())
                .mime_str(mime_type)?;

            let form = reqwest::multipart::Form::new()
                .part("metadata", metadata_part)
                .part("file", file_part);
            
            let url = "https://www.googleapis.com/upload/drive/v3/files?uploadType=multipart";
            let response = self.client
                .post(url)
                .bearer_auth(token)
                .multipart(form)
                .send()
                .await
                .context("Failed to upload file")?;
            
            // Check status and handle error or success
            let status = response.status();
            if status.is_success() {
                let data: serde_json::Value = response.json().await
                    .context("Failed to parse upload response")?;
                
                let id = data.get("id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("No 'id' field in upload response: {:?}", data))?;
                
                Ok(id.to_string())
            } else {
                let status_code = status.as_u16();
                let error_text = response.text().await.unwrap_or_default();
                anyhow::bail!("Failed to upload file: {} - {}", status_code, error_text);
            }
        }
    }

    fn save_tokens(&self) -> Result<()> {
        // In production, use Tauri's secure storage
        // For now, save to a file (not secure, but works)
        let data_dir = crate::config::get_data_dir()?;
        let token_file = data_dir.join("tokens.json");
        
        let tokens = serde_json::json!({
            "access_token": self.access_token,
            "refresh_token": self.refresh_token,
        });
        
        fs::write(&token_file, serde_json::to_string_pretty(&tokens)?)
            .context("Failed to save tokens")?;
        
        Ok(())
    }

    fn load_tokens(&mut self) -> Result<()> {
        let data_dir = crate::config::get_data_dir()?;
        let token_file = data_dir.join("tokens.json");
        
        if !token_file.exists() {
            return Ok(());
        }
        
        let content = fs::read_to_string(&token_file)
            .context("Failed to read token file")?;
        
        let tokens: serde_json::Value = serde_json::from_str(&content)
            .context("Failed to parse token file")?;
        
        self.access_token = tokens.get("access_token")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        self.refresh_token = tokens.get("refresh_token")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        
        Ok(())
    }

    pub fn is_authenticated() -> bool {
        let data_dir = match crate::config::get_data_dir() {
            Ok(dir) => dir,
            Err(_) => return false,
        };
        let token_file = data_dir.join("tokens.json");
        
        if !token_file.exists() {
            return false;
        }

        // We check if we have at least a refresh token, which means we can re-authenticate
        if let Ok(content) = fs::read_to_string(&token_file) {
            if let Ok(tokens) = serde_json::from_str::<serde_json::Value>(&content) {
                return tokens.get("refresh_token").map_or(false, |v| !v.is_null());
            }
        }
        
        false
    }
}
