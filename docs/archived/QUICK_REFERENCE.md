# 快速参考 - IMAP OAuth2 配置

## 关键信息

### ✅ 正确的 Scope

```
https://outlook.office.com/IMAP.AccessAsUser.All offline_access
```

**重要：**
- 使用 `outlook.office.com` ✅
- 不是 `outlook.office365.com` ❌
- 不能使用 `.default` ❌

### 获取 Refresh Token

#### 步骤 1: 构造授权 URL

```
https://login.microsoftonline.com/common/oauth2/v2.0/authorize?client_id=YOUR_CLIENT_ID&response_type=code&redirect_uri=YOUR_REDIRECT_URI&scope=https://outlook.office.com/IMAP.AccessAsUser.All offline_access
```

替换：
- `YOUR_CLIENT_ID` - 你的 Azure AD 应用 ID
- `YOUR_REDIRECT_URI` - 你的回调 URL（需要在 Azure AD 中配置）

#### 步骤 2: 在浏览器中打开 URL

1. 复制上面的 URL 到浏览器
2. 登录 Microsoft 账号
3. 同意授权
4. 浏览器会跳转到回调 URL，URL 中包含 `code` 参数

#### 步骤 3: 用 Code 换取 Token

```bash
curl -X POST https://login.microsoftonline.com/common/oauth2/v2.0/token \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "client_id=YOUR_CLIENT_ID" \
  -d "code=AUTHORIZATION_CODE" \
  -d "redirect_uri=YOUR_REDIRECT_URI" \
  -d "grant_type=authorization_code"
```

响应示例：
```json
{
  "access_token": "eyJ0eXAi...",
  "refresh_token": "0.AXoA...",
  "expires_in": 3600,
  "scope": "https://outlook.office.com/IMAP.AccessAsUser.All"
}
```

保存 `refresh_token` 的值！

### Azure AD 权限配置

1. 登录 [Azure Portal](https://portal.azure.com)
2. 进入 **Azure Active Directory** → **应用注册**
3. 选择你的应用
4. 点击 **API 权限** → **添加权限**
5. 选择 **Office 365 Exchange Online**
6. 选择 **委托的权限**
7. 勾选 `IMAP.AccessAsUser.All`
8. 点击 **添加权限**
9. **重要：** 点击 **授予管理员同意**

### 数据格式

导入系统时使用以下格式：

```
邮箱地址----邮箱密码----客户端ID----refresh_token
```

示例：
```
user@outlook.com----password123----9e5f94bc-e8a4-4e73-b8be-63364c29d753----0.AXoABc1...
```

## 常见错误

### Error: invalid_scope

**原因：** Scope 不正确或 refresh_token 是用其他 scope 生成的

**解决：**
1. 确认使用 `https://outlook.office.com/IMAP.AccessAsUser.All`
2. 重新生成 refresh_token

### Error: AADSTS70011

**原因：** Scope 格式错误

**解决：**
- 检查是否使用了 `office.com` 而不是 `office365.com`
- 不要使用 `.default`

### Error: Authentication failed

**原因：** 
- Refresh token 过期
- Azure AD 权限未授予
- 未点击"授予管理员同意"

**解决：**
1. 重新生成 refresh_token
2. 在 Azure AD 中授予管理员同意

## 测试工具

### 使用 curl 测试 Token

```bash
# 测试 refresh token 是否有效
curl -X POST https://login.microsoftonline.com/common/oauth2/v2.0/token \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "client_id=YOUR_CLIENT_ID" \
  -d "refresh_token=YOUR_REFRESH_TOKEN" \
  -d "grant_type=refresh_token" \
  -d "scope=https://outlook.office.com/IMAP.AccessAsUser.All offline_access"
```

成功响应：
```json
{
  "access_token": "eyJ0eXAi...",
  "refresh_token": "0.AXoA...",
  "expires_in": 3600
}
```

失败响应：
```json
{
  "error": "invalid_scope",
  "error_description": "AADSTS70011: The provided value..."
}
```

## 两种模式对比

| 特性 | Graph API | IMAP |
|------|-----------|------|
| Scope | `https://graph.microsoft.com/.default` | `https://outlook.office.com/IMAP.AccessAsUser.All` |
| 速度 | 快 (2-5秒) | 中等 (5-10秒) |
| 稳定性 | 高 | 中 |
| 推荐场景 | 大批量注册 | 备用方案 |
| Token 通用性 | 不通用 | 不通用 |

**注意：** 两种模式的 refresh_token 不能互换使用！

## 日志示例

### 成功的日志

```
[IMAP] Getting access token for client_id: 9e5f94bc-...
[IMAP] Requesting access token from Microsoft OAuth2 endpoint
[IMAP] OAuth2 response status: 200 OK
[IMAP] Successfully obtained access token
[IMAP] Connecting to outlook.office365.com:993
[IMAP] TCP connection established, starting TLS handshake
[IMAP] TLS connection established, creating IMAP client
[IMAP] Authenticating with XOAUTH2 for user: user@outlook.com
[IMAP] Authentication successful
[IMAP] INBOX selected successfully
[IMAP] Found 3 messages
[IMAP] ✓ Found verification code: 123456
```

### 失败的日志（Scope 错误）

```
[IMAP] Getting access token for client_id: 9e5f94bc-...
[IMAP] Requesting access token from Microsoft OAuth2 endpoint
[IMAP] OAuth2 response status: 400 Bad Request
[IMAP] Failed to get access token: {"error":"invalid_scope",...}
```

## 支持

如果遇到问题：
1. 检查日志输出
2. 确认 scope 是否正确
3. 验证 Azure AD 权限配置
4. 重新生成 refresh_token
5. 查看完整文档：`IMAP_SETUP_GUIDE.md`
