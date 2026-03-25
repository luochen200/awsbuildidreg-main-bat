# AWS CLI SSO 登录分析

## AWS SSO 登录流程

当你使用 `aws sso login` 命令时，AWS CLI 会：

1. 打开浏览器进行 SSO 认证
2. 在本地缓存目录存储认证信息
3. 生成访问令牌和刷新令牌

## 关键文件位置

### Windows 系统
```
%USERPROFILE%\.aws\sso\cache\
%USERPROFILE%\.aws\cli\cache\
```

### Linux/Mac 系统
```
~/.aws/sso/cache/
~/.aws/cli/cache/
```

## 缓存文件内容

AWS SSO 会在缓存目录中创建 JSON 文件，包含：

### 1. SSO Token 文件
文件名格式：`{hash}.json`

内容示例：
```json
{
  "accessToken": "eyJraWQiOiJ...",
  "expiresAt": "2024-01-17T12:00:00Z",
  "region": "us-east-1",
  "startUrl": "https://d-xxxxxxxxxx.awsapps.com/start"
}
```

### 2. OIDC Token 文件
可能包含：
```json
{
  "accessToken": "...",
  "refreshToken": "...",
  "clientId": "...",
  "clientSecret": "...",
  "expiresAt": "...",
  "region": "..."
}
```

## 如何获取这些信息

### 方法1：直接读取缓存文件

```bash
# Windows
dir %USERPROFILE%\.aws\sso\cache\
type %USERPROFILE%\.aws\sso\cache\*.json

# Linux/Mac
ls -la ~/.aws/sso/cache/
cat ~/.aws/sso/cache/*.json
```

### 方法2：使用 AWS CLI 命令

```bash
# 查看当前 SSO 配置
aws configure list
aws configure list-profiles

# 查看 SSO 会话
aws sso list-accounts
```

### 方法3：监控浏览器网络请求

在 AWS SSO 登录过程中，浏览器会发送请求到：
- `https://oidc.{region}.amazonaws.com/`
- `https://{sso-domain}.awsapps.com/`

这些请求的响应中可能包含：
- `access_token`
- `refresh_token`
- `client_id`
- `client_secret`

## 实现方案

### 方案A：解析 AWS CLI 缓存文件

```rust
use std::fs;
use std::path::PathBuf;
use serde_json::Value;

fn get_aws_sso_credentials() -> Result<(String, String, String)> {
    // 获取用户主目录
    let home_dir = dirs::home_dir().ok_or("Cannot find home directory")?;
    
    // AWS SSO 缓存目录
    let sso_cache_dir = home_dir.join(".aws").join("sso").join("cache");
    
    // 读取所有 JSON 文件
    for entry in fs::read_dir(sso_cache_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            let content = fs::read_to_string(&path)?;
            let json: Value = serde_json::from_str(&content)?;
            
            // 提取所需字段
            if let (Some(client_id), Some(refresh_token), Some(client_secret)) = (
                json.get("clientId").and_then(|v| v.as_str()),
                json.get("refreshToken").and_then(|v| v.as_str()),
                json.get("clientSecret").and_then(|v| v.as_str()),
            ) {
                return Ok((
                    client_id.to_string(),
                    refresh_token.to_string(),
                    client_secret.to_string(),
                ));
            }
        }
    }
    
    Err("SSO credentials not found".into())
}
```

### 方案B：拦截浏览器请求

在浏览器自动化过程中，监听网络请求：

```rust
// 在浏览器中注入脚本，拦截 fetch/XMLHttpRequest
let intercept_script = r#"
(function() {
    const originalFetch = window.fetch;
    window.fetch = function(...args) {
        return originalFetch.apply(this, args).then(response => {
            const clonedResponse = response.clone();
            clonedResponse.json().then(data => {
                if (data.refreshToken || data.clientId || data.clientSecret) {
                    console.log('INTERCEPTED_CREDENTIALS:', JSON.stringify(data));
                }
            }).catch(() => {});
            return response;
        });
    };
    
    const originalXHR = window.XMLHttpRequest.prototype.open;
    window.XMLHttpRequest.prototype.open = function(...args) {
        this.addEventListener('load', function() {
            try {
                const data = JSON.parse(this.responseText);
                if (data.refreshToken || data.clientId || data.clientSecret) {
                    console.log('INTERCEPTED_CREDENTIALS:', JSON.stringify(data));
                }
            } catch(e) {}
        });
        return originalXHR.apply(this, args);
    };
})();
"#;
```

### 方案C：使用 AWS SDK

```rust
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_sso::Client as SsoClient;

async fn get_sso_token() -> Result<String> {
    let region_provider = RegionProviderChain::default_provider().or_else("us-east-1");
    let config = aws_config::from_env().region(region_provider).load().await;
    let client = SsoClient::new(&config);
    
    // 使用 SSO 客户端获取令牌
    // ...
}
```

## 推荐方案

### 最简单的方法：手动执行 AWS SSO 登录

1. 用户先执行：
```bash
aws sso login --profile your-profile
```

2. 登录完成后，读取缓存文件：
```bash
# Windows PowerShell
Get-Content $env:USERPROFILE\.aws\sso\cache\*.json | ConvertFrom-Json

# Linux/Mac
cat ~/.aws/sso/cache/*.json | jq .
```

3. 从输出中提取：
   - `clientId`
   - `refreshToken`
   - `clientSecret` (如果有)
   - `accessToken`

## 测试步骤

1. 安装 AWS CLI（如果还没有）
2. 配置 SSO：
```bash
aws configure sso
```

3. 登录：
```bash
aws sso login
```

4. 查看缓存文件：
```bash
# Windows
type %USERPROFILE%\.aws\sso\cache\*.json

# Linux/Mac
cat ~/.aws/sso/cache/*.json
```

5. 查找包含以下字段的文件：
   - `refreshToken`
   - `clientId`
   - `clientSecret`
   - `accessToken`

## 注意事项

1. **Token 过期时间**：这些 token 通常有过期时间（expiresAt）
2. **安全性**：这些是敏感信息，需要妥善保管
3. **区域差异**：不同 AWS 区域的 SSO 端点可能不同
4. **文件格式**：缓存文件可能是加密的或使用特殊格式

## 下一步

如果你想测试这个方法：
1. 先手动执行 `aws sso login`
2. 查看生成的缓存文件
3. 告诉我文件内容的结构（去掉敏感信息）
4. 我可以帮你编写代码自动提取这些信息
