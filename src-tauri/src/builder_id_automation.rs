// AWS Builder ID 自动化登录
// 使用浏览器自动化完成整个 OAuth 流程

use crate::browser_automation::BrowserAutomation;
use crate::kiro_auth::{generate_kiro_idc_auth, IdcAuthParams};
use crate::models::{BrowserConfig, BrowserMode};
use anyhow::{anyhow, Context, Result};

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Builder ID 自动登录结果
#[derive(Debug, Clone, Serialize)]
pub struct BuilderIdLoginResult {
    pub access_token: String,
    pub refresh_token: String,
    pub client_id: String,
    pub client_secret: String,
    pub client_id_hash: String,
    pub region: String,
}

/// AWS SSO OIDC 客户端注册响应
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ClientRegistration {
    #[serde(rename = "clientId")]
    client_id: String,
    #[serde(rename = "clientSecret")]
    client_secret: String,
}

/// 设备授权响应
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct DeviceAuthorizationResponse {
    #[serde(rename = "deviceCode")]
    device_code: String,
    #[serde(rename = "userCode")]
    user_code: String,
    #[serde(rename = "verificationUri")]
    verification_uri: String,
    #[serde(rename = "verificationUriComplete")]
    verification_uri_complete: Option<String>,
    #[serde(rename = "expiresIn")]
    expires_in: i64,
    interval: Option<i64>,
}

/// Token 响应
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct TokenResponse {
    #[serde(rename = "accessToken")]
    access_token: String,
    #[serde(rename = "refreshToken")]
    refresh_token: String,
    #[serde(rename = "expiresIn")]
    expires_in: i64,
}

/// 执行完整的 Builder ID 自动化登录
pub async fn perform_automated_builder_id_login(
    email: &str,
    email_password: &str,
    browser_mode: BrowserMode,
) -> Result<BuilderIdLoginResult> {
    println!("\n========== AWS Builder ID 自动化登录 ==========");
    println!("邮箱: {}", email);

    let region = "us-east-1";

    // 1. 注册 OIDC 客户端
    println!("\n[1/5] 注册 AWS SSO OIDC 客户端...");
    let client_reg = register_oidc_client(region).await?;
    println!("✓ 客户端注册成功");
    println!("  Client ID: {}...", &client_reg.client_id[..20]);

    // 2. 发起设备授权
    println!("\n[2/5] 发起设备授权...");
    let device_auth =
        start_device_authorization(region, &client_reg.client_id, &client_reg.client_secret)
            .await?;
    println!("✓ 设备授权已发起");
    println!("  User Code: {}", device_auth.user_code);
    println!("  Verification URL: {}", device_auth.verification_uri);

    // 3. 使用浏览器自动化完成授权
    println!("\n[3/5] 使用浏览器自动化完成授权...");
    let verification_url = device_auth
        .verification_uri_complete
        .as_ref()
        .unwrap_or(&device_auth.verification_uri);

    automate_browser_authorization(
        verification_url,
        &device_auth.user_code,
        email,
        email_password,
        browser_mode,
    )
    .await?;
    println!("✓ 浏览器授权完成");

    // 4. 轮询获取 Token
    println!("\n[4/5] 轮询获取 Token...");
    let token = poll_for_token(
        region,
        &client_reg.client_id,
        &client_reg.client_secret,
        &device_auth.device_code,
        device_auth.interval.unwrap_or(5) as u64,
    )
    .await?;
    println!("✓ Token 获取成功");

    // 5. 生成 clientIdHash 并保存授权文件
    println!("\n[5/5] 生成授权文件...");
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(client_reg.client_id.as_bytes());
    let client_id_hash = hex::encode(hasher.finalize());

    let params = IdcAuthParams {
        access_token: token.access_token.clone(),
        refresh_token: token.refresh_token.clone(),
        provider: "BuilderId".to_string(),
        client_id: client_reg.client_id.clone(),
        client_secret: client_reg.client_secret.clone(),
        client_id_hash: client_id_hash.clone(),
        region: region.to_string(),
    };

    generate_kiro_idc_auth(params).map_err(|e| anyhow!(e))?;
    println!("✓ 授权文件已保存");

    println!("\n========== 登录完成 ==========");

    Ok(BuilderIdLoginResult {
        access_token: token.access_token,
        refresh_token: token.refresh_token,
        client_id: client_reg.client_id,
        client_secret: client_reg.client_secret,
        client_id_hash,
        region: region.to_string(),
    })
}

/// 注册 OIDC 客户端
async fn register_oidc_client(region: &str) -> Result<ClientRegistration> {
    let url = format!("https://oidc.{}.amazonaws.com/client/register", region);

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

    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .context("Failed to register OIDC client")?;

    let status = response.status();
    let text = response.text().await?;

    if !status.is_success() {
        return Err(anyhow!("Client registration failed ({}): {}", status, text));
    }

    serde_json::from_str(&text).context("Failed to parse client registration")
}

/// 发起设备授权
async fn start_device_authorization(
    region: &str,
    client_id: &str,
    client_secret: &str,
) -> Result<DeviceAuthorizationResponse> {
    let url = format!("https://oidc.{}.amazonaws.com/device_authorization", region);

    let body = serde_json::json!({
        "clientId": client_id,
        "clientSecret": client_secret,
        "startUrl": "https://view.awsapps.com/start"
    });

    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .context("Failed to start device authorization")?;

    let status = response.status();
    let text = response.text().await?;

    if !status.is_success() {
        return Err(anyhow!(
            "Device authorization failed ({}): {}",
            status,
            text
        ));
    }

    serde_json::from_str(&text).context("Failed to parse device authorization")
}

/// 使用浏览器自动化完成授权
async fn automate_browser_authorization(
    verification_url: &str,
    _user_code: &str,
    _email: &str,
    _email_password: &str,
    browser_mode: BrowserMode,
) -> Result<()> {
    let (width, height) = BrowserAutomation::generate_random_window_size();
    let os_version = BrowserAutomation::generate_random_os_version();

    let config = BrowserConfig {
        mode: browser_mode,
        os: "Windows".to_string(),
        os_version,
        device_type: "PC".to_string(),
        language: "zh-CN".to_string(),
        window_width: width,
        window_height: height,
    };

    let automation = BrowserAutomation::new(config);
    let browser = automation.launch_browser()?;
    let tab = browser.new_tab().context("Failed to create new tab")?;

    // 应用指纹保护
    automation.apply_fingerprint_protection(&tab)?;

    // 导航到授权页面（带完整的 user_code）
    println!("  导航到授权页面...");
    tab.navigate_to(verification_url)
        .context("Failed to navigate to verification page")?;
    tab.wait_until_navigated()?;
    println!("  ✓ 页面加载完成");
    std::thread::sleep(Duration::from_secs(5));

    // 第一个授权页面：确认并继续
    println!("  等待第一个授权页面（确认并继续）...");
    std::thread::sleep(Duration::from_secs(5));

    // 尝试多种选择器策略
    let first_button_selectors = [
        "/html/body/div[3]/div[2]/main/div/div/div/div/div[2]/div/div/div/div/div/div[2]/form/div/div[4]/div/div[1]/button",
        "//button[@type='submit']",
        "//button[contains(text(), 'Confirm')]",
        "//button[contains(text(), 'Continue')]",
        "//button[contains(text(), '确认')]",
        "//form//button",
    ];

    let mut first_clicked = false;
    for (i, selector) in first_button_selectors.iter().enumerate() {
        println!("  尝试选择器 {}: {}", i + 1, selector);
        match automation.wait_for_element(&tab, selector, 5).await {
            Ok(true) => {
                println!("  ✓ 找到第一个授权按钮，点击中...");
                automation.click_element(&tab, selector)?;
                first_clicked = true;
                std::thread::sleep(Duration::from_secs(5));
                break;
            }
            Ok(false) => {
                // 元素未找到，继续尝试下一个选择器
                continue;
            }
            Err(e) => {
                println!("  选择器 {} 出错: {}", i + 1, e);
                continue;
            }
        }
    }

    if !first_clicked {
        println!("  ⚠ 未找到第一个授权按钮，可能已自动跳过或页面结构变化");
    }

    // 第二个授权页面：允许访问
    println!("  等待第二个授权页面（允许访问）...");
    std::thread::sleep(Duration::from_secs(5));

    // 尝试多种选择器策略
    let second_button_selectors = [
        "/html/body/div[3]/div[2]/main/div/div/div/div/div[3]/div/div/div[3]/div/div/div[2]/button",
        "//button[contains(text(), 'Allow')]",
        "//button[contains(text(), 'Authorize')]",
        "//button[contains(text(), '允许')]",
        "//button[@type='button' and contains(@class, 'primary')]",
        "//div[contains(@class, 'actions')]//button[last()]",
    ];

    let mut second_clicked = false;
    for (i, selector) in second_button_selectors.iter().enumerate() {
        println!("  尝试选择器 {}: {}", i + 1, selector);
        match automation.wait_for_element(&tab, selector, 5).await {
            Ok(true) => {
                println!("  ✓ 找到第二个授权按钮，点击中...");
                automation.click_element(&tab, selector)?;
                second_clicked = true;
                std::thread::sleep(Duration::from_secs(5));
                break;
            }
            Ok(false) => {
                // 元素未找到，继续尝试下一个选择器
                continue;
            }
            Err(e) => {
                println!("  选择器 {} 出错: {}", i + 1, e);
                continue;
            }
        }
    }

    if !second_clicked {
        println!("  ⚠ 未找到第二个授权按钮，可能已自动跳过或页面结构变化");
    }

    // 等待成功页面
    println!("  等待授权完成...");
    std::thread::sleep(Duration::from_secs(3));

    // 清理浏览器数据
    automation.clear_browser_data()?;

    Ok(())
}

/// 轮询获取 Token
async fn poll_for_token(
    region: &str,
    client_id: &str,
    client_secret: &str,
    device_code: &str,
    interval: u64,
) -> Result<TokenResponse> {
    let url = format!("https://oidc.{}.amazonaws.com/token", region);

    let body = serde_json::json!({
        "clientId": client_id,
        "clientSecret": client_secret,
        "grantType": "urn:ietf:params:oauth:grant-type:device_code",
        "deviceCode": device_code
    });

    let client = reqwest::Client::new();
    let max_attempts = 60; // 最多尝试 60 次（5 分钟）

    for attempt in 1..=max_attempts {
        tokio::time::sleep(Duration::from_secs(interval)).await;

        print!(".");
        std::io::Write::flush(&mut std::io::stdout()).ok();

        let response = client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .context("Failed to poll for token")?;

        let status = response.status();
        let text = response.text().await?;

        if status.is_success() {
            println!(); // 换行
            return serde_json::from_str(&text).context("Failed to parse token");
        }

        // 解析错误
        if let Ok(error) = serde_json::from_str::<serde_json::Value>(&text) {
            if let Some(error_type) = error.get("error").and_then(|e| e.as_str()) {
                match error_type {
                    "authorization_pending" => {
                        // 继续等待
                        continue;
                    }
                    "slow_down" => {
                        // 增加等待时间
                        tokio::time::sleep(Duration::from_secs(5)).await;
                        continue;
                    }
                    "expired_token" => {
                        return Err(anyhow!("Device code expired"));
                    }
                    "access_denied" => {
                        return Err(anyhow!("User denied authorization"));
                    }
                    _ => {
                        return Err(anyhow!("Authorization error: {}", error_type));
                    }
                }
            }
        }

        if attempt == max_attempts {
            return Err(anyhow!("Token polling timeout"));
        }
    }

    Err(anyhow!("Token polling failed"))
}
