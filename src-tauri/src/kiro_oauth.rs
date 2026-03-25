// Kiro IDE 真实 OAuth 登录实现
// 支持 Social (Google) 和 IdC (AWS Builder ID) 两种方式

use crate::kiro_auth::{self, IdcAuthParams, SocialAuthParams};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;
use tokio::sync::mpsc::{Receiver as TokioReceiver, Sender as TokioSender};

// ===== OAuth 回调处理 =====

/// OAuth 回调结果
#[derive(Debug, Clone)]
pub struct OAuthCallbackResult {
    pub code: String,
    #[allow(dead_code)]
    pub state: String,
}

/// 全局回调发送器存储
type OAuthCallbackSender = TokioSender<Result<OAuthCallbackResult, String>>;
type PendingSenderType = Mutex<Option<(String, OAuthCallbackSender)>>;
type ResultReceiverType = Arc<Mutex<Option<TokioReceiver<Result<OAuthCallbackResult, String>>>>>;

static PENDING_SENDER: OnceLock<PendingSenderType> = OnceLock::new();

/// 注册一个新的回调等待器
pub fn register_oauth_waiter(state: &str) -> OAuthCallbackWaiter {
    let (tx, rx) = tokio::sync::mpsc::channel(1);

    let storage = PENDING_SENDER.get_or_init(|| Mutex::new(None));
    *storage.lock().unwrap() = Some((state.to_string(), tx));

    OAuthCallbackWaiter {
        result_rx: Arc::new(Mutex::new(Some(rx))),
        timeout: Duration::from_secs(300), // 5 分钟超时
    }
}

/// OAuth 回调等待器
pub struct OAuthCallbackWaiter {
    result_rx: ResultReceiverType,
    timeout: Duration,
}

impl OAuthCallbackWaiter {
    /// 等待回调结果
    pub async fn wait_for_callback(&self) -> Result<OAuthCallbackResult, String> {
        let mut rx = self
            .result_rx
            .lock()
            .unwrap()
            .take()
            .ok_or("Callback channel already consumed")?;

        match tokio::time::timeout(self.timeout, rx.recv()).await {
            Ok(Some(result)) => result,
            Ok(None) => Err("OAuth callback channel closed".to_string()),
            Err(_) => Err("OAuth callback timeout (5 minutes)".to_string()),
        }
    }
}

/// 处理 deep link URL（由 main.rs 调用）
pub fn handle_oauth_callback(url: &str) -> bool {
    println!("[OAuth] Processing callback URL: {}", url);

    let storage = match PENDING_SENDER.get() {
        Some(s) => s,
        None => {
            println!("[OAuth] No pending sender");
            return false;
        }
    };

    let mut guard = storage.lock().unwrap();
    let (expected_state, tx) = match guard.take() {
        Some(s) => s,
        None => {
            println!("[OAuth] No pending waiter");
            return false;
        }
    };

    // 解析 URL
    let parsed = match url::Url::parse(url) {
        Ok(u) => u,
        Err(e) => {
            println!("[OAuth] URL parse error: {}", e);
            std::mem::drop(tx.send(Err(format!("Invalid URL: {}", e))));
            return false;
        }
    };

    // 检查是否是 kiro:// 协议
    if parsed.scheme() != "kiro" {
        println!("[OAuth] Not kiro:// scheme");
        *guard = Some((expected_state, tx));
        return false;
    }

    // 提取参数
    let params: std::collections::HashMap<_, _> = parsed.query_pairs().collect();

    // 检查错误
    if let Some(error) = params.get("error") {
        let desc = params
            .get("error_description")
            .map(|s| s.to_string())
            .unwrap_or_else(|| "Unknown error".to_string());
        println!("[OAuth] OAuth error: {} - {}", error, desc);
        std::mem::drop(tx.send(Err(format!("OAuth error: {} - {}", error, desc))));
        return true;
    }

    let code = match params.get("code") {
        Some(c) => c.to_string(),
        None => {
            println!("[OAuth] Missing code parameter");
            std::mem::drop(tx.send(Err("Missing code parameter".to_string())));
            return true;
        }
    };

    let state = match params.get("state") {
        Some(s) => s.to_string(),
        None => {
            println!("[OAuth] Missing state parameter");
            std::mem::drop(tx.send(Err("Missing state parameter".to_string())));
            return true;
        }
    };

    // 验证 state
    if state != expected_state {
        println!(
            "[OAuth] State mismatch: expected {}, got {}",
            expected_state, state
        );
        std::mem::drop(tx.send(Err("State mismatch - possible CSRF attack".to_string())));
        return true;
    }

    println!(
        "[OAuth] Callback success, code: {}...",
        &code[..20.min(code.len())]
    );
    std::mem::drop(tx.send(Ok(OAuthCallbackResult { code, state })));
    true
}

// ===== PKCE 工具函数 =====

/// 生成 PKCE code_verifier（32字节，base64url）
pub fn generate_code_verifier() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let bytes: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
    base64_url_encode(&bytes)
}

/// 生成 PKCE code_challenge（SHA256哈希，base64url）
pub fn generate_code_challenge(verifier: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(verifier.as_bytes());
    let result = hasher.finalize();
    base64_url_encode(&result)
}

fn base64_url_encode(data: &[u8]) -> String {
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
    URL_SAFE_NO_PAD.encode(data)
}

// ===== Kiro Auth Service Client =====

const KIRO_AUTH_API: &str = "https://prod.us-east-1.auth.desktop.kiro.dev";

/// Token 响应
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub profile_arn: Option<String>,
    pub expires_in: i64,
    #[allow(dead_code)]
    pub id_token: Option<String>,
    #[allow(dead_code)]
    pub csrf_token: Option<String>,
}

/// 打开浏览器进行 OAuth 登录
pub async fn open_browser_for_oauth(
    provider: &str,
    redirect_uri: &str,
    code_challenge: &str,
    state: &str,
) -> Result<(), String> {
    let login_url = format!(
        "{}/login?idp={}&redirect_uri={}&code_challenge={}&code_challenge_method=S256&state={}",
        KIRO_AUTH_API,
        provider,
        urlencoding::encode(redirect_uri),
        code_challenge,
        state,
    );

    println!("\n[OAuth] Opening browser for {} login", provider);
    println!("URL: {}", login_url);

    // 使用系统默认浏览器打开
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("rundll32")
            .args(["url.dll,FileProtocolHandler", &login_url])
            .spawn()
            .map_err(|e| format!("Failed to open browser: {}", e))?;
    }

    #[cfg(not(target_os = "windows"))]
    {
        std::process::Command::new("open")
            .arg(&login_url)
            .spawn()
            .map_err(|e| format!("Failed to open browser: {}", e))?;
    }

    Ok(())
}

/// 交换授权码为 Token
pub async fn exchange_code_for_token(
    code: &str,
    code_verifier: &str,
    redirect_uri: &str,
) -> Result<TokenResponse, String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| format!("Failed to create client: {}", e))?;

    let body = serde_json::json!({
        "code": code,
        "code_verifier": code_verifier,
        "redirect_uri": redirect_uri,
    });

    println!("\n[OAuth] Exchanging code for tokens...");

    let response = client
        .post(format!("{}/oauth/token", KIRO_AUTH_API))
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Token exchange request failed: {}", e))?;

    let status = response.status();
    let text = response.text().await.unwrap_or_default();

    if !status.is_success() {
        println!("[OAuth] Token exchange failed: {}", text);
        return Err(format!("Token exchange failed ({}): {}", status, text));
    }

    println!("[OAuth] Token exchange successful");

    serde_json::from_str(&text).map_err(|e| format!("Failed to parse token response: {}", e))
}

// ===== 完整的 Kiro OAuth 登录流程 =====

/// Kiro Social OAuth 登录结果
#[derive(Debug, Clone, Serialize)]
pub struct KiroOAuthResult {
    pub access_token: String,
    pub refresh_token: String,
    pub provider: String,
    pub profile_arn: Option<String>,
    pub expires_at: String,
}

/// 执行完整的 Kiro Social OAuth 登录流程
pub async fn perform_kiro_social_login(provider: &str) -> Result<KiroOAuthResult, String> {
    println!("\n========== Kiro {} OAuth Login ==========", provider);

    // 1. 准备 OAuth 参数
    let redirect_uri = "kiro://kiro.kiroAgent/authenticate-success";
    let state = uuid::Uuid::new_v4().to_string();
    let code_verifier = generate_code_verifier();
    let code_challenge = generate_code_challenge(&code_verifier);

    println!("[OAuth] State: {}", state);
    println!("[OAuth] Redirect URI: {}", redirect_uri);

    // 2. 注册回调等待器
    let waiter = register_oauth_waiter(&state);

    // 3. 打开浏览器
    open_browser_for_oauth(provider, redirect_uri, &code_challenge, &state).await?;

    // 4. 等待用户完成登录并回调
    println!("[OAuth] Waiting for user to complete login...");
    println!("[OAuth] Please complete the login in your browser");

    let callback = waiter
        .wait_for_callback()
        .await
        .map_err(|e| format!("OAuth callback failed: {}", e))?;

    println!("[OAuth] Callback received!");

    // 5. 交换授权码为 Token
    let token_response =
        exchange_code_for_token(&callback.code, &code_verifier, redirect_uri).await?;

    // 6. 计算过期时间
    let expires_at = chrono::Utc::now() + chrono::Duration::seconds(token_response.expires_in);

    // 7. 生成授权文件
    let params = SocialAuthParams {
        access_token: token_response.access_token.clone(),
        refresh_token: token_response.refresh_token.clone(),
        provider: provider.to_string(),
        profile_arn: token_response.profile_arn.clone(),
    };

    kiro_auth::generate_kiro_social_auth(params)?;

    println!("[OAuth] ✓ Kiro {} login completed successfully!", provider);
    println!("[OAuth] Authorization file saved to ~/.aws/sso/cache/kiro-auth-token.json");

    Ok(KiroOAuthResult {
        access_token: token_response.access_token,
        refresh_token: token_response.refresh_token,
        provider: provider.to_string(),
        profile_arn: token_response.profile_arn,
        expires_at: expires_at.to_rfc3339(),
    })
}

// ===== AWS SSO OIDC (Builder ID) 登录 =====

/// AWS SSO OIDC 客户端
pub struct AwsSsoClient {
    #[allow(dead_code)]
    region: String,
    base_url: String,
    client: reqwest::Client,
}

impl AwsSsoClient {
    pub fn new(region: &str) -> Self {
        let base_url = format!("https://oidc.{}.amazonaws.com", region);
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            region: region.to_string(),
            base_url,
            client,
        }
    }

    /// 注册设备授权客户端
    pub async fn register_device_client(&self) -> Result<ClientRegistration, String> {
        let url = format!("{}/client/register", self.base_url);

        let body = serde_json::json!({
            "clientName": "Kiro IDE Auto Registration",
            "clientType": "public",
            "scopes": [
                "codewhisperer:completions",
                "codewhisperer:analysis",
                "codewhisperer:conversations",
                "codewhisperer:transformations",
                "codewhisperer:taskassist"
            ],
            "grantTypes": ["urn:ietf:params:oauth:grant-type:device_code", "refresh_token"],
            "issuerUrl": "https://oidc.us-east-1.amazonaws.com"
        });

        println!("\n[AWS SSO] Registering device client...");

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Client registration failed: {}", e))?;

        let status = response.status();
        let text = response.text().await.unwrap_or_default();

        if !status.is_success() {
            return Err(format!("Client registration failed ({}): {}", status, text));
        }

        println!("[AWS SSO] Client registered successfully");

        serde_json::from_str(&text).map_err(|e| format!("Failed to parse registration: {}", e))
    }

    /// 发起设备授权
    pub async fn start_device_authorization(
        &self,
        client_id: &str,
        client_secret: &str,
    ) -> Result<DeviceAuthorizationResponse, String> {
        let url = format!("{}/device_authorization", self.base_url);

        let body = serde_json::json!({
            "clientId": client_id,
            "clientSecret": client_secret,
            "startUrl": "https://view.awsapps.com/start"
        });

        println!("\n[AWS SSO] Starting device authorization...");

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Device authorization failed: {}", e))?;

        let status = response.status();
        let text = response.text().await.unwrap_or_default();

        if !status.is_success() {
            return Err(format!(
                "Device authorization failed ({}): {}",
                status, text
            ));
        }

        let auth_response: DeviceAuthorizationResponse = serde_json::from_str(&text)
            .map_err(|e| format!("Failed to parse device authorization: {}", e))?;

        println!("[AWS SSO] Device authorization started");
        println!("[AWS SSO] User code: {}", auth_response.user_code);
        println!(
            "[AWS SSO] Verification URL: {}",
            auth_response.verification_uri
        );

        Ok(auth_response)
    }

    /// 轮询获取 Token
    pub async fn poll_for_token(
        &self,
        client_id: &str,
        client_secret: &str,
        device_code: &str,
        interval: u64,
    ) -> Result<DeviceTokenResponse, String> {
        let url = format!("{}/token", self.base_url);

        let body = serde_json::json!({
            "clientId": client_id,
            "clientSecret": client_secret,
            "grantType": "urn:ietf:params:oauth:grant-type:device_code",
            "deviceCode": device_code
        });

        println!("[AWS SSO] Polling for token...");

        loop {
            tokio::time::sleep(Duration::from_secs(interval)).await;

            let response = self
                .client
                .post(&url)
                .header("Content-Type", "application/json")
                .json(&body)
                .send()
                .await
                .map_err(|e| format!("Token poll failed: {}", e))?;

            let status = response.status();
            let text = response.text().await.unwrap_or_default();

            if status.is_success() {
                println!("[AWS SSO] Token received!");
                return serde_json::from_str(&text)
                    .map_err(|e| format!("Failed to parse token: {}", e));
            }

            // 解析错误
            if let Ok(error) = serde_json::from_str::<serde_json::Value>(&text) {
                if let Some(error_type) = error.get("error").and_then(|e| e.as_str()) {
                    match error_type {
                        "authorization_pending" => {
                            print!(".");
                            std::io::Write::flush(&mut std::io::stdout()).ok();
                            continue;
                        }
                        "slow_down" => {
                            tokio::time::sleep(Duration::from_secs(5)).await;
                            continue;
                        }
                        "expired_token" => {
                            return Err("Device code expired".to_string());
                        }
                        "access_denied" => {
                            return Err("User denied authorization".to_string());
                        }
                        _ => {
                            return Err(format!("Authorization error: {}", error_type));
                        }
                    }
                }
            }

            return Err(format!("Token poll failed ({}): {}", status, text));
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct ClientRegistration {
    pub client_id: String,
    pub client_secret: String,
    pub client_id_issued_at: Option<i64>,
    pub client_secret_expires_at: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct DeviceAuthorizationResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub verification_uri_complete: Option<String>,
    pub expires_in: i64,
    pub interval: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct DeviceTokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub id_token: Option<String>,
    pub token_type: Option<String>,
    pub expires_in: i64,
}

/// 执行完整的 AWS Builder ID 登录流程
pub async fn perform_builder_id_login() -> Result<KiroOAuthResult, String> {
    println!("\n========== AWS Builder ID Login ==========");

    let region = "us-east-1";
    let client = AwsSsoClient::new(region);

    // 1. 注册客户端
    let registration = client.register_device_client().await?;

    // 2. 发起设备授权
    let auth = client
        .start_device_authorization(&registration.client_id, &registration.client_secret)
        .await?;

    // 3. 打开浏览器
    let verification_url = auth
        .verification_uri_complete
        .as_ref()
        .unwrap_or(&auth.verification_uri);

    println!("\n[AWS SSO] Opening browser for authorization...");
    println!("[AWS SSO] Please authorize in your browser");

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("rundll32")
            .args(["url.dll,FileProtocolHandler", verification_url])
            .spawn()
            .map_err(|e| format!("Failed to open browser: {}", e))?;
    }

    #[cfg(not(target_os = "windows"))]
    {
        std::process::Command::new("open")
            .arg(verification_url)
            .spawn()
            .map_err(|e| format!("Failed to open browser: {}", e))?;
    }

    // 4. 轮询获取 Token
    let interval = auth.interval.unwrap_or(5);
    let token = client
        .poll_for_token(
            &registration.client_id,
            &registration.client_secret,
            &auth.device_code,
            interval as u64,
        )
        .await?;

    // 5. 生成 clientIdHash
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(registration.client_id.as_bytes());
    let client_id_hash = hex::encode(hasher.finalize());

    // 6. 计算过期时间
    let expires_at = chrono::Utc::now() + chrono::Duration::seconds(token.expires_in);

    // 7. 生成授权文件
    let params = IdcAuthParams {
        access_token: token.access_token.clone(),
        refresh_token: token.refresh_token.clone(),
        provider: "BuilderId".to_string(),
        client_id: registration.client_id,
        client_secret: registration.client_secret,
        client_id_hash,
        region: region.to_string(),
    };

    kiro_auth::generate_kiro_idc_auth(params)?;

    println!("\n[AWS SSO] ✓ Builder ID login completed successfully!");
    println!("[AWS SSO] Authorization files saved to ~/.aws/sso/cache/");

    Ok(KiroOAuthResult {
        access_token: token.access_token,
        refresh_token: token.refresh_token,
        provider: "BuilderId".to_string(),
        profile_arn: None,
        expires_at: expires_at.to_rfc3339(),
    })
}
