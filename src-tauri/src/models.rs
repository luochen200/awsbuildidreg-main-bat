use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: i64,
    pub email: String,
    pub email_password: String,
    pub client_id: String,
    pub refresh_token: String,
    pub kiro_password: Option<String>,
    pub status: AccountStatus,
    pub error_reason: Option<String>,
    pub oauth_status: OAuthStatus,
    pub oauth_info: Option<String>, // JSON string of OAuth details
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AccountStatus {
    NotRegistered,
    InProgress,
    Registered,
    Error,
}

impl std::fmt::Display for AccountStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AccountStatus::NotRegistered => write!(f, "not_registered"),
            AccountStatus::InProgress => write!(f, "in_progress"),
            AccountStatus::Registered => write!(f, "registered"),
            AccountStatus::Error => write!(f, "error"),
        }
    }
}

impl AccountStatus {
    pub fn from_string(s: &str) -> Self {
        match s {
            "in_progress" => AccountStatus::InProgress,
            "registered" => AccountStatus::Registered,
            "error" => AccountStatus::Error,
            _ => AccountStatus::NotRegistered,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum OAuthStatus {
    NotAuthorized,
    InProgress,
    Authorized,
    Error,
}

impl std::fmt::Display for OAuthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OAuthStatus::NotAuthorized => write!(f, "not_authorized"),
            OAuthStatus::InProgress => write!(f, "in_progress"),
            OAuthStatus::Authorized => write!(f, "authorized"),
            OAuthStatus::Error => write!(f, "error"),
        }
    }
}

impl OAuthStatus {
    pub fn from_string(s: &str) -> Self {
        match s {
            "in_progress" => OAuthStatus::InProgress,
            "authorized" => OAuthStatus::Authorized,
            "error" => OAuthStatus::Error,
            _ => OAuthStatus::NotAuthorized,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewAccount {
    pub email: String,
    pub email_password: String,
    pub client_id: String,
    pub refresh_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountUpdate {
    pub id: i64,
    pub email: Option<String>,
    pub email_password: Option<String>,
    pub client_id: Option<String>,
    pub refresh_token: Option<String>,
    pub kiro_password: Option<String>,
    pub status: Option<AccountStatus>,
    pub error_reason: Option<String>,
    pub oauth_status: Option<OAuthStatus>,
    pub oauth_info: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub browser_mode: BrowserMode,
    pub email_mode: EmailMode,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum BrowserMode {
    Background,
    Foreground,
}

impl std::fmt::Display for BrowserMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BrowserMode::Background => write!(f, "background"),
            BrowserMode::Foreground => write!(f, "foreground"),
        }
    }
}

impl BrowserMode {
    pub fn from_string(s: &str) -> Self {
        match s {
            "foreground" => BrowserMode::Foreground,
            _ => BrowserMode::Background,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EmailMode {
    GraphApi,
    Imap,
}

impl std::fmt::Display for EmailMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EmailMode::GraphApi => write!(f, "graph_api"),
            EmailMode::Imap => write!(f, "imap"),
        }
    }
}

impl EmailMode {
    pub fn from_string(s: &str) -> Self {
        match s {
            "imap" => EmailMode::Imap,
            _ => EmailMode::GraphApi,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    pub success_count: usize,
    pub error_count: usize,
    pub errors: Vec<ImportError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportError {
    pub line_number: usize,
    pub content: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailMessage {
    pub id: String,
    pub received_datetime: String,
    pub sent_datetime: String,
    pub subject: String,
    pub body_content: String,
    pub from_address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserConfig {
    pub mode: BrowserMode,
    pub os: String,
    pub os_version: String,
    pub device_type: String,
    pub language: String,
    pub window_width: u32,
    pub window_height: u32,
}

/// OAuth 授权信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthInfo {
    pub access_token: String,
    pub refresh_token: String,
    pub provider: String,
    pub auth_method: String,
    pub expires_at: String,
    pub profile_arn: Option<String>,
    pub client_id_hash: Option<String>,
    pub region: Option<String>,
    pub authorized_at: String,
    // 客户端注册信息
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub client_expires_at: Option<String>,
}
