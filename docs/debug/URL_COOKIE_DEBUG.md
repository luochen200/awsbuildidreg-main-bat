# URL和Cookie调试信息

## 修改说明

已经在代码中添加了URL和Cookie的监控功能，会在注册流程的各个关键环节打印详细信息。

## 监控的环节

程序会在以下11个关键环节打印URL、URL参数、Cookies信息：

1. **登录页面加载完成** - 初始页面加载后
2. **点击Google登录按钮后** - 点击Google登录按钮后的页面
3. **邮箱输入页面** - Google邮箱输入页面
4. **提交邮箱后** - 输入邮箱并点击继续后
5. **姓名输入页面** - Kiro姓名输入页面
6. **提交姓名后** - 输入姓名并点击继续后
7. **验证码输入页面** - 邮箱验证码输入页面
8. **提交验证码后** - 输入验证码并点击继续后
9. **密码设置页面** - Google密码设置页面
10. **提交密码后** - 设置密码并点击继续后
11. **注册成功页面** - 最终成功页面

## 监控的信息

每个环节会打印以下信息：

### 1. 当前URL
完整的浏览器地址栏URL

### 2. URL参数
URL中的查询参数（如果有），以JSON格式显示，例如：
```json
{
  "refreshToken": "xxx",
  "clientId": "xxx",
  "clientSecret": "xxx"
}
```

### 3. 所有Cookies
当前页面的所有cookies，格式如：
```
cookie1=value1; cookie2=value2; cookie3=value3
```

### 4. 重点Cookie
特别检查以下重要的cookie：
- `x-amz-sso_authn` - AWS SSO认证token
- `refreshToken` - 刷新令牌
- `clientId` - 客户端ID
- `clientSecret` - 客户端密钥

如果这些cookie存在，会单独显示它们的值。

## 输出格式示例

```
========== 1. 登录页面加载完成 ==========
URL: https://app.kiro.dev/signin
URL Parameters: {}
Cookies: session_id=abc123; user_pref=dark_mode
========================================

========== 2. 点击Google登录按钮后 ==========
URL: https://accounts.google.com/o/oauth2/auth?client_id=xxx&redirect_uri=xxx
URL Parameters: {
  "client_id": "123456789",
  "redirect_uri": "https://app.kiro.dev/callback",
  "response_type": "code"
}
Cookies: NID=xxx; SID=xxx
========================================

========== 3. 邮箱输入页面 ==========
URL: https://accounts.google.com/signin/v2/identifier
Cookies: GAPS=xxx; GALX=xxx
  x-amz-sso_authn = eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
========================================
```

## 如何使用

1. 重新编译应用：
   ```bash
   npm run tauri build
   ```
   或开发模式：
   ```bash
   npm run tauri dev
   ```

2. 运行注册流程

3. 查看控制台输出，会看到每个环节的详细URL和Cookie信息

4. 分析输出，查找：
   - URL中是否包含 `refreshToken`、`clientId`、`clientSecret` 参数
   - Cookies中是否包含 `x-amz-sso_authn` 或其他认证相关的cookie

## 注意事项

- 这些信息会打印到控制台（stdout）
- 如果在开发模式运行，可以在终端看到输出
- 如果是打包后的应用，需要从命令行启动才能看到输出
- 敏感信息（如token）会被完整打印，请注意保密

## 下一步

根据输出的信息，我们可以：
1. 确认哪个环节的URL或Cookie中包含所需的认证信息
2. 提取这些信息用于后续的API调用
3. 优化注册流程，在正确的时机获取认证凭据
