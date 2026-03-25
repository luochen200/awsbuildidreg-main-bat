# URL 和 Cookie 监控功能

## 概述

为了追踪 AWS Builder ID 的 OAuth 凭证，我们在注册流程的各个关键步骤添加了 URL 和 Cookie 监控。

## 监控内容

### 1. URL 监控
在每个步骤打印：
- 当前页面 URL
- URL 查询参数（search params）
- URL Hash 参数

### 2. Cookie 监控
在每个步骤打印：
- 所有 Cookie
- 特别检查 `x-amz-sso_authn` cookie

### 3. 监控步骤

**步骤1**: 登录页面
- URL: https://app.kiro.dev/signin

**步骤2**: 点击 Google 登录后
- 检查是否跳转到 AWS 认证页面

**步骤3**: 输入邮箱后
- 检查 URL 参数和 Cookie

**步骤4**: 输入姓名后
- 检查 URL 参数和 Cookie

**步骤5**: 输入验证码后
- 检查 URL 参数和 Cookie

**步骤6**: 密码输入页面（注入拦截器前）
- 检查 URL 参数和 Cookie
- 特别关注 `x-amz-sso_authn` cookie

**步骤7**: 注册成功页面
- 检查最终的 URL 和 Cookie
- 尝试读取拦截的 OAuth token

## 可能的凭证位置

根据分析，AWS Builder ID 的凭证可能出现在：

1. **URL 参数中**：
   - `refreshToken`
   - `clientId`
   - `clientSecret`

2. **URL Hash 中**：
   - `#access_token=...`
   - `#refresh_token=...`

3. **Cookie 中**：
   - `x-amz-sso_authn` - AWS SSO 认证 token

4. **API 响应中**：
   - POST `https://identity.api.aws.amazon.com/oauth2/token`
   - 响应包含 `refresh_token`, `access_token`, `id_token`

## 使用方法

运行注册流程时，控制台会输出详细的日志：

```
[步骤1] 登录页面
[URL] https://app.kiro.dev/signin
[Cookies] 无
[URL检查] URL参数: {...}

[步骤2] 点击Google登录后
[URL] https://...
[Cookies] ...
[Cookie发现] ✓ 找到 x-amz-sso_authn!
[Cookie值] x-amz-sso_authn=...
```

## 下一步

根据日志输出分析：
1. 确定凭证出现在哪个步骤
2. 确定凭证的具体位置（URL/Cookie/API）
3. 调整拦截策略以正确获取凭证
