# IMAP + OAuth2 实现说明

## 实现参考

本实现参考了以下三个成熟的实现：

1. **server.js** (Node.js + imapflow)
2. **imap_client.py** (Python + imaplib + msal)
3. **async-imap** (Rust 库)

## 关键技术点

### 1. OAuth2 Scope

**正确的 Scope:**
```
https://outlook.office.com/IMAP.AccessAsUser.All offline_access
```

**注意事项:**
- 使用 `outlook.office.com` 而不是 `outlook.office365.com`
- 不能使用 `.default` scope
- 必须包含 `offline_access` 以获取 refresh_token

### 2. XOAUTH2 认证

**认证字符串格式:**
```
user=<email>\x01auth=Bearer <access_token>\x01\x01
```

**实现细节:**
- `\x01` 是 ASCII 控制字符 (SOH - Start of Heading)
- 字符串需要 base64 编码（async-imap 库会自动处理）
- 使用 SASL XOAUTH2 机制

**Rust 实现:**
```rust
struct XOAuth2 {
    user: String,
    access_token: String,
}

impl async_imap::Authenticator for XOAuth2 {
    type Response = String;

    fn process(&mut self, _data: &[u8]) -> Self::Response {
        format!(
            "user={}\x01auth=Bearer {}\x01\x01",
            self.user, self.access_token
        )
    }
}
```

### 3. IMAP 连接流程

```
1. 连接到 outlook.office365.com:993 (TLS)
2. 使用 XOAUTH2 认证
3. 选择 INBOX
4. 获取邮件（使用序列号范围）
5. 解析邮件
6. 提取验证码
7. 断开连接
```

### 4. 邮件获取策略

**使用序列号范围而不是搜索:**
```rust
// 计算起始序列号
let start_seq = if mailbox.exists > limit as u32 {
    mailbox.exists - limit as u32 + 1
} else {
    1
};

// 使用范围获取
let range = format!("{}:*", start_seq);
session.fetch(&range, "RFC822")
```

**优点:**
- 不依赖 IMAP SEARCH 命令（有些服务器可能不支持或有限制）
- 总是获取最新的邮件
- 更可靠

### 5. 邮件解析

**使用 mailparse 库:**
```rust
use mailparse::parse_mail;

if let Ok(parsed) = parse_mail(body) {
    // 提取发件人
    let from = parsed.headers.iter()
        .find(|h| h.get_key().eq_ignore_ascii_case("From"))
        .map(|h| h.get_value())
        .unwrap_or_else(|| "Unknown".to_string());
    
    // 提取正文
    let body_text = parsed.get_body()?;
}
```

### 6. 验证码提取

**使用正则表达式:**
```rust
use regex::Regex;

fn extract_verification_code(body: &str) -> Option<String> {
    let re = Regex::new(r"\b(\d{6})\b").ok()?;
    
    if let Some(captures) = re.captures(body) {
        if let Some(code) = captures.get(1) {
            return Some(code.as_str().to_string());
        }
    }
    
    None
}
```

**匹配规则:**
- 查找 6 位连续数字
- 使用单词边界 `\b` 确保是独立的数字
- 支持 HTML 和纯文本邮件

### 7. 重试机制

**循环获取邮件:**
```rust
loop {
    // 检查超时
    if elapsed > timeout_duration {
        return Err(anyhow!("Timeout"));
    }
    
    // 获取邮件
    match fetch_recent_emails(...).await {
        Ok(emails) => {
            // 查找验证码
            for (from, body) in emails {
                if from.contains("kiro") {
                    if let Some(code) = extract_code(&body) {
                        return Ok(code);
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }
    
    // 等待 5 秒后重试
    tokio::time::sleep(Duration::from_secs(5)).await;
}
```

## 与其他实现的对比

### server.js (Node.js)

**相似点:**
- 使用序列号范围获取邮件
- OAuth2 认证流程
- 相同的 scope

**差异:**
- 使用 `imapflow` 库（更现代）
- 直接传递 `accessToken` 参数

### imap_client.py (Python)

**相似点:**
- XOAUTH2 认证字符串格式
- 使用 MSAL 库获取 token
- 详细的日志输出

**差异:**
- 使用 `imaplib` 标准库
- 手动构造 IMAP 命令
- 使用 SASL-IR (初始响应)

### 本实现 (Rust)

**优势:**
- 类型安全
- 异步处理
- 内存安全
- 性能优秀

**特点:**
- 使用 `async-imap` 库
- 使用 `tokio` 异步运行时
- 使用 `mailparse` 解析邮件
- 完整的错误处理

## 常见问题

### Q: 为什么使用序列号范围而不是搜索？

**A:** 
1. IMAP SEARCH 命令在某些服务器上可能不可靠
2. 日期格式可能有兼容性问题
3. 序列号范围总是返回最新的邮件
4. 更简单、更可靠

### Q: 为什么每次循环都重新连接？

**A:**
1. 避免会话状态问题
2. 确保获取最新的邮件
3. 更容易处理错误
4. 参考了 server.js 的实现方式

### Q: 为什么不使用 imapflow？

**A:**
1. `imapflow` 是 Node.js 库，Rust 没有对应的库
2. `async-imap` 是 Rust 生态中最成熟的 IMAP 库
3. 功能足够满足需求

### Q: XOAUTH2 认证为什么需要特殊格式？

**A:**
这是 RFC 7628 定义的 SASL XOAUTH2 机制的标准格式：
- `user=<email>` - 用户标识
- `\x01` - 分隔符
- `auth=Bearer <token>` - 认证令牌
- `\x01\x01` - 结束标记

## 调试技巧

### 1. 启用详细日志

代码中已经包含了详细的日志输出：
```
[IMAP] Getting access token for client_id: xxx
[IMAP] OAuth2 response status: 200 OK
[IMAP] Successfully obtained access token
[IMAP] Connecting to outlook.office365.com:993
[IMAP] TCP connection established, starting TLS handshake
[IMAP] TLS connection established, creating IMAP client
[IMAP] Authenticating with XOAUTH2 for user: user@example.com
[IMAP] Token length: 1234 chars
[IMAP] Authentication successful
[IMAP] INBOX selected, total messages: 42
[IMAP] Fetching messages in range: 23:*
[IMAP] Email from: noreply@kiro.dev, subject: Your verification code
[IMAP] ✓ Found verification code: 123456
```

### 2. 检查 Token

```rust
println!("[DEBUG] Access token: {}", &access_token[..50]); // 只显示前50个字符
```

### 3. 检查邮件内容

```rust
println!("[DEBUG] Email body: {}", &body_text[..200]); // 显示前200个字符
```

### 4. 测试 OAuth2

使用 curl 测试 token 刷新：
```bash
curl -X POST https://login.microsoftonline.com/common/oauth2/v2.0/token \
  -d "client_id=YOUR_CLIENT_ID" \
  -d "refresh_token=YOUR_REFRESH_TOKEN" \
  -d "grant_type=refresh_token" \
  -d "scope=https://outlook.office.com/IMAP.AccessAsUser.All offline_access"
```

## 性能优化

### 1. 连接池

当前实现每次都重新连接。如果需要频繁获取邮件，可以考虑：
- 维护连接池
- 复用 IMAP 会话
- 使用 IDLE 命令监听新邮件

### 2. 并发处理

如果需要同时处理多个账号：
```rust
use futures::future::join_all;

let tasks: Vec<_> = accounts.iter()
    .map(|account| {
        let client = ImapClient::new();
        async move {
            client.wait_for_verification_code(...).await
        }
    })
    .collect();

let results = join_all(tasks).await;
```

### 3. 缓存 Access Token

Access token 有效期通常是 1 小时，可以缓存：
```rust
struct TokenCache {
    token: String,
    expires_at: DateTime<Utc>,
}
```

## 安全建议

1. **不要记录完整的 access_token**
2. **使用 HTTPS 传输 refresh_token**
3. **定期轮换 refresh_token**
4. **限制 token 的权限范围**
5. **使用环境变量存储敏感信息**

## 参考资料

- [RFC 7628 - SASL XOAUTH2](https://tools.ietf.org/html/rfc7628)
- [Microsoft Identity Platform OAuth2](https://docs.microsoft.com/en-us/azure/active-directory/develop/v2-oauth2-auth-code-flow)
- [IMAP Protocol RFC 3501](https://tools.ietf.org/html/rfc3501)
- [async-imap Documentation](https://docs.rs/async-imap/)
- [mailparse Documentation](https://docs.rs/mailparse/)

## 更新历史

- **2025-01-17**: 初始实现，参考 server.js 和 imap_client.py
- **2025-01-17**: 修复 scope 错误（office365.com → office.com）
- **2025-01-17**: 改用序列号范围获取邮件
- **2025-01-17**: 添加详细日志输出
- **2025-01-17**: 优化错误处理
