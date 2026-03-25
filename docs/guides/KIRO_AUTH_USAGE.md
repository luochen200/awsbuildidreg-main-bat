# Kiro IDE 桌面授权 - 快速使用指南

## 快速开始

### 1. 注册账号并生成授权

1. **导入账号**
   - 在主界面点击"导入账号"
   - 选择包含账号信息的 txt 文件
   - 格式: `email----password----client_id----refresh_token`

2. **开始注册**
   - 在账号列表中找到"未注册"状态的账号
   - 点击"播放"按钮开始注册
   - 等待自动化流程完成

3. **生成 Kiro 授权**
   - 注册成功后会自动弹出"Kiro IDE 桌面授权"对话框
   - 选择授权方式：
     - **Social 登录**: 适用于 Google/GitHub/Apple 账号
     - **IdC 登录**: 适用于 AWS Builder ID
   - 点击"生成授权文件"
   - 等待生成完成

4. **启动 Kiro IDE**
   - 授权文件已自动保存到系统目录
   - 直接启动 Kiro IDE
   - IDE 会自动读取授权并登录

### 2. 查看当前授权

在授权对话框中点击"查看当前授权"按钮，授权信息会输出到浏览器控制台。

## 授权文件位置

### Windows
```
C:\Users\{用户名}\.aws\sso\cache\kiro-auth-token.json
C:\Users\{用户名}\.aws\sso\cache\{clientIdHash}.json  (仅 IdC)
```

### macOS/Linux
```
~/.aws/sso/cache/kiro-auth-token.json
~/.aws/sso/cache/{clientIdHash}.json  (仅 IdC)
```

## 授权方式对比

| 特性 | Social 登录 | IdC 登录 |
|------|------------|----------|
| 提供商 | Google/GitHub/Apple | AWS Builder ID |
| 授权流程 | OAuth 2.0 PKCE | AWS SSO OIDC |
| 文件数量 | 1 个 | 2 个 |
| 适用场景 | 社交账号 | AWS 开发者 |
| Token 有效期 | 1 小时 | 1 小时 |
| 客户端注册 | 不需要 | 需要 (90天) |

## 常见问题

### Q: 授权文件生成后 IDE 还是无法登录？
A: 请检查：
1. 授权文件是否存在于正确位置
2. 文件内容是否完整（JSON 格式）
3. Token 是否已过期
4. Kiro IDE 版本是否兼容

### Q: 可以同时使用多个账号吗？
A: 目前每次生成会覆盖之前的授权文件，如需切换账号，需要重新生成授权。

### Q: Token 过期后怎么办？
A: 重新运行注册流程，或者手动刷新 Token（需要实现 Token 刷新功能）。

### Q: 为什么是"模拟登录"？
A: 当前版本生成的是测试用的 Token，实际使用时需要集成真实的 OAuth 流程。

## 技术细节

### Token 结构

**Social Token:**
```json
{
  "accessToken": "eyJ...",
  "refreshToken": "eyJ...",
  "expiresAt": "2024-01-18T13:00:00Z",
  "authMethod": "social",
  "provider": "Google",
  "profileArn": "arn:aws:codewhisperer:us-east-1:699475941385:profile/EHGA3GRVQMUK"
}
```

**IdC Token:**
```json
{
  "accessToken": "eyJ...",
  "refreshToken": "eyJ...",
  "expiresAt": "2024-01-18T13:00:00Z",
  "authMethod": "IdC",
  "provider": "BuilderId",
  "clientIdHash": "abc123...",
  "region": "us-east-1"
}
```

### 安全性

- 使用原子写入避免文件损坏
- Token 存储在本地用户目录
- 自动设置合理的过期时间
- 支持 SHA256 哈希生成 clientIdHash

## 开发者信息

### 添加新的授权方式

1. 在 `kiro_auth.rs` 中定义新的参数结构
2. 实现生成函数
3. 在 `commands.rs` 中添加 Tauri 命令
4. 在前端添加 UI 选项

### 调试

查看 Rust 日志输出：
```bash
# 开发模式
npm run tauri dev

# 查看控制台输出
```

查看生成的文件：
```bash
# Windows
type %USERPROFILE%\.aws\sso\cache\kiro-auth-token.json

# macOS/Linux
cat ~/.aws/sso/cache/kiro-auth-token.json
```

## 参考资料

- [Kiro IDE 官方文档](https://kiro.dev)
- [AWS SSO OIDC API](https://docs.aws.amazon.com/singlesignon/latest/OIDCAPIReference/)
- [OAuth 2.0 PKCE](https://oauth.net/2/pkce/)
- 参考项目: `kiro-account-manager`
