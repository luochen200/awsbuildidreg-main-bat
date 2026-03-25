# AWS SSO OIDC 完整指南

## 什么是 AWS SSO OIDC？

AWS SSO OIDC (OpenID Connect) 是 AWS 提供的身份认证服务，用于：
- 获取访问令牌 (access_token)
- 获取刷新令牌 (refresh_token)
- 获取客户端凭据 (client_id, client_secret)

这些凭据可以用来访问 AWS 服务和第三方应用（如 Kiro）。

---

## 方法1：使用 AWS CLI 获取 OIDC 凭据 ⭐推荐

### 步骤1：注册 OIDC 客户端

```bash
# 注册一个新的 OIDC 客户端
aws sso-oidc register-client \
  --client-name "KiroRegistration" \
  --client-type "public" \
  --region us-east-1
```

**输出示例：**
```json
{
    "clientId": "xxxxxxxxxxxxxxxxxxxxxxxx",
    "clientSecret": "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
    "clientIdIssuedAt": 1234567890,
    "clientSecretExpiresAt": 1234567890
}
```

⭐ **这就是你需要的 `clientId` 和 `clientSecret`！**

### 步骤2：启动设备授权流程

```bash
# 使用获取的 clientId 和 clientSecret
aws sso-oidc start-device-authorization \
  --client-id "你的clientId" \
  --client-secret "你的clientSecret" \
  --start-url "https://d-xxxxxxxxxx.awsapps.com/start" \
  --region us-east-1
```

**输出示例：**
```json
{
    "deviceCode": "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx",
    "userCode": "ABCD-EFGH",
    "verificationUri": "https://device.sso.us-east-1.amazonaws.com/",
    "verificationUriComplete": "https://device.sso.us-east-1.amazonaws.com/?user_code=ABCD-EFGH",
    "expiresIn": 600,
    "interval": 5
}
```

### 步骤3：用户授权

1. 打开浏览器访问 `verificationUriComplete`
2. 输入你的 Google 账号登录
3. 授权应用访问

### 步骤4：获取令牌

```bash
# 轮询获取令牌（用户授权后）
aws sso-oidc create-token \
  --client-id "你的clientId" \
  --client-secret "你的clientSecret" \
  --grant-type "urn:ietf:params:oauth:grant-type:device_code" \
  --device-code "步骤2获取的deviceCode" \
  --region us-east-1
```

**输出示例：**
```json
{
    "accessToken": "eyJraWQiOiJ...",
    "tokenType": "Bearer",
    "expiresIn": 28800,
    "refreshToken": "Atzr|IwEBIA...",
    "idToken": "eyJraWQiOiJ..."
}
```

⭐ **这就是你需要的 `refreshToken`！**

---

## 方法2：使用 Rust 实现完整流程 ⭐⭐推荐

### 完整实现代码

```rust
use reqwest::Client;
use serde::{Deserialize, Serialize};
use anyhow::{Result, Context};

// ============ 数据结构 ============

#[derive(Serialize)]
struct RegisterClientRequest {
    #[serde(rename = "clientName")]
    client_name: String,
    #[serde(rename = "clientType")]
    client_type: String,
}

#[derive(Deserialize, Debug)]
struct RegisterClientResponse {
    #[serde(rename = "clientId")]
    pub client_id: String,
    #[serde(rename = "clientSecret")]
    pub client_secret: String,
    #[serde(rename = "clientIdIssuedAt")]
    pub client_id_issued_at: i64,
    #[serde(rename = "clientSecretExpiresAt")]
    pub client_secret_expires_at: i64,
}

#[derive(Serialize)]
struct StartDeviceAuthRequest {
    #[serde(rename = "clientId")]
    client_id: String,
    #[serde(rename = "clientSecret")]
    client_secret: String,
    #[serde(rename = "startUrl")]
    start_url: String,
}

#[derive(Deserialize, Debug)]
struct StartDeviceAuthResponse {
    #[serde(rename = "deviceCode")]
    pub device_code: String,
    #[serde(rename = "userCode")]
    pub user_code: String,
    #[serde(rename = "verificationUri")]
    pub verification_uri: String,
    #[serde(rename = "verificationUriComplete")]
    pub verification_uri_complete: String,
    #[serde(rename = "expiresIn")]
    pub expires_in: i32,
    #[serde(rename = "interval")]
    pub interval: i32,
}

#[derive(Serialize)]
struct CreateTokenRequest {
    #[serde(rename = "clientId")]
    client_id: String,
    #[serde(rename = "clientSecret")]
    client_secret: String,
    #[serde(rename = "grantType")]
    grant_type: String,
    #[serde(rename = "deviceCode")]
    device_code: String,
}

#[derive(Deserialize, Debug)]
struct CreateTokenResponse {
    #[serde(rename = "accessToken")]
    pub access_token: String,
    #[serde(rename = "tokenType")]
    pub token_type: String,
    #[serde(rename = "expiresIn")]
    pub expires_in: i32,
    #[serde(rename = "refreshToken")]
    pub refresh_token: Option<String>,
    #[serde(rename = "idToken")]
    pub id_token: Option<String>,
}

#[derive(Deserialize, Debug)]
struct ErrorResponse {
    error: String,
    #[serde(rename = "error_description")]
    error_description: Option<String>,
}

// ============ AWS SSO OIDC 客户端 ============

pub struct AwsSsoOidcClient {
    client: Client,
    region: String,
}

impl AwsSsoOidcClient {
    pub fn new(region: &str) -> Self {
        Self {
            client: Client::new(),
            region: region.to_string(),
        }
    }
    
    fn get_endpoint(&self) -> String {
        format!("https://oidc.{}.amazonaws.com", self.region)
    }
    
    /// 步骤1：注册 OIDC 客户端
    pub async fn register_client(&self, client_name: &str) -> Result<RegisterClientResponse> {
        let url = format!("{}/client/register", self.get_endpoint());
        
        let request = RegisterClientRequest {
            client_name: client_name.to_string(),
            client_type: "public".to_string(),
        };
        
        let response = self.client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .context("Failed to register client")?;
        
        if !response.status().is_success() {
            let error: ErrorResponse = response.json().await?;
            return Err(anyhow::anyhow!("Register client failed: {} - {:?}", 
                error.error, error.error_description));
        }
        
        let result: RegisterClientResponse = response.json().await?;
        
        println!("✅ 客户端注册成功！");
        println!("   Client ID: {}", result.client_id);
        println!("   Client Secret: {}...", &result.client_secret[..20]);
        
        Ok(result)
    }
    
    /// 步骤2：启动设备授权流程
    pub async fn start_device_authorization(
        &self,
        client_id: &str,
        client_secret: &str,
        start_url: &str,
    ) -> Result<StartDeviceAuthResponse> {
        let url = format!("{}/device_authorization", self.get_endpoint());
        
        let request = StartDeviceAuthRequest {
            client_id: client_id.to_string(),
            client_secret: client_secret.to_string(),
            start_url: start_url.to_string(),
        };
        
        let response = self.client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .context("Failed to start device authorization")?;
        
        if !response.status().is_success() {
            let error: ErrorResponse = response.json().await?;
            return Err(anyhow::anyhow!("Start device auth failed: {} - {:?}", 
                error.error, error.error_description));
        }
        
        let result: StartDeviceAuthResponse = response.json().await?;
        
        println!("✅ 设备授权已启动！");
        println!("   User Code: {}", result.user_code);
        println!("   请访问: {}", result.verification_uri_complete);
        
        Ok(result)
    }
    
    /// 步骤3：创建令牌（轮询直到用户授权）
    pub async fn create_token(
        &self,
        client_id: &str,
        client_secret: &str,
        device_code: &str,
    ) -> Result<CreateTokenResponse> {
        let url = format!("{}/token", self.get_endpoint());
        
        let request = CreateTokenRequest {
            client_id: client_id.to_string(),
            client_secret: client_secret.to_string(),
            grant_type: "urn:ietf:params:oauth:grant-type:device_code".to_string(),
            device_code: device_code.to_string(),
        };
        
        let response = self.client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .context("Failed to create token")?;
        
        if !response.status().is_success() {
            let error: ErrorResponse = response.json().await?;
            
            // authorization_pending 是正常的，表示用户还没授权
            if error.error == "authorization_pending" {
                return Err(anyhow::anyhow!("authorization_pending"));
            }
            
            return Err(anyhow::anyhow!("Create token failed: {} - {:?}", 
                error.error, error.error_description));
        }
        
        let result: CreateTokenResponse = response.json().await?;
        
        println!("✅ 令牌创建成功！");
        println!("   Access Token: {}...", &result.access_token[..20]);
        if let Some(ref refresh_token) = result.refresh_token {
            println!("   Refresh Token: {}...", &refresh_token[..20]);
        }
        
        Ok(result)
    }
    
    /// 完整流程：自动化获取所有凭据
    pub async fn get_credentials_with_browser(
        &self,
        start_url: &str,
        email: &str,
        email_password: &str,
    ) -> Result<(String, String, String)> {
        // 步骤1：注册客户端
        println!("\n📝 步骤1：注册 OIDC 客户端...");
        let client_info = self.register_client("KiroAutoRegistration").await?;
        
        // 步骤2：启动设备授权
        println!("\n🔐 步骤2：启动设备授权...");
        let device_auth = self.start_device_authorization(
            &client_info.client_id,
            &client_info.client_secret,
            start_url,
        ).await?;
        
        // 步骤3：使用浏览器自动化完成授权
        println!("\n🌐 步骤3：打开浏览器进行授权...");
        self.authorize_with_browser(
            &device_auth.verification_uri_complete,
            email,
            email_password,
        ).await?;
        
        // 步骤4：轮询获取令牌
        println!("\n⏳ 步骤4：等待授权完成...");
        let token = self.poll_for_token(
            &client_info.client_id,
            &client_info.client_secret,
            &device_auth.device_code,
            device_auth.interval,
            device_auth.expires_in,
        ).await?;
        
        let refresh_token = token.refresh_token
            .ok_or_else(|| anyhow::anyhow!("No refresh token returned"))?;
        
        println!("\n🎉 成功获取所有凭据！");
        
        Ok((
            client_info.client_id,
            client_info.client_secret,
            refresh_token,
        ))
    }
    
    /// 使用浏览器自动化完成授权
    async fn authorize_with_browser(
        &self,
        verification_url: &str,
        email: &str,
        email_password: &str,
    ) -> Result<()> {
        use crate::browser_automation::BrowserAutomation;
        use crate::models::BrowserConfig;
        
        let config = BrowserConfig {
            mode: crate::models::BrowserMode::Visible,
            os: "Windows".to_string(),
            os_version: "Windows 10".to_string(),
            device_type: "PC".to_string(),
            language: "zh-CN".to_string(),
            window_width: 1280,
            window_height: 800,
        };
        
        let automation = BrowserAutomation::new(config);
        let browser = automation.launch_browser()?;
        let tab = browser.new_tab()?;
        
        // 访问授权页面
        tab.navigate_to(verification_url)?;
        tab.wait_until_navigated()?;
        
        std::thread::sleep(std::time::Duration::from_secs(3));
        
        // 这里需要根据实际的授权页面进行自动化操作
        // 通常是：
        // 1. 输入 Google 账号
        // 2. 输入密码
        // 3. 点击授权按钮
        
        // 等待用户完成授权（或自动化完成）
        println!("   等待授权完成...");
        std::thread::sleep(std::time::Duration::from_secs(10));
        
        Ok(())
    }
    
    /// 轮询获取令牌
    async fn poll_for_token(
        &self,
        client_id: &str,
        client_secret: &str,
        device_code: &str,
        interval: i32,
        expires_in: i32,
    ) -> Result<CreateTokenResponse> {
        let start_time = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs(expires_in as u64);
        let poll_interval = std::time::Duration::from_secs(interval as u64);
        
        loop {
            if start_time.elapsed() > timeout {
                return Err(anyhow::anyhow!("Device authorization timeout"));
            }
            
            match self.create_token(client_id, client_secret, device_code).await {
                Ok(token) => return Ok(token),
                Err(e) => {
                    if e.to_string().contains("authorization_pending") {
                        // 继续等待
                        tokio::time::sleep(poll_interval).await;
                        continue;
                    } else {
                        return Err(e);
                    }
                }
            }
        }
    }
}
```

### 使用示例

```rust
// 在 commands.rs 中添加新命令
#[tauri::command]
pub async fn get_sso_credentials(
    email: String,
    email_password: String,
) -> Result<(String, String, String), String> {
    let client = AwsSsoOidcClient::new("us-east-1");
    
    // AWS SSO 起始 URL（需要根据实际情况配置）
    let start_url = "https://d-xxxxxxxxxx.awsapps.com/start";
    
    let (client_id, client_secret, refresh_token) = client
        .get_credentials_with_browser(start_url, &email, &email_password)
        .await
        .map_err(|e| e.to_string())?;
    
    Ok((client_id, client_secret, refresh_token))
}
```

---

## 方法3：从现有 AWS CLI 会话获取

### 查找现有的 OIDC 客户端

```bash
# 查看 AWS CLI 配置
cat ~/.aws/config

# 查看 SSO 缓存
ls -la ~/.aws/sso/cache/
cat ~/.aws/sso/cache/*.json
```

### 提取信息的脚本

```rust
use std::fs;
use std::path::PathBuf;
use serde_json::Value;

pub fn get_oidc_from_aws_cache() -> Result<(String, String, String)> {
    let home_dir = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))?;
    
    let sso_cache_dir = home_dir.join(".aws").join("sso").join("cache");
    
    for entry in fs::read_dir(sso_cache_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            let content = fs::read_to_string(&path)?;
            let json: Value = serde_json::from_str(&content)?;
            
            // 尝试提取字段（字段名可能不同）
            let client_id = json.get("clientId")
                .or_else(|| json.get("client_id"))
                .and_then(|v| v.as_str());
            
            let client_secret = json.get("clientSecret")
                .or_else(|| json.get("client_secret"))
                .and_then(|v| v.as_str());
            
            let refresh_token = json.get("refreshToken")
                .or_else(|| json.get("refresh_token"))
                .and_then(|v| v.as_str());
            
            if let (Some(cid), Some(cs), Some(rt)) = (client_id, client_secret, refresh_token) {
                return Ok((
                    cid.to_string(),
                    cs.to_string(),
                    rt.to_string(),
                ));
            }
        }
    }
    
    Err(anyhow::anyhow!("OIDC credentials not found in cache"))
}
```

---

## 方法4：使用 AWS SDK for Rust

### 添加依赖

```toml
[dependencies]
aws-config = "1.0"
aws-sdk-ssooidc = "1.0"
tokio = { version = "1", features = ["full"] }
```

### 实现代码

```rust
use aws_sdk_ssooidc::{Client, Region};

pub async fn register_oidc_client() -> Result<(String, String)> {
    let config = aws_config::load_from_env().await;
    let client = Client::new(&config);
    
    let response = client
        .register_client()
        .client_name("KiroRegistration")
        .client_type("public")
        .send()
        .await?;
    
    let client_id = response.client_id()
        .ok_or_else(|| anyhow::anyhow!("No client_id"))?;
    
    let client_secret = response.client_secret()
        .ok_or_else(|| anyhow::anyhow!("No client_secret"))?;
    
    Ok((client_id.to_string(), client_secret.to_string()))
}

pub async fn get_device_code(
    client_id: &str,
    client_secret: &str,
    start_url: &str,
) -> Result<String> {
    let config = aws_config::load_from_env().await;
    let client = Client::new(&config);
    
    let response = client
        .start_device_authorization()
        .client_id(client_id)
        .client_secret(client_secret)
        .start_url(start_url)
        .send()
        .await?;
    
    println!("请访问: {}", response.verification_uri_complete().unwrap_or(""));
    println!("用户代码: {}", response.user_code().unwrap_or(""));
    
    Ok(response.device_code().unwrap_or("").to_string())
}

pub async fn get_token(
    client_id: &str,
    client_secret: &str,
    device_code: &str,
) -> Result<String> {
    let config = aws_config::load_from_env().await;
    let client = Client::new(&config);
    
    // 轮询直到用户授权
    loop {
        match client
            .create_token()
            .client_id(client_id)
            .client_secret(client_secret)
            .grant_type("urn:ietf:params:oauth:grant-type:device_code")
            .device_code(device_code)
            .send()
            .await
        {
            Ok(response) => {
                if let Some(refresh_token) = response.refresh_token() {
                    return Ok(refresh_token.to_string());
                }
            }
            Err(e) => {
                // 如果是 authorization_pending，继续等待
                if e.to_string().contains("AuthorizationPendingException") {
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    continue;
                }
                return Err(e.into());
            }
        }
    }
}
```

---

## 完整工作流程

```
1. 注册 OIDC 客户端
   ↓
   获得: clientId, clientSecret
   
2. 启动设备授权
   ↓
   获得: deviceCode, verificationUrl
   
3. 用户在浏览器中授权
   ↓
   输入 Google 账号密码
   
4. 轮询获取令牌
   ↓
   获得: accessToken, refreshToken
   
5. 使用这些凭据注册 Kiro
```

---

## 推荐实现方案

### 🥇 方案：完全自动化（推荐）

1. 使用方法2的 Rust 实现
2. 集成到现有的浏览器自动化流程
3. 一键获取所有凭据

### 实现步骤

1. 我帮你把上面的代码集成到项目中
2. 添加一个新的命令 `get_sso_credentials`
3. 在注册前自动获取凭据
4. 完全自动化，用户无需手动操作

---

## 需要我帮你实现吗？

我可以：
1. ✅ 创建 `aws_sso_oidc.rs` 模块
2. ✅ 集成到现有代码
3. ✅ 添加 Tauri 命令
4. ✅ 实现完全自动化流程

你想要我现在就开始实现吗？
