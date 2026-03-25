# AWS OAuth 凭证自动截获功能使用指南

## 功能概述

系统现在支持在 Kiro 注册成功后，自动登录 AWS 并截获 AWS OAuth 凭证（Refresh Token、Client ID、Client Secret）。

## 工作原理

### 1. 拦截机制

使用 JavaScript 注入技术，在浏览器中拦截所有网络请求：

```javascript
// 拦截 fetch 请求
window.fetch = function(...) {
    // 检查是否是 AWS OAuth token 请求
    if (url.includes('identity.api.aws.amazon.com/oauth2/token')) {
        // 捕获响应数据
        window.__awsTokenData = response;
    }
}

// 拦截 XMLHttpRequest
XMLHttpRequest.prototype.send = function(...) {
    // 同样的拦截逻辑
}
```

### 2. 目标 API 端点

拦截器会监听以下 AWS OAuth 端点：
- `https://identity.api.aws.amazon.com/oauth2/token`
- 任何包含 `oidc` 和 `token` 的 AWS 域名
- 任何包含 `amazonaws.com` 和 `token` 的 URL

### 3. 捕获的数据格式

```json
{
  "url": "https://identity.api.aws.amazon.com/oauth2/token",
  "timestamp": "2025-01-17T12:00:00Z",
  "data": {
    "access_token": "eyJraWQiOiJ...",
    "refresh_token": "Atzr|IwEBIA...",
    "id_token": "eyJraWQiOiJ...",
    "expires_in": 3600,
    "token_type": "Bearer"
  }
}
```

## 完整流程

```
1. 用户启动注册
   ↓
2. 完成 Kiro 注册（输入邮箱、姓名、验证码、密码）
   ↓
3. Kiro 注册成功页面出现
   ↓
4. 🔹 注入 AWS OAuth 拦截器
   ↓
5. 🔹 自动导航到 AWS 登录页面
   ↓
6. 🔹 尝试使用 Kiro 邮箱登录 AWS
   ↓
7. 🔹 等待 AWS OAuth token 请求（最多 60 秒）
   ↓
8. 🔹 拦截器自动捕获凭证
   ↓
9. 🔹 提取并打印凭证到控制台
   ↓
10. 清理浏览器数据，完成流程
```

## 使用方法

### 1. 设置浏览器模式

为了观察 AWS 登录流程，建议使用**前台模式**：

```typescript
// 在前端设置中选择
browserMode: 'foreground'
```

### 2. 启动注册

正常启动 Kiro 注册流程，系统会自动：
1. 完成 Kiro 注册
2. 尝试登录 AWS
3. 截获 OAuth 凭证

### 3. 查看结果

在控制台（终端）中查看输出：

```
🎉 Kiro 注册成功！现在开始登录 AWS 并截获凭证...

========== 开始 AWS 登录流程 ==========

✅ AWS OAuth token interceptor injected
📍 导航到 AWS 登录页面: https://console.aws.amazon.com/

========== AWS 登录页面 ==========
URL: https://console.aws.amazon.com/
...

⏳ 等待 AWS OAuth token 请求...

[AWS INTERCEPTOR] Detected AWS OAuth token request: https://identity.api.aws.amazon.com/oauth2/token
[AWS INTERCEPTOR] ✅ AWS OAuth credentials captured!

✅ 成功截获 AWS OAuth 凭证！
AWS Credentials: {
  "url": "https://identity.api.aws.amazon.com/oauth2/token",
  "timestamp": "2025-01-17T12:00:00Z",
  "data": {
    "access_token": "eyJraWQiOiJ...",
    "refresh_token": "Atzr|IwEBIA...",
    "id_token": "eyJraWQiOiJ...",
    "expires_in": 3600
  }
}
```

## AWS 登录页面适配

### 当前支持的选择器

代码会尝试查找以下元素：

```rust
let builder_id_selectors = vec![
    "//button[contains(text(), 'Builder ID')]",
    "//a[contains(text(), 'Builder ID')]",
    "//button[contains(text(), 'Sign in')]",
    "//input[@id='resolving_input']", // Email input
];
```

### 如何调整选择器

如果 AWS 登录页面结构发生变化，你需要：

1. **运行前台模式**，观察实际的页面结构
2. **使用浏览器开发者工具**，找到正确的元素
3. **更新选择器**：

```rust
// 在 src-tauri/src/commands.rs 的 login_aws_and_capture_credentials 函数中
let builder_id_selectors = vec![
    "你的新XPath1",
    "你的新XPath2",
    // ...
];
```

## 常见问题

### Q1: 拦截器没有捕获到凭证？

**可能原因：**
- AWS 登录流程没有触发 OAuth token 请求
- 页面结构变化，选择器失效
- 等待时间不够（默认 60 秒）

**解决方法：**
1. 使用前台模式观察实际流程
2. 检查控制台输出，看是否有 `[AWS INTERCEPTOR]` 日志
3. 调整等待时间或选择器

### Q2: 拦截器被页面刷新清除？

**解决方法：**
代码会自动检测并重新注入拦截器：

```rust
// 每 2 秒检查一次
if let Ok(false) = automation.is_aws_interceptor_installed(tab) {
    println!("⚠️  拦截器已失效，重新注入...");
    automation.inject_aws_token_interceptor(tab)?;
}
```

### Q3: AWS 登录需要额外的验证步骤？

**解决方法：**
如果 AWS 需要 MFA 或其他验证，你需要：
1. 在代码中添加相应的自动化步骤
2. 或者手动完成验证（在前台模式下）
3. 拦截器会继续等待 OAuth 请求

### Q4: 想要保存凭证到数据库？

**实现步骤：**

1. **添加数据库字段**（在 `src-tauri/src/database.rs`）：

```rust
CREATE TABLE accounts (
    ...
    aws_client_id TEXT,
    aws_client_secret TEXT,
    aws_refresh_token TEXT,
    ...
);
```

2. **在截获凭证后保存**（在 `src-tauri/src/commands.rs`）：

```rust
match login_aws_and_capture_credentials(&automation, &tab, email).await {
    Ok(Some(aws_creds)) => {
        // 提取凭证
        let refresh_token = aws_creds["data"]["refresh_token"]
            .as_str()
            .unwrap_or("");
        
        // 保存到数据库
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        database::update_account(
            &conn,
            AccountUpdate {
                id: account_id,
                aws_refresh_token: Some(refresh_token.to_string()),
                // ...
            },
        )?;
    }
    // ...
}
```

## 调试技巧

### 1. 查看浏览器控制台

在前台模式下，打开浏览器开发者工具（F12），查看 Console 标签：

```
[AWS INTERCEPTOR] Installing AWS OAuth token interceptor...
[AWS INTERCEPTOR] ✅ Interceptor installed successfully
[AWS INTERCEPTOR] Detected AWS OAuth token request: ...
[AWS INTERCEPTOR] Token response data: {...}
[AWS INTERCEPTOR] ✅ AWS OAuth credentials captured!
```

### 2. 增加日志输出

在代码中添加更多 `println!` 语句：

```rust
println!("当前 URL: {}", automation.get_current_url(tab)?);
println!("当前 Cookies: {}", automation.get_cookies(tab)?);
```

### 3. 调整等待时间

如果网络较慢，增加等待时间：

```rust
let timeout = std::time::Duration::from_secs(120); // 从 60 改为 120
```

## 安全注意事项

⚠️ **这些凭证非常敏感！**

1. **不要分享**：不要将截获的凭证分享给任何人
2. **不要提交到 Git**：确保凭证不会被提交到版本控制
3. **定期轮换**：定期更新和轮换凭证
4. **加密存储**：如果保存到数据库，考虑加密存储

## 下一步优化

### 1. 支持更多 AWS 登录方式
- AWS Builder ID
- IAM User
- Root User
- SSO

### 2. 自动处理 MFA
- 短信验证码
- TOTP（Google Authenticator）
- 硬件密钥

### 3. 智能重试机制
- 网络错误自动重试
- 页面加载失败重试
- 选择器失效时尝试其他方法

### 4. 凭证验证
- 截获后立即验证凭证是否有效
- 使用凭证调用 AWS API 测试
- 自动刷新过期的 token

## 技术细节

### 拦截器实现原理

```javascript
// 1. 保存原始方法
const originalFetch = window.fetch;

// 2. 重写方法
window.fetch = function(...args) {
    // 3. 调用原始方法
    return originalFetch.apply(this, args).then(response => {
        // 4. 克隆响应（不影响原始流程）
        const clonedResponse = response.clone();
        
        // 5. 检查并捕获数据
        clonedResponse.json().then(data => {
            if (isAwsOAuthToken(data)) {
                window.__awsTokenData = data;
            }
        });
        
        // 6. 返回原始响应
        return response;
    });
};
```

### 为什么使用 JavaScript 注入而不是代理？

**优点：**
- ✅ 不需要配置代理服务器
- ✅ 不需要处理 HTTPS 证书
- ✅ 代码简单，易于维护
- ✅ 不影响其他网络请求

**缺点：**
- ❌ 可能被 CSP（内容安全策略）阻止
- ❌ 页面刷新后需要重新注入
- ❌ 无法拦截 Service Worker 请求

## 参考资料

- [AWS Identity API 文档](https://docs.aws.amazon.com/singlesignon/latest/userguide/what-is.html)
- [OAuth 2.0 规范](https://oauth.net/2/)
- [Chrome DevTools Protocol](https://chromedevtools.github.io/devtools-protocol/)

## 需要帮助？

如果遇到问题：
1. 查看控制台输出
2. 使用前台模式观察流程
3. 检查 AWS 登录页面是否有变化
4. 调整选择器和等待时间
