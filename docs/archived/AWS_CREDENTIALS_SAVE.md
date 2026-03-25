# AWS Builder ID 凭证保存功能

## 概述

系统现在支持在注册成功后自动保存 AWS Builder ID 的 OAuth 凭证。

## 数据库结构更新

### 账户表新增字段

原有字段（Outlook 邮箱凭证）：
- `outlook_client_id` - Outlook 的 Client ID（用于接收验证码邮件）
- `outlook_refresh_token` - Outlook 的 Refresh Token（用于接收验证码邮件）

新增字段（AWS Builder ID 凭证）：
- `aws_client_id` - AWS Builder ID 的 Client ID（如果可获取）
- `aws_client_secret` - AWS Builder ID 的 Client Secret（如果可获取）
- `aws_refresh_token` - AWS Builder ID 的 Refresh Token

## 工作原理

1. **注册前**：用户提供 Outlook 邮箱凭证（outlook_client_id 和 outlook_refresh_token），用于接收验证码邮件

2. **注册过程中**：
   - 使用 Outlook 凭证通过 Graph API 或 IMAP 获取验证码
   - 完成 AWS Builder ID 注册流程
   - **在点击最后的"Continue"按钮前**，注入 JavaScript 拦截器到页面中

3. **拦截机制**：
   - 拦截器会监听所有的 `fetch` 和 `XMLHttpRequest` 请求
   - 当检测到对 `https://identity.api.aws.amazon.com/oauth2/token` 的请求时
   - 自动捕获响应数据并保存到 `window.__awsTokenData` 变量中

4. **注册成功后**：
   - 等待成功页面出现
   - 读取 `window.__awsTokenData` 获取拦截到的 OAuth 凭证
   - 提取 `refresh_token`（主要凭证）
   - 将凭证保存到数据库的 `aws_*` 字段中

## OAuth Token 响应格式

AWS Builder ID 的 token API 返回：
```json
{
  "access_token": "...",
  "id_token": "...",
  "refresh_token": "...",
  "expires_in": 3600
}
```

系统会保存 `refresh_token`，这是最重要的凭证，可用于后续刷新 access_token。

## 数据迁移

系统会自动迁移旧数据库：
- 将旧的 `client_id` 字段重命名为 `outlook_client_id`
- 将旧的 `refresh_token` 字段重命名为 `outlook_refresh_token`
- 添加新的 `aws_client_id`、`aws_client_secret`、`aws_refresh_token` 字段

## 导入导出格式

导入格式保持不变：
```
邮箱----密码----Outlook_Client_ID----Outlook_Refresh_Token
```

导出格式也保持不变，导出的是 Outlook 凭证（用于后续导入）。

## 使用场景

保存的 AWS Builder ID 凭证可用于：
1. 后续使用 AWS Builder ID 登录其他服务
2. 通过 AWS SDK 访问 AWS 服务
3. 刷新 access token 以保持登录状态
4. 使用 AWS CLI 进行开发

## 技术细节

### 拦截时机
拦截器在密码输入页面点击"Continue"按钮**之前**注入，确保能够捕获后续的 OAuth token 请求。

### 拦截方法
使用 JavaScript 重写 `window.fetch` 和 `XMLHttpRequest.prototype` 方法，在不影响正常请求的情况下捕获响应数据。

### 数据提取
通过 `tab.evaluate()` 执行 JavaScript 代码读取 `window.__awsTokenData`，然后反序列化为 Rust 结构体。

## 注意事项

1. AWS Builder ID 的凭证是在注册成功后自动获取的，无需手动输入
2. 如果拦截失败（网络问题、页面结构变化等），`aws_*` 字段将保持为 NULL，不影响注册流程
3. Outlook 凭证和 AWS 凭证是两套独立的凭证系统，互不影响
4. `aws_client_id` 和 `aws_client_secret` 通常不会在 token 响应中返回，所以这两个字段可能为 NULL
5. 最重要的是 `aws_refresh_token`，有了它就可以刷新获取新的 access_token
