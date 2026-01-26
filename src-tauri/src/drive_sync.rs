use std::path::{Path, PathBuf};
use std::fs;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use anyhow::{Result, Context};

const GOOGLE_OAUTH_AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const GOOGLE_OAUTH_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const GOOGLE_DRIVE_API_BASE: &str = "https://www.googleapis.com/drive/v3";

// These should be set via environment or config in production
// For now, using placeholder - user will need to set these
const DEFAULT_CLIENT_ID: &str = "YOUR_CLIENT_ID";
const DEFAULT_CLIENT_SECRET: &str = "YOUR_CLIENT_SECRET";
const REDIRECT_URI: &str = "urn:ietf:wg:oauth:2.0:oob"; // Use out-of-band for desktop apps

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
        let client_id = std::env::var("GOOGLE_CLIENT_ID")
            .unwrap_or_else(|_| DEFAULT_CLIENT_ID.to_string());
        
        if client_id == DEFAULT_CLIENT_ID {
            anyhow::bail!("Please set GOOGLE_CLIENT_ID environment variable or configure it in the app");
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
        let client_id = std::env::var("GOOGLE_CLIENT_ID")
            .unwrap_or_else(|_| DEFAULT_CLIENT_ID.to_string());
        let client_secret = std::env::var("GOOGLE_CLIENT_SECRET")
            .unwrap_or_else(|_| DEFAULT_CLIENT_SECRET.to_string());
        
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
        
        let client_id = std::env::var("GOOGLE_CLIENT_ID")
            .unwrap_or_else(|_| DEFAULT_CLIENT_ID.to_string());
        let client_secret = std::env::var("GOOGLE_CLIENT_SECRET")
            .unwrap_or_else(|_| DEFAULT_CLIENT_SECRET.to_string());
        
        let params = [
            ("refresh_token", refresh_token),
            ("client_id", &client_id),
            ("client_secret", &client_secret),
            ("grant_type", "refresh_token"),
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
            .query(&[("q", &query), ("fields", "files(id,name)")])
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
        
        let folder: DriveFile = response.json().await
            .context("Failed to parse folder creation response")?;
        
        Ok(folder.id)
    }

    pub async fn upload_file(&mut self, file_path: &Path, folder_id: &str) -> Result<String> {
        self.ensure_authenticated().await?;
        let token = self.access_token.as_ref().unwrap();
        
        let file_name = file_path.file_name()
            .and_then(|n| n.to_str())
            .context("Invalid file name")?;
        
        // Check if file already exists
        let query = format!("name='{}' and '{}' in parents and trashed=false", 
            file_name.replace("'", "\\'"), folder_id);
        let response = self.client
            .get(&format!("{}/files", GOOGLE_DRIVE_API_BASE))
            .bearer_auth(token)
            .query(&[("q", &query), ("fields", "files(id)")])
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
            let url = format!("{}/upload/drive/v3/files/{}", GOOGLE_DRIVE_API_BASE, existing_id);
            let response = self.client
                .patch(&url)
                .bearer_auth(token)
                .header("Content-Type", "application/octet-stream")
                .body(file_data)
                .send()
                .await
                .context("Failed to update file")?;
            
            if !response.status().is_success() {
                anyhow::bail!("Failed to update file: {}", response.status());
            }
            
            Ok(existing_id)
        } else {
            // Create new file using multipart upload
            let metadata = serde_json::json!({
                "name": file_name,
                "parents": [folder_id]
            });
            
            let form = reqwest::multipart::Form::new()
                .text("metadata", serde_json::to_string(&metadata)?)
                .part("file", reqwest::multipart::Part::bytes(file_data)
                    .file_name(file_name.to_string())
                    .mime_str("application/octet-stream")?);
            
            let url = format!("{}/upload/drive/v3/files?uploadType=multipart", GOOGLE_DRIVE_API_BASE);
            let response = self.client
                .post(&url)
                .bearer_auth(token)
                .multipart(form)
                .send()
                .await
                .context("Failed to upload file")?;
            
            if !response.status().is_success() {
                let error_text = response.text().await.unwrap_or_default();
                anyhow::bail!("Failed to upload file: {} - {}", response.status(), error_text);
            }
            
            let file: DriveFile = response.json().await
                .context("Failed to parse upload response")?;
            
            Ok(file.id)
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
}
