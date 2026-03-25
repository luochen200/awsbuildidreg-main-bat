# Kiro IDE 桌面授权功能说明

## 功能概述

本功能允许在账号注册成功后，自动生成 Kiro IDE 桌面授权文件，实现快速登录 Kiro IDE。

## 核心功能

### 1. 授权方式

支持两种 Kiro IDE 授权方式：

#### Social 登录 (Google/GitHub/Apple)

- 使用 OAuth 2.0 PKCE 流程
- 生成 `accessToken` 和 `refreshToken`
- 包含 `profileArn` 信息
- 适用于社交账号登录

#### IdC 登录 (AWS Builder ID)

- 使用 AWS SSO OIDC 设备授权流程
- 生成客户端注册信息
- 包含 `clientId`、`clientSecret` 和 `clientIdHash`
- 适用于 AWS Builder ID 账号

### 2. 生成的文件

#### Token 文件

位置: `~/.aws/sso/cache/kiro-auth-token.json`

Social 格式:

```json
{
  "accessToken": "...",
  "refreshToken": "...",
  "expiresAt": "2024-01-18T12:00:00Z",
  "authMethod": "social",
  "provider": "Google",
  "profileArn": "arn:aws:codewhisperer:us-east-1:699475941385:profile/EHGA3GRVQMUK"
}
```

IdC 格式:

```json
{
  "accessToken": "...",
  "refreshToken": "...",
  "expiresAt": "2024-01-18T12:00:00Z",
  "authMethod": "IdC",
  "provider": "BuilderId",
  "clientIdHash": "abc123...",
  "region": "us-east-1"
}
```

#### 客户端注册文件 (仅 IdC)

位置: `~/.aws/sso/cache/{clientIdHash}.json`

```json
{
  "clientId": "...",
  "clientSecret": "...",
  "expiresAt": "2024-04-18T12:00:00Z"
}
```

## 使用流程

### 1. 注册账号

- 导入或添加账号信息
- 点击"开始注册"按钮
- 等待自动化注册完成

### 2. 生成授权

注册成功后会自动弹出"Kiro IDE 桌面授权"对话框：

1. 选择授权方式（Social 或 IdC）
2. 点击"生成授权文件"按钮
3. 等待生成完成

### 3. 使用授权

- 授权文件生成后，直接启动 Kiro IDE
- IDE 会自动读取授权文件并登录
- 无需手动输入账号密码

## API 接口

### Rust 命令

#### `generate_kiro_social_auth`

生成 Social 授权文件

```rust
#[tauri::command]
pub async fn generate_kiro_social_auth(
    access_token: String,
    refresh_token: String,
    provider: String,
    profile_arn: Option<String>,
) -> Result<String, String>
```

#### `generate_kiro_idc_auth`

生成 IdC 授权文件

```rust
#[tauri::command]
pub async fn generate_kiro_idc_auth(
    access_token: String,
    refresh_token: String,
    provider: String,
    client_id: String,
    client_secret: String,
    client_id_hash: String,
    region: Option<String>,
) -> Result<String, String>
```

#### `read_kiro_auth_token`

读取当前授权信息

```rust
#[tauri::command]
pub async fn read_kiro_auth_token() -> Result<KiroAuthToken, String>
```

#### `simulate_kiro_login`

模拟登录（用于测试）

```rust
#[tauri::command]
pub async fn simulate_kiro_login(
    email: String,
    kiro_password: String,
    auth_method: String,
) -> Result<String, String>
```

### TypeScript API

```typescript
import { kiroAuthApi } from "./api";

// 生成 Social 授权
await kiroAuthApi.generateSocialAuth(
  accessToken,
  refreshToken,
  "Google",
  profileArn,
);

// 生成 IdC 授权
await kiroAuthApi.generateIdcAuth(
  accessToken,
  refreshToken,
  "BuilderId",
  clientId,
  clientSecret,
  clientIdHash,
  "us-east-1",
);

// 读取当前授权
const token = await kiroAuthApi.readAuthToken();

// 模拟登录
await kiroAuthApi.simulateLogin(email, password, "social");
```

## 技术实现

### 核心模块

#### `src-tauri/src/kiro_auth.rs`

- 授权文件的读写操作
- Token 结构定义
- 文件路径管理
- 原子写入保证数据安全

#### `src-tauri/src/commands.rs`

- Tauri 命令定义
- 业务逻辑处理
- 错误处理

#### `src/components/KiroAuthDialog.tsx`

- 授权对话框 UI
- 用户交互处理
- 状态管理

### 安全特性

1. **原子写入**: 先写临时文件，再重命名，避免数据损坏
2. **目录自动创建**: 确保必要的目录存在
3. **错误处理**: 完善的错误提示和日志
4. **Token 过期**: 自动设置合理的过期时间

## 参考项目

本功能参考了 `kiro-account-manager` 项目的实现：

- AWS SSO OIDC 流程
- Kiro IDE 授权机制
- 文件结构和格式
- 最佳实践

## 注意事项

1. **文件权限**: 确保有权限写入 `~/.aws/sso/cache/` 目录
2. **IDE 版本**: 不同版本的 Kiro IDE 可能有不同的授权格式
3. **Token 刷新**: Token 过期后需要重新生成
4. **多账号**: 每次生成会覆盖之前的授权文件

## 未来扩展

- [ ] 支持真实的 OAuth 流程（目前是模拟）
- [ ] 支持 Token 自动刷新
- [ ] 支持多账号管理
- [ ] 支持导出/导入授权文件
- [ ] 集成浏览器自动化完成真实登录
