# IMAP + OAuth 邮件接收模式设置指南

## 概述

本系统现在支持两种邮件验证码接收方式：

1. **Microsoft Graph API**（推荐）- 速度快，直接通过 API 获取
2. **IMAP + OAuth**（备用）- 使用标准 IMAP 协议连接 Outlook

## 前提条件

无论使用哪种模式，你都需要准备以下信息：

- 邮箱地址（Outlook/Hotmail）
- 邮箱密码
- Azure AD 应用的 Client ID
- Refresh Token

## 如何选择邮件接收模式

### Microsoft Graph API 模式（默认）

**优点：**

- 速度快，响应迅速
- API 调用更稳定
- 推荐用于大批量注册

**缺点：**

- 需要 Azure AD 应用有 Mail.Read 权限
- 可能受到 API 限流影响

### IMAP + OAuth 模式

**优点：**

- 使用标准 IMAP 协议，兼容性好
- 不受 Graph API 限流影响
- 适合作为备用方案

**缺点：**

- 速度相对较慢
- 需要 IMAP 访问权限

## 使用步骤

### 1. 打开系统设置

在应用主界面，点击左侧控制面板的"系统设置"按钮。

### 2. 选择邮件接收模式

在设置对话框中，你会看到两个选项：

- **Microsoft Graph API** - 使用 Graph API 获取邮件（推荐，速度快）
- **IMAP + OAuth** - 使用 IMAP 协议获取邮件（备用方案）

选择你想要使用的模式。

### 3. 保存设置

点击"保存设置"按钮，设置将立即生效。

### 4. 开始注册

导入账号数据后，点击"开始注册"或"全部注册"按钮，系统将使用你选择的邮件接收模式自动获取验证码。

## 数据格式

无论选择哪种模式，导入的数据格式都是相同的：

```
邮箱地址----邮箱密码----客户端ID----refresh_token
```

示例：

```
user@outlook.com----password123----abc123-client-id----def456-refresh-token
```

## 权限要求

### Microsoft Graph API 模式

你的 Azure AD 应用需要以下权限：

- `Mail.Read` 或使用 `https://graph.microsoft.com/.default`
- `offline_access`

### IMAP + OAuth 模式

你的 Azure AD 应用需要以下权限：

- `https://outlook.office.com/IMAP.AccessAsUser.All` （必须）
- `offline_access`

**重要提示：**

- IMAP 模式**不能**使用 `.default` scope
- 必须使用完整的权限 URL：`https://outlook.office.com/IMAP.AccessAsUser.All`
- 注意是 `outlook.office.com` 而不是 `outlook.office365.com`
- 在 Azure AD 中添加权限时，选择 "Office 365 Exchange Online" API

## 故障排除

### Graph API 模式失败

如果 Graph API 模式失败，可能的原因：

1. Refresh Token 已过期
2. 应用权限不足
3. API 限流

**解决方案：** 切换到 IMAP 模式

### IMAP 模式失败

如果 IMAP 模式失败，可能的原因：

1. **Scope 错误** - 最常见的问题
   - ❌ 错误：使用 `https://outlook.office365.com/IMAP.AccessAsUser.All`
   - ✅ 正确：使用 `https://outlook.office.com/IMAP.AccessAsUser.All`
   - 注意域名是 `office.com` 而不是 `office365.com`
   - IMAP 不支持 `.default` scope，必须使用完整的权限 URL
2. **Refresh Token 不匹配** - Token 是用其他 scope 生成的
   - 需要使用正确的 scope 重新生成 refresh_token
3. **Azure AD 权限未配置**
   - 在 Azure Portal 中添加 "Office 365 Exchange Online" 的 IMAP 权限
   - 必须点击"授予管理员同意"
4. 网络连接问题

**解决方案：**

1. 检查 Azure AD 应用配置：
   - 进入 Azure Portal → Azure Active Directory → 应用注册
   - 找到你的应用
   - 点击"API 权限"
   - 点击"添加权限" → "Office 365 Exchange Online"
   - 选择"委托的权限" → 勾选 `IMAP.AccessAsUser.All`
   - 点击"授予管理员同意"
2. **重新生成 Refresh Token**（重要！）：
   - 使用正确的 scope：`https://outlook.office.com/IMAP.AccessAsUser.All offline_access`
   - 注意是 `office.com` 不是 `office365.com`
   - 旧的 refresh_token 如果是用其他 scope 生成的将无法使用
3. 检查网络连接
4. 查看应用日志获取详细错误信息

### 验证码获取超时

两种模式都有 60 秒超时机制：

- 如果 60 秒内未收到验证码，系统会自动点击"重发验证码"按钮
- 然后再等待 60 秒
- 如果仍未收到，注册将失败

## 技术细节

### IMAP 连接参数

- **服务器：** outlook.office365.com
- **端口：** 993
- **加密：** TLS/SSL
- **认证：** XOAUTH2

### 邮件搜索条件

系统会搜索：

- 发件人：kiro.dev
- 时间范围：最近 10 分钟
- 验证码格式：6 位数字

### 验证码提取

系统使用正则表达式自动提取邮件中的 6 位数字验证码：

```regex
\b(\d{6})\b
```

## 性能对比

| 特性         | Graph API  | IMAP     |
| ------------ | ---------- | -------- |
| 平均响应时间 | 2-5 秒     | 5-10 秒  |
| 稳定性       | 高         | 中       |
| 并发支持     | 好         | 一般     |
| 推荐场景     | 大批量注册 | 备用方案 |

## 最佳实践

1. **优先使用 Graph API 模式** - 速度快，稳定性好
2. **IMAP 作为备用** - 当 Graph API 出现问题时切换
3. **定期更新 Refresh Token** - 避免 Token 过期导致失败
4. **合理控制注册频率** - 避免触发限流机制

## 更新日志

### v1.1.0 (2025-01-17)

- ✨ 新增 IMAP + OAuth 邮件接收模式
- ✨ 支持在两种模式间自由切换
- 🔧 优化邮件验证码获取逻辑
- 📝 添加详细的设置指南

## 技术支持

如果遇到问题，请检查：

1. Refresh Token 是否有效
2. Azure AD 应用权限是否正确
3. 网络连接是否正常
4. 查看应用日志获取详细错误信息

### 如何获取正确的 Refresh Token

#### 方法 1: 使用 OAuth2 授权码流程

1. 构造授权 URL（根据你选择的模式）：

**Graph API 模式：**

```
https://login.microsoftonline.com/common/oauth2/v2.0/authorize?
client_id=YOUR_CLIENT_ID
&response_type=code
&redirect_uri=YOUR_REDIRECT_URI
&scope=https://graph.microsoft.com/.default offline_access
```

**IMAP 模式：**

```
https://login.microsoftonline.com/common/oauth2/v2.0/authorize?
client_id=YOUR_CLIENT_ID
&response_type=code
&redirect_uri=YOUR_REDIRECT_URI
&scope=https://outlook.office.com/IMAP.AccessAsUser.All offline_access
```

**注意：**

- IMAP 模式必须使用完整的 scope URL，不能使用 `.default`
- 使用 `outlook.office.com` 而不是 `outlook.office365.com`

2. 在浏览器中打开 URL，登录并授权
3. 获取返回的 authorization code
4. 使用 code 换取 refresh_token：

```bash
curl -X POST https://login.microsoftonline.com/common/oauth2/v2.0/token \
  -d "client_id=YOUR_CLIENT_ID" \
  -d "code=AUTHORIZATION_CODE" \
  -d "redirect_uri=YOUR_REDIRECT_URI" \
  -d "grant_type=authorization_code"
```

5. 响应中的 `refresh_token` 就是你需要的

#### 方法 2: 使用 Postman 或类似工具

1. 在 Postman 中创建新的 OAuth2 请求
2. 配置参数：
   - Auth URL: `https://login.microsoftonline.com/common/oauth2/v2.0/authorize`
   - Access Token URL: `https://login.microsoftonline.com/common/oauth2/v2.0/token`
   - Client ID: 你的应用 ID
   - Scope: 根据模式选择对应的 scope
3. 点击"Get New Access Token"
4. 登录并授权
5. 保存返回的 refresh_token

### 日志说明

应用会在控制台输出详细的日志信息，格式如下：

**Graph API 日志：**

```
[Graph API] Getting access token for client_id: xxx
[Graph API] OAuth2 response status: 200 OK
[Graph API] Successfully obtained access token
[Graph API] Fetching recent emails for: user@example.com
[Graph API] Found 5 messages
[Graph API] ✓ Found verification code: 123456
```

**IMAP 日志：**

```
[IMAP] Getting access token for client_id: xxx
[IMAP] OAuth2 response status: 200 OK
[IMAP] Successfully obtained access token
[IMAP] Connecting to outlook.office365.com:993
[IMAP] TCP connection established, starting TLS handshake
[IMAP] TLS connection established, creating IMAP client
[IMAP] Authenticating with XOAUTH2 for user: user@example.com
[IMAP] Authentication successful
[IMAP] INBOX selected successfully
[IMAP] Found 3 messages
[IMAP] ✓ Found verification code: 123456
```

如果看到错误，日志会显示详细的错误信息，帮助你诊断问题。

## 常见问题 FAQ

### Q1: 为什么 IMAP 模式报错 "invalid_scope"？

**A:** 这是因为 refresh_token 是用错误的 scope 生成的。IMAP 模式需要使用 `https://outlook.office.com/IMAP.AccessAsUser.All` scope（注意是 `office.com` 不是 `office365.com`），而不是 `.default`。

**解决方法：**

1. 使用正确的 scope 重新生成 refresh_token
2. 授权 URL 示例：

```
https://login.microsoftonline.com/common/oauth2/v2.0/authorize?
client_id=YOUR_CLIENT_ID
&response_type=code
&redirect_uri=YOUR_REDIRECT_URI
&scope=https://outlook.office.com/IMAP.AccessAsUser.All offline_access
```

### Q2: Graph API 和 IMAP 可以使用同一个 refresh_token 吗？

**A:** 不可以。两种模式需要不同的 scope，因此需要分别生成 refresh_token：

- Graph API: `https://graph.microsoft.com/.default offline_access`
- IMAP: `https://outlook.office.com/IMAP.AccessAsUser.All offline_access`

注意 IMAP 使用的是 `outlook.office.com` 而不是 `outlook.office365.com`。

如果你想同时支持两种模式，需要准备两套数据（不同的 refresh_token）。

### Q3: 如何知道我的 refresh_token 是用什么 scope 生成的？

**A:** 无法直接查看，但可以通过以下方式判断：

1. 尝试使用 - 如果报 "invalid_scope" 错误，说明 scope 不匹配
2. 查看生成 token 时的授权 URL
3. 重新生成一个新的 refresh_token（推荐）

### Q4: 为什么 Graph API 可以用 .default 但 IMAP 不行？

**A:** 这是 Microsoft OAuth2 的设计：

- Graph API 支持 `.default` scope，会自动包含应用配置的所有 Graph 权限
- Exchange Online (IMAP/SMTP/POP) 不支持 `.default`，必须明确指定权限

### Q5: 我应该选择哪种模式？

**A:** 建议：

- **优先使用 Graph API** - 速度快，稳定性好，适合大批量操作
- **IMAP 作为备用** - 当 Graph API 受限或出现问题时使用

### Q6: 可以在运行时切换模式吗？

**A:** 可以！在系统设置中随时切换，但前提是：

- 你的 refresh_token 必须是用对应模式的 scope 生成的
- 如果 token 不匹配，切换后会失败

### Q7: 如何在 Azure AD 中添加 IMAP 权限？

**A:** 步骤：

1. 登录 Azure Portal (portal.azure.com)
2. 进入 "Azure Active Directory" → "应用注册"
3. 选择你的应用
4. 点击 "API 权限" → "添加权限"
5. 选择 "Office 365 Exchange Online"（不是 Microsoft Graph）
6. 选择 "委托的权限"
7. 勾选 `IMAP.AccessAsUser.All`
8. 点击 "添加权限"
9. **重要：** 点击 "授予管理员同意"

### Q8: 验证码一直获取不到怎么办？

**A:** 检查以下几点：

1. 邮箱是否真的收到了验证码邮件
2. 发件人是否是 kiro.dev
3. 邮件是否在最近 10 分钟内
4. 查看日志，确认是否成功连接到邮箱
5. 尝试切换到另一种模式
