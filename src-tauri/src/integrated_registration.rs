// 集成注册和 OAuth 授权流程
// 注册成功后直接在同一个浏览器会话中完成 Builder ID OAuth 授权

use crate::browser_automation::BrowserAutomation;
use crate::database::{update_account, DbState};
use crate::graph_api::GraphApiClient;
use crate::imap_client::ImapClient;
use crate::kiro_auth::{generate_kiro_idc_auth, IdcAuthParams};
use crate::models::{AccountUpdate, BrowserConfig, BrowserMode, EmailMode, OAuthInfo, OAuthStatus};
use anyhow::{anyhow, Context, Result};
use headless_chrome::{Browser, Tab};
use serde::Serialize;
use std::sync::Arc;
use std::time::Duration;
use tauri::Manager;

/// 集成注册结果
#[derive(Debug, Clone, Serialize)]
pub struct IntegratedRegistrationResult {
    pub kiro_password: String,
    pub oauth_completed: bool,
    pub oauth_message: Option<String>,
    pub client_id_hash: Option<String>,
}

/// 执行集成的注册和 OAuth 授权流程
pub struct RegistrationParams<'a> {
    pub email: &'a str,
    pub email_password: &'a str,
    pub client_id: &'a str,
    pub refresh_token: &'a str,
    pub name: &'a str,
    pub browser_mode: BrowserMode,
    pub email_mode: EmailMode,
    pub account_id: i64,
    pub app_handle: tauri::AppHandle,
}

#[allow(clippy::too_many_arguments)]
pub async fn perform_integrated_registration_and_oauth(
    email: &str,
    email_password: &str,
    client_id: &str,
    refresh_token: &str,
    name: &str,
    browser_mode: BrowserMode,
    email_mode: EmailMode,
    account_id: i64,
    app_handle: tauri::AppHandle,
) -> Result<IntegratedRegistrationResult> {
    let params = RegistrationParams {
        email,
        email_password,
        client_id,
        refresh_token,
        name,
        browser_mode,
        email_mode,
        account_id,
        app_handle,
    };
    perform_integrated_registration_and_oauth_impl(params).await
}

async fn perform_integrated_registration_and_oauth_impl(
    params: RegistrationParams<'_>,
) -> Result<IntegratedRegistrationResult> {
    println!("\n========== 集成注册和 OAuth 授权 ==========");
    println!("邮箱: {}", params.email);

    // 第一步：执行注册流程（不关闭浏览器）
    println!("\n[阶段 1/2] 执行 Kiro 注册...");
    let (kiro_password, browser, tab) = perform_registration_keep_browser(
        params.email,
        params.email_password,
        params.client_id,
        params.refresh_token,
        params.name,
        params.browser_mode.clone(),
        params.email_mode,
    )
    .await?;

    println!("✓ 注册成功！密码: {}", kiro_password);
    println!("✓ 浏览器会话保持打开，继续 OAuth 授权...");

    // 第二步：在同一个浏览器会话中执行 Builder ID OAuth 授权
    println!("\n[阶段 2/2] 执行 Builder ID OAuth 授权...");

    let oauth_result = perform_builder_id_oauth_with_existing_browser(
        params.email,
        params.email_password,
        browser,
        tab,
        params.browser_mode,
        params.account_id,
        params.app_handle,
    )
    .await;

    match oauth_result {
        Ok(client_id_hash) => {
            println!("✓ OAuth 授权成功！");
            println!("✓ 授权文件已保存");

            Ok(IntegratedRegistrationResult {
                kiro_password,
                oauth_completed: true,
                oauth_message: Some("OAuth 授权成功！授权文件已保存".to_string()),
                client_id_hash: Some(client_id_hash),
            })
        }
        Err(e) => {
            println!("⚠ OAuth 授权失败: {}", e);
            println!("✓ 注册已完成，可以稍后手动点击授权按钮");

            Ok(IntegratedRegistrationResult {
                kiro_password,
                oauth_completed: false,
                oauth_message: Some(format!("OAuth 授权失败: {}。可以稍后手动点击授权按钮", e)),
                client_id_hash: None,
            })
        }
    }
}

/// 执行注册流程但保持浏览器打开
async fn perform_registration_keep_browser(
    email: &str,
    _email_password: &str,
    client_id: &str,
    refresh_token: &str,
    name: &str,
    browser_mode: BrowserMode,
    email_mode: EmailMode,
) -> Result<(String, Browser, Arc<Tab>)> {
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

    // 导航到登录页面
    tab.navigate_to("https://app.kiro.dev/signin")
        .context("Failed to navigate to signin page")?;
    tab.wait_until_navigated()?;
    std::thread::sleep(Duration::from_secs(3));

    // 点击 Build ID 登录按钮
    // let google_button_xpath =
    //     "/html/body/div[2]/div/div[1]/main/div/div/div/div/div/div/div/div[1]/button[3]";
    let builder_id_button_xpath = "//button[contains(., 'Builder ID') and contains(., 'Sign in')]";

    if automation
        .wait_for_element(&tab, builder_id_button_xpath, 60)
        .await?
    {
        automation.click_element(&tab, builder_id_button_xpath)?;
        std::thread::sleep(Duration::from_secs(5));
    } else {
        return Err(anyhow!("Builer ID sign-in button not found"));
    }

    // 输入邮箱
    // let email_page_xpath =
    //     "/html/body/div[2]/div[2]/div[1]/div/div/div/form/div/div/div/div/div/div/div/div/div[2]/div/div[2]/div/div/div/div/div/input";
    // if automation
    //     .wait_for_element(&tab, email_page_xpath, 60)
    //     .await?
    // {
    //     std::thread::sleep(Duration::from_millis(500));
    //     let email_input_xpath = "/html/body/div[2]/div[2]/div[1]/div/div/div/form/div/div/div/div/div/div/div/div/div[2]/div/div[2]/div/div/div/div/div/input";
    //     automation.input_text(&tab, email_input_xpath, email)?;
    //     std::thread::sleep(Duration::from_millis(2000));
    //     let continue_button_xpath = "/html/body/div[2]/div[2]/div[1]/div/div/div/form/div/div/div/div/div/div/div/div/div[3]/button";
    //     automation.click_element(&tab, continue_button_xpath)?;
    //     std::thread::sleep(Duration::from_secs(4));
    // } else {
    //     return Err(anyhow!("Email input page not found"));
    // }

        // 修复后的 Email XPath
    let email_input_xpath = "//div[@data-testid='test-input']/input";
    
    if automation
        .wait_for_element(&tab, email_input_xpath, 10)
        .await?
    {
        std::thread::sleep(Duration::from_millis(500));
        automation.input_text(&tab, email_input_xpath, email)?;
        std::thread::sleep(Duration::from_millis(2000));
        
        // 修复后的 Button XPath
        let continue_button_xpath = "//button[@data-testid='test-primary-button']";
        
        automation.click_element(&tab, continue_button_xpath)?;
        std::thread::sleep(Duration::from_secs(4));
    } else {
        return Err(anyhow!("Email input page not found"));
    }

    // 输入姓名
    //signup-full-name-input
    let name_input_xpath = "//div[@data-testid='signup-full-name-input']/input";

    // let name_page_xpath = "/html/body/div[2]/div/div/div[2]/div/div/div/div[2]/div";
    if automation
        .wait_for_element(&tab, name_input_xpath, 60)
        .await?
    {
        std::thread::sleep(Duration::from_millis(500));
        automation.input_text(&tab, name_input_xpath, name)?;
        std::thread::sleep(Duration::from_millis(2000));
        let continue_button_xpath = "//*[@data-testid='signup-next-button']";
        automation.click_element(&tab, continue_button_xpath)?;
        std::thread::sleep(Duration::from_secs(4));
    } else {
        return Err(anyhow!("Name input page not found"));
    }

    // 获取验证码
    
    let code_input_xpath = "//div[@data-testid='email-verification-form-code-input']/input";
    if automation
        .wait_for_element(&tab, code_input_xpath, 60)
        .await?
    {
        std::thread::sleep(Duration::from_millis(500));

        let verification_code = match email_mode {
            EmailMode::GraphApi => {
                let graph_client = GraphApiClient::new();
                graph_client
                    .wait_for_verification_code(client_id, refresh_token, email, 120)
                    .await?
            }
            EmailMode::Imap => {
                let imap_client = ImapClient::new();
                imap_client
                    .wait_for_verification_code(client_id, refresh_token, email, 120)
                    .await?
            }
        };
        automation.input_text(&tab, code_input_xpath, &verification_code)?;
        std::thread::sleep(Duration::from_millis(2000));
        let continue_button_xpath = "//button[@data-testid='email-verification-verify-button']";

        automation.click_element(&tab, continue_button_xpath)?;
        std::thread::sleep(Duration::from_secs(4));
    } else {
        return Err(anyhow!("Verification code page not found"));
    }

    // 设置密码

    let password_xpath = "//div[@data-testid='test-input']/input";
    let confirm_password_xpath = "//div[@data-testid='test-retype-input']/input";
    if automation
        .wait_for_element(&tab, password_xpath, 60)
        .await?
    {
        std::thread::sleep(Duration::from_millis(500));

        let password = generate_secure_password();

        automation.input_text(&tab, password_xpath, &password)?;
        std::thread::sleep(Duration::from_millis(2000));
        automation.input_text(&tab, confirm_password_xpath, &password)?;
        std::thread::sleep(Duration::from_millis(2000));
        let continue_button_xpath = "//button[@data-testid='test-primary-button']";
        automation.click_element(&tab, continue_button_xpath)?;
        std::thread::sleep(Duration::from_secs(4));

        let success_page_xpath =
            "/html/body/div[2]/div/div[1]/main/div/div[1]/div/div/div/div/div[2]";
        if automation
            .wait_for_element(&tab, success_page_xpath, 60)
            .await?
        {
            // 注册成功，返回密码和浏览器实例（不关闭）
            Ok((password, browser, tab))
        } else {
            Err(anyhow!("Success page not found"))
        }
    } else {
        Err(anyhow!("Password input page not found"))
    }
}

/// 在现有浏览器会话中执行 Builder ID OAuth 授权
async fn perform_builder_id_oauth_with_existing_browser(
    _email: &str,
    _email_password: &str,
    browser: Browser,
    _existing_tab: Arc<Tab>,
    _browser_mode: BrowserMode, // 忽略用户设置，强制使用有头模式
    account_id: i64,
    app_handle: tauri::AppHandle,
) -> Result<String> {
    println!("  使用现有浏览器会话...");

    let region = "us-east-1";

    // 更新 OAuth 状态为进行中
    if let Some(db_state) = app_handle.try_state::<DbState>() {
        let conn = db_state.0.lock().unwrap();
        let _ = update_account(
            &conn,
            AccountUpdate {
                id: account_id,
                email: None,
                email_password: None,
                client_id: None,
                refresh_token: None,
                kiro_password: None,
                status: None,
                error_reason: None,
                oauth_status: Some(OAuthStatus::InProgress),
                oauth_info: None,
            },
        );
    }

    // 1. 注册 OIDC 客户端
    println!("  [1/4] 注册 OIDC 客户端...");
    let client_reg = register_oidc_client(region).await?;

    // 2. 发起设备授权
    println!("  [2/4] 发起设备授权...");
    let device_auth =
        start_device_authorization(region, &client_reg.client_id, &client_reg.client_secret)
            .await?;

    // 3. 在新 tab 中打开授权页面（授权页面会在新 tab 中打开）
    println!("  [3/4] 打开授权页面...");
    let new_tab = browser.new_tab().context("Failed to create new tab")?;

    // 创建 BrowserAutomation 实例
    // 注意：OAuth 授权必须使用有头模式，因为 React SPA 在无头模式下无法正确渲染
    let (width, height) = BrowserAutomation::generate_random_window_size();
    let os_version = BrowserAutomation::generate_random_os_version();
    let config = BrowserConfig {
        mode: BrowserMode::Foreground, // 强制使用有头模式
        os: "Windows".to_string(),
        os_version,
        device_type: "PC".to_string(),
        language: "zh-CN".to_string(),
        window_width: width,
        window_height: height,
    };
    let automation = BrowserAutomation::new(config);

    let verification_url = device_auth
        .verification_uri_complete
        .as_ref()
        .unwrap_or(&device_auth.verification_uri);

    println!("  导航到: {}", verification_url);
    new_tab.navigate_to(verification_url)?;
    new_tab.wait_until_navigated()?;
    println!("  ✓ HTML 加载完成");
    std::thread::sleep(Duration::from_secs(5));

    // 第一个授权页面：确认并继续
    println!("  等待第一个授权页面（确认并继续）...");

    let first_button_xpath = "//*[@id='cli_verification_btn']";

    // 使用 automation.wait_for_element（和注册流程一样）
    if automation
        .wait_for_element(&new_tab, first_button_xpath, 60)
        .await?
    {
        println!("  ✓ 找到第一个授权按钮");
        std::thread::sleep(Duration::from_millis(500));
        automation.click_element(&new_tab, first_button_xpath)?;
        println!("  ✓ 已点击第一个按钮");
        std::thread::sleep(Duration::from_secs(5));
    } else {
        println!("  ⚠ 未找到第一个授权按钮");
    }

    // 第二个授权页面：允许访问
    println!("  等待第二个授权页面（允许访问）...");
    std::thread::sleep(Duration::from_secs(5));

    let second_button_xpath = "//*[@id=':rh:']/div[3]/div/div/div[2]/button";

    // 使用 automation.wait_for_element（和注册流程一样）
    if automation
        .wait_for_element(&new_tab, second_button_xpath, 60)
        .await?
    {
        println!("  ✓ 找到第二个授权按钮");
        std::thread::sleep(Duration::from_millis(500));
        automation.click_element(&new_tab, second_button_xpath)?;
        println!("  ✓ 已点击第二个按钮");
        std::thread::sleep(Duration::from_secs(5));
    } else {
        println!("  ⚠ 未找到第二个授权按钮");
    }

    println!("  等待授权完成...");
    std::thread::sleep(Duration::from_secs(3));

    // 4. 轮询获取 Token
    println!("  [4/4] 轮询获取 Token...");
    let token = poll_for_token(
        region,
        &client_reg.client_id,
        &client_reg.client_secret,
        &device_auth.device_code,
        device_auth.interval.unwrap_or(5) as u64,
    )
    .await?;

    // 5. 生成授权文件
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

    // 6. 保存 OAuth 信息到数据库
    let oauth_info = OAuthInfo {
        access_token: token.access_token,
        refresh_token: token.refresh_token,
        provider: "BuilderId".to_string(),
        auth_method: "IdC".to_string(),
        expires_at: (chrono::Utc::now() + chrono::Duration::hours(1)).to_rfc3339(),
        profile_arn: None,
        client_id_hash: Some(client_id_hash.clone()),
        region: Some(region.to_string()),
        authorized_at: chrono::Utc::now().to_rfc3339(),
        // 客户端注册信息
        client_id: Some(client_reg.client_id),
        client_secret: Some(client_reg.client_secret),
        client_expires_at: Some((chrono::Utc::now() + chrono::Duration::days(90)).to_rfc3339()),
    };

    let oauth_info_json = serde_json::to_string(&oauth_info)
        .map_err(|e| anyhow!("Failed to serialize OAuth info: {}", e))?;

    // 更新数据库中的 OAuth 状态和信息
    if let Some(db_state) = app_handle.try_state::<DbState>() {
        let conn = db_state.0.lock().unwrap();
        update_account(
            &conn,
            AccountUpdate {
                id: account_id,
                email: None,
                email_password: None,
                client_id: None,
                refresh_token: None,
                kiro_password: None,
                status: None,
                error_reason: None,
                oauth_status: Some(OAuthStatus::Authorized),
                oauth_info: Some(oauth_info_json),
            },
        )
        .map_err(|e| anyhow!("Failed to update OAuth status: {}", e))?;
    }

    // 清理浏览器
    drop(browser);

    Ok(client_id_hash)
}

// 辅助函数（从 builder_id_automation.rs 复制）

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct ClientRegistration {
    #[serde(rename = "clientId")]
    client_id: String,
    #[serde(rename = "clientSecret")]
    client_secret: String,
}

#[derive(Debug, Deserialize)]
struct DeviceAuthorizationResponse {
    #[serde(rename = "deviceCode")]
    device_code: String,
    #[serde(rename = "userCode")]
    #[allow(dead_code)]
    user_code: String,
    #[serde(rename = "verificationUri")]
    verification_uri: String,
    #[serde(rename = "verificationUriComplete")]
    verification_uri_complete: Option<String>,
    interval: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    #[serde(rename = "accessToken")]
    access_token: String,
    #[serde(rename = "refreshToken")]
    refresh_token: String,
}

async fn register_oidc_client(region: &str) -> Result<ClientRegistration> {
    let url = format!("https://oidc.{}.amazonaws.com/client/register", region);
    let body = serde_json::json!({
        "clientName": "Kiro IDE Auto Registration",
        "clientType": "public",
        "scopes": ["codewhisperer:completions", "codewhisperer:analysis", "codewhisperer:conversations"],
        "grantTypes": ["urn:ietf:params:oauth:grant-type:device_code", "refresh_token"],
        "issuerUrl": "https://oidc.us-east-1.amazonaws.com"
    });

    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await?;
    let text = response.text().await?;
    Ok(serde_json::from_str(&text)?)
}

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
        .await?;
    let text = response.text().await?;
    Ok(serde_json::from_str(&text)?)
}

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
    for _ in 0..60 {
        tokio::time::sleep(Duration::from_secs(interval)).await;
        print!(".");
        std::io::Write::flush(&mut std::io::stdout()).ok();

        let response = client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;
        if response.status().is_success() {
            let text = response.text().await?;
            println!();
            return Ok(serde_json::from_str(&text)?);
        }
    }
    Err(anyhow!("Token polling timeout"))
}

fn generate_secure_password() -> String {
    use rand::seq::SliceRandom;
    use rand::Rng;
    let mut rng = rand::thread_rng();
    const UPPERCASE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    const LOWERCASE: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
    const DIGITS: &[u8] = b"0123456789";
    const SPECIAL: &[u8] = b"!@#$%^&*";
    let mut password = vec![
        UPPERCASE[rng.gen_range(0..UPPERCASE.len())] as char,
        UPPERCASE[rng.gen_range(0..UPPERCASE.len())] as char,
        LOWERCASE[rng.gen_range(0..LOWERCASE.len())] as char,
        LOWERCASE[rng.gen_range(0..LOWERCASE.len())] as char,
        DIGITS[rng.gen_range(0..DIGITS.len())] as char,
        DIGITS[rng.gen_range(0..DIGITS.len())] as char,
        SPECIAL[rng.gen_range(0..SPECIAL.len())] as char,
        SPECIAL[rng.gen_range(0..SPECIAL.len())] as char,
    ];
    const ALL_CHARS: &[u8] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*";
    for _ in 0..(16 - password.len()) {
        password.push(ALL_CHARS[rng.gen_range(0..ALL_CHARS.len())] as char);
    }
    password.shuffle(&mut rng);
    password.into_iter().collect()
}
