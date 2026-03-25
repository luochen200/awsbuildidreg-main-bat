// Kiro IDE 桌面授权模块
// 模拟登录并生成 Kiro IDE 所需的授权文件

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Kiro 授权 Token 结构
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KiroAuthToken {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: String,
    pub auth_method: String,
    pub provider: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_arn: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_id_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
}

/// 客户端注册信息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientRegistration {
    pub client_id: String,
    pub client_secret: String,
    pub expires_at: String,
}

/// 获取 Kiro Token 文件路径
fn get_kiro_token_path() -> Result<PathBuf, String> {
    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .map_err(|_| "无法获取用户主目录")?;

    let path = PathBuf::from(home).join(".aws").join("sso").join("cache");

    std::fs::create_dir_all(&path).map_err(|e| format!("创建目录失败: {}", e))?;

    Ok(path.join("kiro-auth-token.json"))
}

/// 获取客户端注册文件路径
#[allow(dead_code)]
fn get_client_registration_path(client_id_hash: &str) -> Result<PathBuf, String> {
    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .map_err(|_| "无法获取用户主目录")?;

    let path = PathBuf::from(home).join(".aws").join("sso").join("cache");

    std::fs::create_dir_all(&path).map_err(|e| format!("创建目录失败: {}", e))?;

    Ok(path.join(format!("{}.json", client_id_hash)))
}

/// 写入 Kiro 授权 Token
/// 注意：此功能已禁用，改为使用界面导出功能配合外部工具导入
pub fn write_kiro_auth_token(_token: &KiroAuthToken) -> Result<(), String> {
    // 功能已禁用 - 使用界面导出功能代替
    // let path = get_kiro_token_path()?;
    //
    // let content = serde_json::to_string_pretty(token)
    //     .map_err(|e| format!("序列化失败: {}", e))?;
    //
    // // 原子写入：先写临时文件，再重命名
    // let temp_path = path.with_extension("tmp");
    // std::fs::write(&temp_path, &content)
    //     .map_err(|e| format!("写入临时文件失败: {}", e))?;
    // std::fs::rename(&temp_path, &path)
    //     .map_err(|e| format!("重命名文件失败: {}", e))?;
    //
    // println!("✓ Kiro 授权文件已写入: {}", path.display());

    println!("ℹ Kiro 授权文件写入功能已禁用，请使用界面导出功能");
    Ok(())
}

/// 写入客户端注册信息（IdC 专用）
/// 注意：此功能已禁用，改为使用界面导出功能配合外部工具导入
pub fn write_client_registration(
    _client_id_hash: &str,
    _registration: &ClientRegistration,
) -> Result<(), String> {
    // 功能已禁用 - 使用界面导出功能代替
    // let path = get_client_registration_path(client_id_hash)?;
    //
    // let content = serde_json::to_string_pretty(registration)
    //     .map_err(|e| format!("序列化失败: {}", e))?;
    //
    // let temp_path = path.with_extension("tmp");
    // std::fs::write(&temp_path, &content)
    //     .map_err(|e| format!("写入临时文件失败: {}", e))?;
    // std::fs::rename(&temp_path, &path)
    //     .map_err(|e| format!("重命名文件失败: {}", e))?;
    //
    // println!("✓ 客户端注册文件已写入: {}", path.display());

    println!("ℹ 客户端注册文件写入功能已禁用，请使用界面导出功能");
    Ok(())
}

/// 读取当前 Kiro 授权 Token
pub fn read_kiro_auth_token() -> Result<KiroAuthToken, String> {
    let path = get_kiro_token_path()?;

    if !path.exists() {
        return Err("Kiro 授权文件不存在".to_string());
    }

    let content = std::fs::read_to_string(&path).map_err(|e| format!("读取文件失败: {}", e))?;

    serde_json::from_str(&content).map_err(|e| format!("解析文件失败: {}", e))
}

/// 生成 Social 授权参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialAuthParams {
    pub access_token: String,
    pub refresh_token: String,
    pub provider: String,
    #[serde(default)]
    pub profile_arn: Option<String>,
}

/// 生成 IdC 授权参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdcAuthParams {
    pub access_token: String,
    pub refresh_token: String,
    pub provider: String,
    pub client_id: String,
    pub client_secret: String,
    pub client_id_hash: String,
    #[serde(default = "default_region")]
    pub region: String,
}

fn default_region() -> String {
    "us-east-1".to_string()
}

/// 生成 Kiro Social 授权
pub fn generate_kiro_social_auth(params: SocialAuthParams) -> Result<(), String> {
    let expires_at = chrono::Utc::now() + chrono::Duration::hours(1);

    let profile_arn = params.profile_arn.unwrap_or_else(|| {
        "arn:aws:codewhisperer:us-east-1:699475941385:profile/EHGA3GRVQMUK".to_string()
    });

    let token = KiroAuthToken {
        access_token: params.access_token,
        refresh_token: params.refresh_token,
        expires_at: expires_at.to_rfc3339(),
        auth_method: "social".to_string(),
        provider: params.provider,
        profile_arn: Some(profile_arn),
        client_id_hash: None,
        region: None,
    };

    write_kiro_auth_token(&token)
}

/// 生成 Kiro IdC 授权
pub fn generate_kiro_idc_auth(params: IdcAuthParams) -> Result<(), String> {
    let expires_at = chrono::Utc::now() + chrono::Duration::hours(1);
    let client_expires = chrono::Utc::now() + chrono::Duration::days(90);

    // 写入 Token 文件
    let token = KiroAuthToken {
        access_token: params.access_token,
        refresh_token: params.refresh_token,
        expires_at: expires_at.to_rfc3339(),
        auth_method: "IdC".to_string(),
        provider: params.provider,
        profile_arn: None,
        client_id_hash: Some(params.client_id_hash.clone()),
        region: Some(params.region),
    };

    write_kiro_auth_token(&token)?;

    // 写入客户端注册文件
    let registration = ClientRegistration {
        client_id: params.client_id,
        client_secret: params.client_secret,
        expires_at: client_expires.to_rfc3339(),
    };

    write_client_registration(&params.client_id_hash, &registration)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_path() {
        let path = get_kiro_token_path();
        assert!(path.is_ok());
        println!("Token path: {:?}", path.unwrap());
    }
}
