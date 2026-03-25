# 🚀 Kiro IDE 桌面授权 - 快速启动

## 一分钟上手

### 1️⃣ 启动应用

```bash
npm run tauri dev
```

### 2️⃣ 注册账号

1. 导入账号或手动添加
2. 点击"开始注册"按钮
3. 等待自动化流程完成

### 3️⃣ 生成授权

注册成功后会自动弹出对话框：

```
┌─────────────────────────────────────┐
│   Kiro IDE 桌面授权                  │
├─────────────────────────────────────┤
│ 邮箱: user@example.com              │
│ 密码: SecurePass123!                │
│                                     │
│ ○ Social 登录 (Google/GitHub)       │
│ ● IdC 登录 (AWS Builder ID)         │
│                                     │
│ [生成授权文件] [查看当前授权]        │
└─────────────────────────────────────┘
```

### 4️⃣ 启动 Kiro IDE

授权文件已自动保存，直接启动 IDE 即可登录！

## 授权文件位置

### Windows
```
C:\Users\你的用户名\.aws\sso\cache\kiro-auth-token.json
```

### macOS/Linux
```
~/.aws/sso/cache/kiro-auth-token.json
```

## 验证授权

### 方法 1: 查看文件
```bash
# Windows
type %USERPROFILE%\.aws\sso\cache\kiro-auth-token.json

# macOS/Linux
cat ~/.aws/sso/cache/kiro-auth-token.json
```

### 方法 2: 使用应用
点击对话框中的"查看当前授权"按钮，信息会输出到浏览器控制台。

## 常见问题

### ❓ 对话框没有弹出？
- 确保注册成功（状态为"已注册"）
- 检查是否有 `kiro_password` 字段
- 刷新页面重试

### ❓ 授权文件生成失败？
- 检查目录权限
- 确保 `~/.aws/sso/cache/` 目录存在
- 查看控制台错误信息

### ❓ Kiro IDE 无法登录？
- 验证授权文件格式（JSON）
- 检查 Token 是否过期
- 确认 IDE 版本兼容性

## 两种授权方式对比

| 特性 | Social | IdC |
|------|--------|-----|
| 适用账号 | Google/GitHub/Apple | AWS Builder ID |
| 文件数量 | 1 个 | 2 个 |
| 设置难度 | ⭐ 简单 | ⭐⭐ 中等 |
| 推荐场景 | 社交账号登录 | AWS 开发者 |

## 下一步

- 📖 阅读 [完整文档](./KIRO_AUTH_GUIDE.md)
- 💻 查看 [代码示例](./KIRO_AUTH_EXAMPLES.md)
- 🔧 了解 [使用指南](./KIRO_AUTH_USAGE.md)
- 📋 查看 [实现总结](./KIRO_AUTH_SUMMARY.md)

## 技术支持

遇到问题？
1. 查看文档中的"常见问题"部分
2. 检查浏览器控制台的错误信息
3. 查看 Rust 日志输出
4. 参考 `参考项目/kiro-account-manager-main`

---

**提示:** 当前版本使用模拟 Token，实际使用时需要集成真实的 OAuth 流程。
