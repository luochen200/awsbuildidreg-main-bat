use crate::browser_automation::BrowserAutomation;
use crate::builder_id_automation;
use crate::database::{self, DbState};
use crate::graph_api::GraphApiClient;
use crate::imap_client::ImapClient;
use crate::integrated_registration;
use crate::kiro_auth::{self, IdcAuthParams, SocialAuthParams};
use crate::kiro_oauth;
use crate::models::*;
use anyhow::{anyhow, Context, Result};
use tauri::State;

#[tauri::command]
pub async fn get_accounts(
    db: State<'_, DbState>,
    status_filter: Option<String>,
) -> Result<Vec<Account>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let accounts = if let Some(status) = status_filter {
        database::get_accounts_by_status(&conn, &status)
    } else {
        database::get_all_accounts(&conn)
    };

    accounts.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn add_account(db: State<'_, DbState>, account: NewAccount) -> Result<i64, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    database::insert_account(&conn, account).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_account(db: State<'_, DbState>, update: AccountUpdate) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    database::update_account(&conn, update).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_account(db: State<'_, DbState>, id: i64) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    database::delete_account(&conn, id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_all_accounts(db: State<'_, DbState>) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    database::delete_all_accounts(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn import_accounts(
    db: State<'_, DbState>,
    content: String,
) -> Result<ImportResult, String> {
    let mut success_count = 0;
    let mut error_count = 0;
    let mut errors = Vec::new();

    let lines: Vec<&str> = content.lines().collect();

    for (index, line) in lines.iter().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split("----").collect();

        if parts.len() != 4 {
            error_count += 1;
            errors.push(ImportError {
                line_number: index + 1,
                content: line.to_string(),
                reason: format!(
                    "Invalid format: expected 4 fields separated by '----', got {}",
                    parts.len()
                ),
            });
            continue;
        }

        let email = parts[0].trim();
        let password = parts[1].trim();
        let client_id = parts[2].trim();
        let refresh_token = parts[3].trim();

        // Validate email format
        if !email.contains('@') {
            error_count += 1;
            errors.push(ImportError {
                line_number: index + 1,
                content: line.to_string(),
                reason: "Invalid email address".to_string(),
            });
            continue;
        }

        // Validate that fields are not empty
        if email.is_empty()
            || password.is_empty()
            || client_id.is_empty()
            || refresh_token.is_empty()
        {
            error_count += 1;
            errors.push(ImportError {
                line_number: index + 1,
                content: line.to_string(),
                reason: "One or more fields are empty".to_string(),
            });
            continue;
        }

        let new_account = NewAccount {
            email: email.to_string(),
            email_password: password.to_string(),
            client_id: client_id.to_string(),
            refresh_token: refresh_token.to_string(),
        };

        let conn = db.0.lock().map_err(|e| e.to_string())?;
        match database::insert_account(&conn, new_account) {
            Ok(_) => success_count += 1,
            Err(e) => {
                error_count += 1;
                errors.push(ImportError {
                    line_number: index + 1,
                    content: line.to_string(),
                    reason: format!("Database error: {}", e),
                });
            }
        }
    }

    Ok(ImportResult {
        success_count,
        error_count,
        errors,
    })
}

#[tauri::command]
pub async fn get_settings(db: State<'_, DbState>) -> Result<Settings, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    database::get_settings(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_settings(db: State<'_, DbState>, settings: Settings) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    database::update_settings(&conn, settings).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn start_registration(db: State<'_, DbState>, account_id: i64) -> Result<String, String> {
    // Get account details
    let account = {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        database::get_account_by_id(&conn, account_id).map_err(|e| e.to_string())?
    };

    // Update status to in_progress
    {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        database::update_account(
            &conn,
            AccountUpdate {
                id: account_id,
                email: None,
                email_password: None,
                client_id: None,
                refresh_token: None,
                kiro_password: None,
                status: Some(AccountStatus::InProgress),
                error_reason: None,
                oauth_status: None,
                oauth_info: None,
            },
        )
        .map_err(|e| e.to_string())?;
    }

    // Get browser settings
    let settings = {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        database::get_settings(&conn).map_err(|e| e.to_string())?
    };

    // Generate random name for registration
    let names = vec![
        "Zhang Wei",
        "Wang Fang",
        "Li Na",
        "Liu Yang",
        "Chen Jing",
        "Zhang Min",
        "Wang Lei",
        "Li Qiang",
        "Liu Min",
        "Chen Wei",
    ];
    let random_name = names[rand::random::<usize>() % names.len()];

    // Start registration process
    let result = perform_registration(
        &account.email,
        &account.email_password,
        &account.client_id,
        &account.refresh_token,
        random_name,
        settings.browser_mode,
        settings.email_mode,
    )
    .await;

    match result {
        Ok(kiro_password) => {
            // Update account with success
            let conn = db.0.lock().map_err(|e| e.to_string())?;
            database::update_account(
                &conn,
                AccountUpdate {
                    id: account_id,
                    email: None,
                    email_password: None,
                    client_id: None,
                    refresh_token: None,
                    kiro_password: Some(kiro_password.clone()),
                    status: Some(AccountStatus::Registered),
                    error_reason: None,
                    oauth_status: None,
                    oauth_info: None,
                },
            )
            .map_err(|e| e.to_string())?;

            Ok(format!(
                "Registration completed successfully. Password: {}",
                kiro_password
            ))
        }
        Err(e) => {
            // Update account with error
            let conn = db.0.lock().map_err(|e| e.to_string())?;
            database::update_account(
                &conn,
                AccountUpdate {
                    id: account_id,
                    email: None,
                    email_password: None,
                    client_id: None,
                    refresh_token: None,
                    kiro_password: None,
                    status: Some(AccountStatus::Error),
                    error_reason: Some(e.to_string()),
                    oauth_status: None,
                    oauth_info: None,
                },
            )
            .map_err(|e| e.to_string())?;

            Err(e.to_string())
        }
    }
}

async fn perform_registration(
    email: &str,
    _email_password: &str,
    client_id: &str,
    refresh_token: &str,
    name: &str,
    browser_mode: BrowserMode,
    email_mode: EmailMode,
) -> Result<String> {
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

    // Apply fingerprint protection
    automation.apply_fingerprint_protection(&tab)?;

    // Navigate to signin page
    tab.navigate_to("https://app.kiro.dev/signin")
        .context("Failed to navigate to signin page")?;
    tab.wait_until_navigated()?;

    std::thread::sleep(std::time::Duration::from_secs(3));

    // Click the third button (Google sign in button)
    let google_button_xpath =
        "/html/body/div[2]/div/div[1]/main/div/div/div/div/div/div/div/div[1]/button[3]";

    if automation
        .wait_for_element(&tab, google_button_xpath, 60)
        .await?
    {
        automation.click_element(&tab, google_button_xpath)?;
        std::thread::sleep(std::time::Duration::from_secs(5));
    } else {
        return Err(anyhow!("Google sign-in button not found"));
    }

    // Wait for email input page
    let email_page_xpath =
        "/html/body/div[2]/div[2]/div[1]/div/div/div/form/div/div/div/div/div/div/div/div";

    if automation
        .wait_for_element(&tab, email_page_xpath, 60)
        .await?
    {
        std::thread::sleep(std::time::Duration::from_millis(500));

        // Input email
        let email_input_xpath = "/html/body/div[2]/div[2]/div[1]/div/div/div/form/div/div/div/div/div/div/div/div/div[2]/div/div[2]/div/div/div/div/div/input";
        automation.input_text(&tab, email_input_xpath, email)?;
        std::thread::sleep(std::time::Duration::from_millis(2000));

        // Click continue button
        let continue_button_xpath = "/html/body/div[2]/div[2]/div[1]/div/div/div/form/div/div/div/div/div/div/div/div/div[3]/button";
        automation.click_element(&tab, continue_button_xpath)?;
        std::thread::sleep(std::time::Duration::from_secs(4));
    } else {
        return Err(anyhow!("Email input page not found"));
    }

    // Wait for name input page
    let name_page_xpath = "/html/body/div[2]/div/div/div[2]/div/div/div/div[2]/div";

    if automation
        .wait_for_element(&tab, name_page_xpath, 60)
        .await?
    {
        std::thread::sleep(std::time::Duration::from_millis(500));

        // Input name
        let name_input_xpath = "/html/body/div[2]/div/div/div[1]/div/div/form/fieldset/div/div/div/div/div/div/div/div/div[3]/div/div[2]/div/div/div/div/div/input";
        automation.input_text(&tab, name_input_xpath, name)?;
        std::thread::sleep(std::time::Duration::from_millis(2000));

        // Click continue button
        let continue_button_xpath = "/html/body/div[2]/div/div/div[1]/div/div/form/fieldset/div/div/div/div/div/div/div/div/div[4]/button";
        automation.click_element(&tab, continue_button_xpath)?;
        std::thread::sleep(std::time::Duration::from_secs(4));
    } else {
        return Err(anyhow!("Name input page not found"));
    }

    // Wait for verification code page
    let verification_page_xpath =
        "/html/body/div[2]/div/div/div[1]/div/div/div[2]/form/fieldset/div/div/div/div/div/div";

    if automation
        .wait_for_element(&tab, verification_page_xpath, 60)
        .await?
    {
        std::thread::sleep(std::time::Duration::from_millis(500));

        // Fetch verification code based on email mode
        let verification_code = match email_mode {
            EmailMode::GraphApi => {
                let graph_client = GraphApiClient::new();
                match graph_client
                    .wait_for_verification_code(client_id, refresh_token, email, 120)
                    .await
                {
                    Ok(code) => code,
                    Err(_) => {
                        // If no code received after 120 seconds, click resend button
                        let resend_button_xpath = "/html/body/div[2]/div/div/div[1]/div/div/div[2]/form/fieldset/div/div/div/div/div/div/div[3]/div/div[2]/div/div/div/div/div/div[1]/div/div[2]/button";
                        automation.click_element(&tab, resend_button_xpath)?;
                        std::thread::sleep(std::time::Duration::from_secs(5));

                        // Wait again for verification code
                        graph_client
                            .wait_for_verification_code(client_id, refresh_token, email, 120)
                            .await?
                    }
                }
            }
            EmailMode::Imap => {
                let imap_client = ImapClient::new();
                match imap_client
                    .wait_for_verification_code(client_id, refresh_token, email, 120)
                    .await
                {
                    Ok(code) => code,
                    Err(_) => {
                        // If no code received after 120 seconds, click resend button
                        let resend_button_xpath = "/html/body/div[2]/div/div/div[1]/div/div/div[2]/form/fieldset/div/div/div/div/div/div/div[3]/div/div[2]/div/div/div/div/div/div[1]/div/div[2]/button";
                        automation.click_element(&tab, resend_button_xpath)?;
                        std::thread::sleep(std::time::Duration::from_secs(5));

                        // Wait again for verification code
                        imap_client
                            .wait_for_verification_code(client_id, refresh_token, email, 120)
                            .await?
                    }
                }
            }
        };

        // Input verification code
        let code_input_xpath = "/html/body/div[2]/div/div/div[1]/div/div/div[2]/form/fieldset/div/div/div/div/div/div/div[3]/div/div[2]/div/div/div/div/div/div[1]/div/div[1]/div/input";
        automation.input_text(&tab, code_input_xpath, &verification_code)?;
        std::thread::sleep(std::time::Duration::from_millis(2000));

        // Click continue button
        let continue_button_xpath = "/html/body/div[2]/div/div/div[1]/div/div/div[2]/form/fieldset/div/div/div/div/div/div/div[4]/button";
        automation.click_element(&tab, continue_button_xpath)?;
        std::thread::sleep(std::time::Duration::from_secs(4));
    } else {
        return Err(anyhow!("Verification code page not found"));
    }

    // Wait for password input page
    let password_page_xpath = "/html/body/div[2]/div[2]/div[1]/div/div/div/form/div/div/div/div/div/div/div/div[2]/div[3]/button";

    if automation
        .wait_for_element(&tab, password_page_xpath, 60)
        .await?
    {
        std::thread::sleep(std::time::Duration::from_millis(500));

        // Generate a secure random password
        let password = generate_secure_password();

        // Input password
        let password_input_xpath = "/html/body/div[2]/div[2]/div[1]/div/div/div/form/div/div/div/div/div/div/div/div[1]/div[3]/div/div[2]/div/div/div/div/div/span/span/div/input";
        automation.input_text(&tab, password_input_xpath, &password)?;
        std::thread::sleep(std::time::Duration::from_millis(2000));

        // Input confirm password
        let confirm_password_xpath = "/html/body/div[2]/div[2]/div[1]/div/div/div/form/div/div/div/div/div/div/div/div[1]/div[4]/div/div[2]/div/div/div/div/div/input";
        automation.input_text(&tab, confirm_password_xpath, &password)?;
        std::thread::sleep(std::time::Duration::from_millis(2000));

        // Click continue button
        let continue_button_xpath = "/html/body/div[2]/div[2]/div[1]/div/div/div/form/div/div/div/div/div/div/div/div[2]/div[3]/button";
        automation.click_element(&tab, continue_button_xpath)?;
        std::thread::sleep(std::time::Duration::from_secs(4));

        // Wait for success page
        let success_page_xpath =
            "/html/body/div[2]/div/div[1]/main/div/div[1]/div/div/div/div/div[2]";

        if automation
            .wait_for_element(&tab, success_page_xpath, 60)
            .await?
        {
            // Registration successful
            automation.clear_browser_data()?;
            Ok(password)
        } else {
            Err(anyhow!(
                "Success page not found - registration may have failed"
            ))
        }
    } else {
        Err(anyhow!("Password input page not found"))
    }
}

fn generate_secure_password() -> String {
    use rand::seq::SliceRandom;
    use rand::Rng;

    let mut rng = rand::thread_rng();

    // Define character sets
    const UPPERCASE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    const LOWERCASE: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
    const DIGITS: &[u8] = b"0123456789";
    const SPECIAL: &[u8] = b"!@#$%^&*";

    // Ensure at least one character from each required category
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

    // Fill the rest randomly (total 16 characters)
    const ALL_CHARS: &[u8] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*";
    for _ in 0..(16 - password.len()) {
        password.push(ALL_CHARS[rng.gen_range(0..ALL_CHARS.len())] as char);
    }

    // Shuffle to avoid predictable pattern
    password.shuffle(&mut rng);

    password.into_iter().collect()
}

#[tauri::command]
pub async fn start_batch_registration(db: State<'_, DbState>) -> Result<String, String> {
    // Get all accounts with status 'not_registered'
    let accounts = {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        database::get_accounts_by_status(&conn, "not_registered").map_err(|e| e.to_string())?
    };

    if accounts.is_empty() {
        return Ok("没有需要注册的账号".to_string());
    }

    let total_count = accounts.len();
    let mut success_count = 0;
    let mut error_count = 0;

    // Get browser settings once
    let settings = {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        database::get_settings(&conn).map_err(|e| e.to_string())?
    };

    // Process each account sequentially
    for account in accounts {
        // Update status to in_progress
        {
            let conn = db.0.lock().map_err(|e| e.to_string())?;
            database::update_account(
                &conn,
                AccountUpdate {
                    id: account.id,
                    email: None,
                    email_password: None,
                    client_id: None,
                    refresh_token: None,
                    kiro_password: None,
                    status: Some(AccountStatus::InProgress),
                    error_reason: None,
                    oauth_status: None,
                    oauth_info: None,
                },
            )
            .map_err(|e| e.to_string())?;
        }

        // Generate random name for registration
        let names = vec![
            "Zhang Wei",
            "Wang Fang",
            "Li Na",
            "Liu Yang",
            "Chen Jing",
            "Zhang Min",
            "Wang Lei",
            "Li Qiang",
            "Liu Min",
            "Chen Wei",
        ];
        let random_name = names[rand::random::<usize>() % names.len()];

        // Start registration process
        let result = perform_registration(
            &account.email,
            &account.email_password,
            &account.client_id,
            &account.refresh_token,
            random_name,
            settings.browser_mode.clone(),
            settings.email_mode.clone(),
        )
        .await;

        match result {
            Ok(kiro_password) => {
                // Update account with success
                let conn = db.0.lock().map_err(|e| e.to_string())?;
                database::update_account(
                    &conn,
                    AccountUpdate {
                        id: account.id,
                        email: None,
                        email_password: None,
                        client_id: None,
                        refresh_token: None,
                        kiro_password: Some(kiro_password),
                        status: Some(AccountStatus::Registered),
                        error_reason: None,
                        oauth_status: None,
                        oauth_info: None,
                    },
                )
                .map_err(|e| e.to_string())?;

                success_count += 1;
            }
            Err(e) => {
                // Update account with error
                let conn = db.0.lock().map_err(|e| e.to_string())?;
                database::update_account(
                    &conn,
                    AccountUpdate {
                        id: account.id,
                        email: None,
                        email_password: None,
                        client_id: None,
                        refresh_token: None,
                        kiro_password: None,
                        status: Some(AccountStatus::Error),
                        error_reason: Some(e.to_string()),
                        oauth_status: None,
                        oauth_info: None,
                    },
                )
                .map_err(|e| e.to_string())?;

                error_count += 1;
            }
        }
    }

    Ok(format!(
        "批量注册完成！总计: {}, 成功: {}, 失败: {}",
        total_count, success_count, error_count
    ))
}

#[tauri::command]
pub async fn export_accounts(
    db: State<'_, DbState>,
    status_filter: Option<String>,
) -> Result<String, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let accounts = if let Some(status) = status_filter {
        database::get_accounts_by_status(&conn, &status)
    } else {
        database::get_all_accounts(&conn)
    };

    let accounts = accounts.map_err(|e| e.to_string())?;

    if accounts.is_empty() {
        return Ok(String::new());
    }

    let mut lines = Vec::new();
    for account in accounts {
        // Format: email----password----client_id----refresh_token
        let line = format!(
            "{}----{}----{}----{}",
            account.email, account.email_password, account.client_id, account.refresh_token
        );
        lines.push(line);
    }

    Ok(lines.join("\n"))
}

// ===== Kiro IDE 桌面授权相关命令 =====

/// 生成 Kiro IDE Social 授权（Google/GitHub/Apple）
#[tauri::command]
pub async fn generate_kiro_social_auth(
    access_token: String,
    refresh_token: String,
    provider: String,
    profile_arn: Option<String>,
) -> Result<String, String> {
    let params = SocialAuthParams {
        access_token,
        refresh_token,
        provider: provider.clone(),
        profile_arn,
    };

    kiro_auth::generate_kiro_social_auth(params)?;

    Ok(format!("✓ Kiro {} Social 授权文件已生成", provider))
}

/// 生成 Kiro IDE IdC 授权（AWS Builder ID）
#[tauri::command]
pub async fn generate_kiro_idc_auth(
    access_token: String,
    refresh_token: String,
    provider: String,
    client_id: String,
    client_secret: String,
    client_id_hash: String,
    region: Option<String>,
) -> Result<String, String> {
    let params = IdcAuthParams {
        access_token,
        refresh_token,
        provider: provider.clone(),
        client_id,
        client_secret,
        client_id_hash,
        region: region.unwrap_or_else(|| "us-east-1".to_string()),
    };

    kiro_auth::generate_kiro_idc_auth(params)?;

    Ok(format!("✓ Kiro {} IdC 授权文件已生成", provider))
}

/// 读取当前 Kiro IDE 授权信息
#[tauri::command]
pub async fn read_kiro_auth_token() -> Result<kiro_auth::KiroAuthToken, String> {
    kiro_auth::read_kiro_auth_token()
}

/// 模拟 Kiro IDE 登录（注册成功后自动生成授权）
#[tauri::command]
pub async fn simulate_kiro_login(
    email: String,
    _kiro_password: String,
    auth_method: String,
) -> Result<String, String> {
    // 这里可以根据实际需求实现完整的登录流程
    // 目前先返回一个示例响应

    match auth_method.as_str() {
        "social" => {
            // 模拟 Social 登录，生成假的 token
            let access_token = format!("social_access_token_{}", uuid::Uuid::new_v4());
            let refresh_token = format!("social_refresh_token_{}", uuid::Uuid::new_v4());

            let params = SocialAuthParams {
                access_token,
                refresh_token,
                provider: "Google".to_string(),
                profile_arn: Some(
                    "arn:aws:codewhisperer:us-east-1:699475941385:profile/EHGA3GRVQMUK".to_string(),
                ),
            };

            kiro_auth::generate_kiro_social_auth(params)?;
            Ok(format!("✓ 已为 {} 生成 Kiro Social 授权", email))
        }
        "idc" => {
            // 模拟 IdC 登录，生成假的 token
            let access_token = format!("idc_access_token_{}", uuid::Uuid::new_v4());
            let refresh_token = format!("idc_refresh_token_{}", uuid::Uuid::new_v4());
            let client_id = format!("client_{}", uuid::Uuid::new_v4());
            let client_secret = format!("secret_{}", uuid::Uuid::new_v4());

            // 生成 client_id_hash
            use sha2::{Digest, Sha256};
            let mut hasher = Sha256::new();
            hasher.update(client_id.as_bytes());
            let client_id_hash = hex::encode(hasher.finalize());

            let params = IdcAuthParams {
                access_token,
                refresh_token,
                provider: "BuilderId".to_string(),
                client_id,
                client_secret,
                client_id_hash,
                region: "us-east-1".to_string(),
            };

            kiro_auth::generate_kiro_idc_auth(params)?;
            Ok(format!("✓ 已为 {} 生成 Kiro IdC 授权", email))
        }
        _ => Err(format!("不支持的授权方式: {}", auth_method)),
    }
}

// ===== 真实的 Kiro OAuth 登录 =====

/// 执行真实的 Kiro Social OAuth 登录（Google）
#[tauri::command]
pub async fn kiro_oauth_login_google() -> Result<kiro_oauth::KiroOAuthResult, String> {
    kiro_oauth::perform_kiro_social_login("Google").await
}

/// 执行真实的 Kiro Social OAuth 登录（GitHub）
#[tauri::command]
pub async fn kiro_oauth_login_github() -> Result<kiro_oauth::KiroOAuthResult, String> {
    kiro_oauth::perform_kiro_social_login("Github").await
}

/// 执行真实的 AWS Builder ID 登录
#[tauri::command]
pub async fn kiro_oauth_login_builder_id() -> Result<kiro_oauth::KiroOAuthResult, String> {
    kiro_oauth::perform_builder_id_login().await
}

/// 处理 OAuth 回调 URL（由前端调用）
#[tauri::command]
pub fn handle_oauth_callback_url(url: String) -> bool {
    kiro_oauth::handle_oauth_callback(&url)
}

// ===== Builder ID 自动化登录 =====

/// 使用浏览器自动化完成 Builder ID 登录（推荐）
#[tauri::command]
pub async fn builder_id_automated_login(
    db: State<'_, DbState>,
    account_id: i64,
) -> Result<String, String> {
    // 获取账号信息
    let account = {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        database::get_account_by_id(&conn, account_id).map_err(|e| e.to_string())?
    };

    // 获取浏览器设置
    let settings = {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        database::get_settings(&conn).map_err(|e| e.to_string())?
    };

    println!("\n========== Builder ID 自动化登录 ==========");
    println!("账号: {}", account.email);

    // 执行自动化登录
    let result = builder_id_automation::perform_automated_builder_id_login(
        &account.email,
        &account.email_password,
        settings.browser_mode,
    )
    .await
    .map_err(|e| e.to_string())?;

    println!("✓ Builder ID 登录成功！");
    println!("✓ 授权文件已保存到 ~/.aws/sso/cache/");

    Ok(format!(
        "✓ AWS Builder ID 登录成功！\n\n授权文件已保存:\n- kiro-auth-token.json\n- {}.json",
        result.client_id_hash
    ))
}

// ===== 集成注册和 OAuth 授权 =====

/// 注册并自动完成 OAuth 授权（推荐）
#[tauri::command]
pub async fn start_registration_with_oauth(
    db: State<'_, DbState>,
    app_handle: tauri::AppHandle,
    account_id: i64,
) -> Result<String, String> {
    // 获取账号信息
    let account = {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        database::get_account_by_id(&conn, account_id).map_err(|e| e.to_string())?
    };

    // 更新状态为进行中
    {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        database::update_account(
            &conn,
            AccountUpdate {
                id: account_id,
                email: None,
                email_password: None,
                client_id: None,
                refresh_token: None,
                kiro_password: None,
                status: Some(AccountStatus::InProgress),
                error_reason: None,
                oauth_status: None,
                oauth_info: None,
            },
        )
        .map_err(|e| e.to_string())?;
    }

    // 获取浏览器设置
    let settings = {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        database::get_settings(&conn).map_err(|e| e.to_string())?
    };

    // 生成随机姓名
    let names = vec![
        "Zhang Wei",
        "Wang Fang",
        "Li Na",
        "Liu Yang",
        "Chen Jing",
        "Zhang Min",
        "Wang Lei",
        "Li Qiang",
        "Liu Min",
        "Chen Wei",
    ];
    let random_name = names[rand::random::<usize>() % names.len()];

    // 执行集成注册和 OAuth 授权
    let result = integrated_registration::perform_integrated_registration_and_oauth(
        &account.email,
        &account.email_password,
        &account.client_id,
        &account.refresh_token,
        random_name,
        settings.browser_mode,
        settings.email_mode,
        account_id,
        app_handle,
    )
    .await;

    match result {
        Ok(integrated_result) => {
            // 更新账号状态为已注册
            let conn = db.0.lock().map_err(|e| e.to_string())?;
            database::update_account(
                &conn,
                AccountUpdate {
                    id: account_id,
                    email: None,
                    email_password: None,
                    client_id: None,
                    refresh_token: None,
                    kiro_password: Some(integrated_result.kiro_password.clone()),
                    status: Some(AccountStatus::Registered),
                    error_reason: None,
                    oauth_status: None,
                    oauth_info: None,
                },
            )
            .map_err(|e| e.to_string())?;

            // 构建返回消息
            let mut message = format!("✓ 注册成功！密码: {}\n", integrated_result.kiro_password);

            if integrated_result.oauth_completed {
                message.push_str("\n✓ OAuth 授权成功！\n");
                if let Some(hash) = integrated_result.client_id_hash {
                    message.push_str(&format!(
                        "✓ 授权文件已保存:\n  - kiro-auth-token.json\n  - {}.json",
                        hash
                    ));
                }
            } else {
                message.push_str("\n⚠ OAuth 授权失败\n");
                if let Some(msg) = integrated_result.oauth_message {
                    message.push_str(&format!("  {}\n", msg));
                }
                message.push_str("  可以点击授权按钮手动重试");
            }

            Ok(message)
        }
        Err(e) => {
            // 更新账号状态为错误
            let conn = db.0.lock().map_err(|e| e.to_string())?;
            database::update_account(
                &conn,
                AccountUpdate {
                    id: account_id,
                    email: None,
                    email_password: None,
                    client_id: None,
                    refresh_token: None,
                    kiro_password: None,
                    status: Some(AccountStatus::Error),
                    error_reason: Some(e.to_string()),
                    oauth_status: None,
                    oauth_info: None,
                },
            )
            .map_err(|e| e.to_string())?;

            Err(e.to_string())
        }
    }
}

// ===== OAuth 授权管理命令 =====

/// 获取账号的 OAuth 授权详情
#[tauri::command]
pub async fn get_oauth_info(
    db: State<'_, DbState>,
    account_id: i64,
) -> Result<Option<OAuthInfo>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let account = database::get_account_by_id(&conn, account_id).map_err(|e| e.to_string())?;

    if let Some(oauth_info_json) = account.oauth_info {
        let oauth_info: OAuthInfo = serde_json::from_str(&oauth_info_json)
            .map_err(|e| format!("Failed to parse OAuth info: {}", e))?;
        Ok(Some(oauth_info))
    } else {
        Ok(None)
    }
}

/// 手动执行 OAuth 授权（仅对已注册账号）
#[tauri::command]
pub async fn manual_oauth_authorization(
    db: State<'_, DbState>,
    account_id: i64,
) -> Result<String, String> {
    // 获取账号信息
    let account = {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        database::get_account_by_id(&conn, account_id).map_err(|e| e.to_string())?
    };

    // 检查账号是否已注册
    if account.status != AccountStatus::Registered {
        return Err("账号尚未注册，请先完成注册".to_string());
    }

    if account.kiro_password.is_none() {
        return Err("账号缺少 Kiro 密码，请重新注册".to_string());
    }

    // 更新 OAuth 状态为进行中
    {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        database::update_account(
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
        )
        .map_err(|e| e.to_string())?;
    }

    // 获取浏览器设置
    let settings = {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        database::get_settings(&conn).map_err(|e| e.to_string())?
    };

    // 执行 Builder ID 自动化登录
    let result = builder_id_automation::perform_automated_builder_id_login(
        &account.email,
        &account.email_password,
        settings.browser_mode,
    )
    .await;

    match result {
        Ok(login_result) => {
            // 保存 OAuth 信息到数据库
            let oauth_info = OAuthInfo {
                access_token: login_result.access_token,
                refresh_token: login_result.refresh_token,
                provider: "BuilderId".to_string(),
                auth_method: "IdC".to_string(),
                expires_at: (chrono::Utc::now() + chrono::Duration::hours(1)).to_rfc3339(),
                profile_arn: None,
                client_id_hash: Some(login_result.client_id_hash.clone()),
                region: Some(login_result.region),
                authorized_at: chrono::Utc::now().to_rfc3339(),
                // 客户端注册信息
                client_id: Some(login_result.client_id.clone()),
                client_secret: Some(login_result.client_secret),
                client_expires_at: Some(
                    (chrono::Utc::now() + chrono::Duration::days(90)).to_rfc3339(),
                ),
            };

            let oauth_info_json = serde_json::to_string(&oauth_info)
                .map_err(|e| format!("Failed to serialize OAuth info: {}", e))?;

            // 更新数据库中的 OAuth 状态和信息
            let conn = db.0.lock().map_err(|e| e.to_string())?;
            database::update_account(
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
            .map_err(|e| e.to_string())?;

            Ok(format!(
                "✓ OAuth 授权成功！\n\n授权文件已保存:\n- kiro-auth-token.json\n- {}.json",
                login_result.client_id_hash
            ))
        }
        Err(e) => {
            // 更新 OAuth 状态为错误
            let conn = db.0.lock().map_err(|e| e.to_string())?;
            database::update_account(
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
                    oauth_status: Some(OAuthStatus::Error),
                    oauth_info: None,
                },
            )
            .map_err(|e| e.to_string())?;

            Err(format!("OAuth 授权失败: {}", e))
        }
    }
}

/// 导出账号的 Kiro 授权 JSON 文件
#[tauri::command]
pub async fn export_kiro_auth_json(
    db: State<'_, DbState>,
    account_id: i64,
) -> Result<String, String> {
    // 获取账号信息
    let account = {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        database::get_account_by_id(&conn, account_id).map_err(|e| e.to_string())?
    };

    // 检查账号是否有 OAuth 信息
    let oauth_info_json = account.oauth_info.ok_or("账号尚未完成 OAuth 授权")?;
    let oauth_info: OAuthInfo = serde_json::from_str(&oauth_info_json)
        .map_err(|e| format!("解析 OAuth 信息失败: {}", e))?;

    // 构建 Kiro 授权 Token
    let kiro_token = kiro_auth::KiroAuthToken {
        access_token: oauth_info.access_token,
        refresh_token: oauth_info.refresh_token,
        expires_at: oauth_info.expires_at,
        auth_method: oauth_info.auth_method,
        provider: oauth_info.provider,
        profile_arn: oauth_info.profile_arn,
        client_id_hash: oauth_info.client_id_hash,
        region: oauth_info.region,
    };

    // 序列化为 JSON
    let json_content =
        serde_json::to_string_pretty(&kiro_token).map_err(|e| format!("序列化失败: {}", e))?;

    Ok(json_content)
}
