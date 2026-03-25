# 如何获取 AWS SSO 认证凭据

## 快速开始

### 方法1：使用检查脚本（推荐）

#### Windows 用户
```powershell
# 在 PowerShell 中运行
.\check_aws_sso_cache.ps1
```

#### Linux/Mac 用户
```bash
# 添加执行权限
chmod +x check_aws_sso_cache.sh

# 运行脚本
./check_aws_sso_cache.sh
```

这个脚本会自动：
- ✅ 查找所有 AWS SSO 缓存文件
- ✅ 显示 `refreshToken`、`clientId`、`clientSecret` 等关键字段
- ✅ 检查 token 是否过期
- ✅ 显示完整的 JSON 内容

---

## 方法2：手动查找

### 步骤1：执行 AWS SSO 登录

如果还没有登录，先执行：

```bash
# 如果已经配置了 SSO profile
aws sso login --profile your-profile

# 如果还没有配置，先配置
aws configure sso
```

### 步骤2：查找缓存文件

#### Windows
```powershell
# 查看所有缓存文件
Get-ChildItem $env:USERPROFILE\.aws\sso\cache\*.json

# 查看文件内容
Get-Content $env:USERPROFILE\.aws\sso\cache\*.json | ConvertFrom-Json | ConvertTo-Json -Depth 10
```

#### Linux/Mac
```bash
# 查看所有缓存文件
ls -la ~/.aws/sso/cache/*.json

# 查看文件内容（需要安装 jq）
cat ~/.aws/sso/cache/*.json | jq .

# 或者不用 jq
cat ~/.aws/sso/cache/*.json
```

### 步骤3：提取关键信息

在 JSON 文件中查找以下字段：

```json
{
  "accessToken": "eyJraWQiOiJ...",           // 访问令牌
  "refreshToken": "Atzr|IwEBIA...",          // ⭐ 刷新令牌
  "clientId": "xxxxxxxxxxxxxxxx",            // ⭐ 客户端ID
  "clientSecret": "xxxxxxxxxxxxxxxx",        // ⭐ 客户端密钥
  "expiresAt": "2024-01-17T12:00:00Z",      // 过期时间
  "region": "us-east-1",                     // 区域
  "startUrl": "https://xxx.awsapps.com/start" // SSO 起始URL
}
```

**注意**：不是所有文件都包含所有字段，可能需要查看多个文件。

---

## 方法3：从浏览器开发者工具获取

### 步骤1：打开浏览器开发者工具

1. 打开 Chrome/Edge 浏览器
2. 按 `F12` 打开开发者工具
3. 切换到 **Network（网络）** 标签
4. 勾选 **Preserve log（保留日志）**

### 步骤2：执行 SSO 登录

```bash
aws sso login --profile your-profile
```

浏览器会自动打开 SSO 登录页面。

### 步骤3：监控网络请求

在开发者工具的 Network 标签中，查找以下请求：

1. **OIDC Token 请求**
   - URL 包含：`/token` 或 `/oauth2/token`
   - 方法：POST
   - 查看 **Response（响应）** 标签

2. **SSO 认证请求**
   - URL 包含：`awsapps.com` 或 `amazonaws.com`
   - 查看响应中的 JSON 数据

### 步骤4：提取数据

在响应中查找：
```json
{
  "access_token": "...",
  "refresh_token": "...",
  "client_id": "...",
  "client_secret": "...",
  "token_type": "Bearer",
  "expires_in": 3600
}
```

---

## 方法4：使用我们的应用监控（已实现）

运行我们修改后的应用，它会在控制台打印每个环节的 URL 和 Cookies：

```bash
# 开发模式
npm run tauri dev

# 或者编译后运行
npm run tauri build
```

查看控制台输出，寻找包含这些字段的环节。

---

## 常见问题

### Q1: 找不到缓存文件？
**A**: 确保已经执行过 `aws sso login`。如果还没有配置，先运行 `aws configure sso`。

### Q2: 缓存文件中没有 `refreshToken` 或 `clientSecret`？
**A**: 这些字段可能在不同的文件中，或者使用了不同的字段名。尝试：
- 查看所有 `.json` 文件
- 查找类似的字段名：`refresh_token`、`client_id`、`client_secret`
- 检查 `~/.aws/cli/cache/` 目录

### Q3: Token 已过期怎么办？
**A**: 重新执行 `aws sso login` 生成新的 token。

### Q4: 不同的 AWS 账户有不同的凭据吗？
**A**: 是的，每个 SSO profile 可能有不同的缓存文件。使用 `--profile` 参数指定。

---

## 安全提示

⚠️ **这些是敏感信息！**

- 不要分享 `accessToken`、`refreshToken`、`clientSecret`
- 不要提交到 Git 仓库
- 定期轮换凭据
- 使用后及时清理缓存（如果需要）

---

## 下一步

获取到这些信息后：

1. **测试凭据是否有效**
   ```bash
   # 使用 AWS CLI 测试
   aws sts get-caller-identity --profile your-profile
   ```

2. **在应用中使用**
   - 将这些凭据填入应用的导入文件
   - 格式：`email----password----clientId----refreshToken`

3. **如果需要自动化**
   - 可以编写代码自动读取缓存文件
   - 或者使用 AWS SDK 获取凭据

---

## 需要帮助？

如果以上方法都不行，可以：

1. 运行检查脚本，把输出发给我（记得隐藏敏感信息）
2. 告诉我你的 AWS SSO 配置方式
3. 我可以帮你定制获取方案
