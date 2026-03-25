# Token 测试指南

## 问题诊断

当前 IMAP 认证失败，可能的原因：

### 1. Refresh Token 的 Scope 不匹配

**检查方法：**
- 如果 Graph API 模式可以工作，说明 token 本身是有效的
- 但 IMAP 需要特定的 scope

**解决方案：**
需要使用正确的 scope 重新生成 refresh_token：
```
https://outlook.office.com/IMAP.AccessAsUser.All offline_access
```

### 2. 测试步骤

#### 步骤 1: 测试 Graph API 模式

1. 在系统设置中选择 "Microsoft Graph API" 模式
2. 尝试注册一个账号
3. 如果成功，说明 refresh_token 本身是有效的，但 scope 不对

#### 步骤 2: 检查 Token Scope

使用 curl 测试 token：

```bash
# 测试当前 token 的 scope
curl -X POST https://login.microsoftonline.com/common/oauth2/v2.0/token \
  -d "client_id=YOUR_CLIENT_ID" \
  -d "refresh_token=YOUR_REFRESH_TOKEN" \
  -d "grant_type=refresh_token" \
  -d "scope=https://outlook.office.com/IMAP.AccessAsUser.All offline_access"
```

**成功响应示例：**
```json
{
  "token_type": "Bearer",
  "scope": "https://outlook.office.com/IMAP.AccessAsUser.All",
  "expires_in": 3600,
  "access_token": "eyJ0eXAi...",
  "refresh_token": "0.AXoA..."
}
```

**失败响应示例：**
```json
{
  "error": "invalid_grant",
  "error_description": "AADSTS65001: The user or administrator has not consented..."
}
```

#### 步骤 3: 重新生成 Refresh Token

如果上面的测试失败，需要重新生成 token：

1. **构造授权 URL：**
```
https://login.microsoftonline.com/common/oauth2/v2.0/authorize?client_id=YOUR_CLIENT_ID&response_type=code&redirect_uri=YOUR_REDIRECT_URI&scope=https://outlook.office.com/IMAP.AccessAsUser.All%20offline_access&prompt=consent
```

注意：
- 添加 `&prompt=consent` 强制重新授权
- scope 必须是 `https://outlook.office.com/IMAP.AccessAsUser.All offline_access`

2. **在浏览器中打开 URL**
3. **登录并授权**
4. **获取 authorization code**（从回调 URL 中）
5. **用 code 换取 token：**

```bash
curl -X POST https://login.microsoftonline.com/common/oauth2/v2.0/token \
  -d "client_id=YOUR_CLIENT_ID" \
  -d "code=AUTHORIZATION_CODE" \
  -d "redirect_uri=YOUR_REDIRECT_URI" \
  -d "grant_type=authorization_code"
```

6. **保存新的 refresh_token**

### 3. 验证 Azure AD 权限

确保在 Azure Portal 中配置了正确的权限：

1. 登录 [Azure Portal](https://portal.azure.com)
2. 进入 "Azure Active Directory" → "应用注册"
3. 选择你的应用
4. 点击 "API 权限"
5. 确认有以下权限：
   - **Office 365 Exchange Online**
     - `IMAP.AccessAsUser.All` (委托的权限)
6. 点击 "授予管理员同意"

### 4. 常见错误

#### ConnectionLost

**原因：** 认证失败，服务器断开连接

**可能的问题：**
1. Access token 无效或过期
2. Access token 的 scope 不包含 IMAP 权限
3. 邮箱账号没有启用 IMAP

**解决方案：**
1. 确认 token 是用正确的 scope 生成的
2. 检查 Azure AD 权限配置
3. 确认邮箱启用了 IMAP（Outlook.com 默认启用）

#### invalid_grant

**原因：** Refresh token 无效或已过期

**解决方案：**
重新生成 refresh_token

#### invalid_scope

**原因：** Scope 不正确

**解决方案：**
使用正确的 scope：`https://outlook.office.com/IMAP.AccessAsUser.All offline_access`

### 5. 调试技巧

#### 查看详细日志

日志会显示：
```
[IMAP] Getting access token for client_id: xxx
[IMAP] Refresh token length: 123 chars
[IMAP] Refresh token prefix: 0.AXoA...
[IMAP] OAuth2 response status: 200 OK
[IMAP] Successfully obtained access token (length: 1234)
[IMAP] Access token prefix: eyJ0eXAi...
[IMAP] Connecting to outlook.office365.com:993
[IMAP] TCP connection established, starting TLS handshake
[IMAP] TLS connection established, creating IMAP client
[IMAP] Authenticating with XOAUTH2 for user: user@outlook.com
[IMAP] Token length: 1234 chars
[IMAP] Authentication failed: ConnectionLost
```

#### 手动测试 IMAP 连接

使用 Python 脚本测试（参考 imap_client.py）：

```python
import imaplib
import base64

email = "your@outlook.com"
access_token = "YOUR_ACCESS_TOKEN"

# 构造认证字符串
auth_string = f"user={email}\x01auth=Bearer {access_token}\x01\x01"
auth_base64 = base64.b64encode(auth_string.encode("ascii"))

# 连接
mail = imaplib.IMAP4_SSL("outlook.office365.com", 993)

# 认证
tag = mail._new_tag().decode('ascii')
command = f'{tag} AUTHENTICATE XOAUTH2 {auth_base64.decode("ascii")}\r\n'
mail.send(command.encode('ascii'))

# 查看响应
while True:
    line = mail.readline()
    print(line)
    if line.startswith(tag.encode('ascii')):
        break
```

### 6. 对比测试

| 测试项 | Graph API | IMAP |
|--------|-----------|------|
| Token 获取 | ✅ 成功 | ✅ 成功 |
| 邮件获取 | ✅ 成功 | ❌ 认证失败 |

如果 Graph API 成功但 IMAP 失败，说明：
- Refresh token 本身有效
- 但 scope 不包含 IMAP 权限
- 需要重新生成 token

### 7. 快速解决方案

**最简单的方法：**

1. 暂时使用 Graph API 模式（如果可用）
2. 重新生成一个专门用于 IMAP 的 refresh_token
3. 使用新的 token 测试 IMAP 模式

**生成 IMAP Token 的完整命令：**

```bash
# 1. 获取 authorization code（在浏览器中打开）
https://login.microsoftonline.com/common/oauth2/v2.0/authorize?client_id=YOUR_CLIENT_ID&response_type=code&redirect_uri=http://localhost&scope=https://outlook.office.com/IMAP.AccessAsUser.All%20offline_access&prompt=consent

# 2. 用 code 换取 token
curl -X POST https://login.microsoftonline.com/common/oauth2/v2.0/token \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "client_id=YOUR_CLIENT_ID" \
  -d "code=AUTHORIZATION_CODE_FROM_STEP_1" \
  -d "redirect_uri=http://localhost" \
  -d "grant_type=authorization_code"

# 3. 保存响应中的 refresh_token
```

### 8. 验证新 Token

```bash
# 测试新 token 是否可以获取 IMAP access token
curl -X POST https://login.microsoftonline.com/common/oauth2/v2.0/token \
  -d "client_id=YOUR_CLIENT_ID" \
  -d "refresh_token=NEW_REFRESH_TOKEN" \
  -d "grant_type=refresh_token" \
  -d "scope=https://outlook.office.com/IMAP.AccessAsUser.All offline_access"
```

如果成功，响应中的 `scope` 字段应该包含 `IMAP.AccessAsUser.All`。
