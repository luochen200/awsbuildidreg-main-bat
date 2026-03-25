# Kiro OAuth 登录触发流程总结

## 📍 触发位置

### 主要触发点：注册成功后

**文件:** `src/components/AccountsTable.tsx`  
**函数:** `handleStartRegistration` (第 88-110 行)  
**触发条件:** 账号注册成功且 `kiro_password` 存在

## 🔄 完整流程

### 1. 用户操作

```
用户点击账号列表中的"播放"按钮（开始注册）
```

### 2. 注册流程

```typescript
// src/components/AccountsTable.tsx
const handleStartRegistration = async (id: number) => {
  setProcessingId(id);

  // 执行自动化注册
  const result = await api.startRegistration(id);

  // 刷新账号列表
  onRefresh();

  // 显示成功提示
  await showSuccess(result);

  // ⭐ 关键：注册成功后的处理
  const account = accounts.find((a) => a.id === id);
  if (account && account.kiro_password) {
    // 询问用户是否立即登录
    const shouldLogin = await showConfirm(
      "注册成功！是否立即登录 Kiro IDE 并生成授权文件？\n\n这将打开浏览器完成 OAuth 授权流程。",
      "Kiro IDE 授权",
    );

    if (shouldLogin) {
      // 打开授权对话框
      setSelectedAccount(account);
      setIsKiroAuthDialogOpen(true);
    }
  }
};
```

### 3. 授权对话框

```typescript
// src/components/KiroAuthDialog.tsx
<KiroAuthDialog
  isOpen={isKiroAuthDialogOpen}
  onClose={() => setIsKiroAuthDialogOpen(false)}
  email={selectedAccount.email}
  kiroPassword={selectedAccount.kiro_password}
  autoStart={false}  // 可以设置为 true 自动开始
/>
```

### 4. 用户选择

```
对话框中用户可以选择：
1. 登录模式：真实 OAuth / 模拟登录
2. 授权方式：Social (Google/GitHub) / IdC (Builder ID)
3. 点击"开始登录"按钮
```

### 5. OAuth 执行

```typescript
// src/components/KiroAuthDialog.tsx
const handleGenerateAuth = async () => {
  if (authMode === "real") {
    if (authMethod === "social") {
      if (socialProvider === "google") {
        await kiroAuthApi.oauthLoginGoogle();
      } else {
        await kiroAuthApi.oauthLoginGithub();
      }
    } else {
      await kiroAuthApi.oauthLoginBuilderId();
    }
  }
};
```

### 6. 后端处理

```rust
// src-tauri/src/commands.rs
#[tauri::command]
pub async fn kiro_oauth_login_google() -> Result<KiroOAuthResult, String> {
    kiro_oauth::perform_kiro_social_login("Google").await
}
```

### 7. OAuth 流程

```rust
// src-tauri/src/kiro_oauth.rs
pub async fn perform_kiro_social_login(provider: &str) -> Result<KiroOAuthResult, String> {
    // 1. 生成 PKCE 参数
    let code_verifier = generate_code_verifier();
    let code_challenge = generate_code_challenge(&code_verifier);
    let state = uuid::Uuid::new_v4().to_string();

    // 2. 注册回调等待器
    let waiter = register_oauth_waiter(&state);

    // 3. 打开浏览器
    open_browser_for_oauth(provider, redirect_uri, &code_challenge, &state).await?;

    // 4. 等待回调
    let callback = waiter.wait_for_callback()?;

    // 5. 交换 token
    let token = exchange_code_for_token(&callback.code, &code_verifier, redirect_uri).await?;

    // 6. 生成授权文件
    kiro_auth::generate_kiro_social_auth(params)?;

    Ok(result)
}
```

## 📊 流程图

```
┌─────────────────────────────────────────────────────────────┐
│                     用户点击"开始注册"                        │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│              执行自动化注册流程（浏览器自动化）                │
│  - 打开 app.kiro.dev/signin                                 │
│  - 点击 Google 登录                                          │
│  - 输入邮箱                                                  │
│  - 输入姓名                                                  │
│  - 获取验证码                                                │
│  - 设置密码                                                  │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│                    ✓ 注册成功                                │
│              kiro_password 已保存到数据库                     │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│              弹出确认对话框（showConfirm）                    │
│  "注册成功！是否立即登录 Kiro IDE 并生成授权文件？"           │
│                                                              │
│              [是]              [否]                          │
└────────────┬────────────────────┬───────────────────────────┘
             │                    │
             │                    └──> 跳过，稍后可手动操作
             │
             ▼
┌─────────────────────────────────────────────────────────────┐
│              打开 Kiro 授权对话框                             │
│  ┌───────────────────────────────────────────────────────┐  │
│  │ 登录模式：                                             │  │
│  │  ○ 真实 OAuth 登录（推荐）                             │  │
│  │  ○ 模拟登录（测试用）                                  │  │
│  │                                                        │  │
│  │ 授权方式：                                             │  │
│  │  ○ Social 登录 (Google/GitHub)                        │  │
│  │  ○ IdC 登录 (AWS Builder ID)                          │  │
│  │                                                        │  │
│  │ 选择提供商：                                           │  │
│  │  ○ Google                                              │  │
│  │  ○ GitHub                                              │  │
│  │                                                        │  │
│  │              [开始登录]  [查看当前授权]                 │  │
│  └───────────────────────────────────────────────────────┘  │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│              用户点击"开始登录"按钮                           │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│              调用后端 Tauri 命令                              │
│  kiro_oauth_login_google() / kiro_oauth_login_github()      │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│              执行 OAuth 流程（Rust 后端）                     │
│  1. 生成 PKCE 参数                                           │
│  2. 注册回调等待器                                           │
│  3. 打开浏览器到 Kiro Auth Service                           │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│              浏览器自动打开                                   │
│  https://prod.us-east-1.auth.desktop.kiro.dev/login         │
│  ?idp=Google&redirect_uri=kiro://...&code_challenge=...     │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│              用户在浏览器中完成登录授权                        │
│  - 选择 Google 账号                                          │
│  - 授权 Kiro IDE 访问                                        │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│              浏览器回调到应用                                 │
│  kiro://kiro.kiroAgent/authenticate-success?code=xxx&state=xxx│
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│              应用捕获回调 URL                                 │
│  - 验证 state 参数                                           │
│  - 提取 authorization code                                   │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│              交换 code 为 token                               │
│  POST /oauth/token                                           │
│  - code                                                      │
│  - code_verifier                                             │
│  - redirect_uri                                              │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│              获取 access_token 和 refresh_token               │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│              生成授权文件                                     │
│  ~/.aws/sso/cache/kiro-auth-token.json                      │
│  {                                                           │
│    "accessToken": "...",                                     │
│    "refreshToken": "...",                                    │
│    "expiresAt": "...",                                       │
│    "authMethod": "social",                                   │
│    "provider": "Google",                                     │
│    "profileArn": "..."                                       │
│  }                                                           │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│              ✓ 完成！                                         │
│  - 显示成功消息                                              │
│  - 授权文件已保存                                            │
│  - 可以启动 Kiro IDE                                         │
└─────────────────────────────────────────────────────────────┘
```

## 🎯 关键代码位置

### 1. 触发入口

```typescript
// src/components/AccountsTable.tsx:88-110
const handleStartRegistration = async (id: number) => {
  // ... 注册逻辑 ...

  // ⭐ 这里是触发点
  if (account && account.kiro_password) {
    const shouldLogin = await showConfirm(...);
    if (shouldLogin) {
      setIsKiroAuthDialogOpen(true);
    }
  }
};
```

### 2. 对话框组件

```typescript
// src/components/KiroAuthDialog.tsx
export function KiroAuthDialog({
  isOpen,
  onClose,
  email,
  kiroPassword,
  autoStart,
}) {
  // autoStart 参数可以控制是否自动开始
  useEffect(() => {
    if (isOpen && autoStart) {
      handleGenerateAuth();
    }
  }, [isOpen, autoStart]);
}
```

### 3. OAuth 执行

```typescript
// src/api.ts
export const kiroAuthApi = {
  async oauthLoginGoogle(): Promise<KiroOAuthResult> {
    return invoke("kiro_oauth_login_google");
  },
};
```

### 4. 后端实现

```rust
// src-tauri/src/commands.rs
#[tauri::command]
pub async fn kiro_oauth_login_google() -> Result<KiroOAuthResult, String> {
    kiro_oauth::perform_kiro_social_login("Google").await
}
```

### 5. OAuth 核心逻辑

```rust
// src-tauri/src/kiro_oauth.rs
pub async fn perform_kiro_social_login(provider: &str) -> Result<KiroOAuthResult, String> {
    // PKCE + Deep Link + Token Exchange + File Generation
}
```

## 🔧 自定义触发方式

### 方式 1: 自动开始（无需用户点击）

```typescript
// 在 AccountsTable.tsx 中
if (shouldLogin) {
  setSelectedAccount(account);
  setIsKiroAuthDialogOpen(true);
  // 传入 autoStart=true
}

// 在 KiroAuthDialog 中
<KiroAuthDialog
  autoStart={true}  // ⭐ 自动开始
/>
```

### 方式 2: 完全自动（不显示对话框）

```typescript
// 在 AccountsTable.tsx 中
if (account && account.kiro_password) {
  // 直接调用 API，不显示对话框
  try {
    const result = await kiroAuthApi.oauthLoginGoogle();
    await showSuccess("✓ Google 登录成功！");
  } catch (error) {
    await showError("登录失败: " + error);
  }
}
```

### 方式 3: 批量自动登录

```typescript
// 批量注册完成后
const registeredAccounts = accounts.filter(
  (a) => a.status === "registered" && a.kiro_password,
);

for (const account of registeredAccounts) {
  await kiroAuthApi.oauthLoginGoogle();
}
```

## 📝 总结

### 当前实现

✅ **触发时机:** 注册成功后  
✅ **触发方式:** 询问用户确认  
✅ **用户交互:** 需要选择登录方式并点击按钮  
✅ **自动化程度:** 半自动（需要用户确认和选择）

### 可选改进

1. **完全自动:** 注册成功后直接执行 OAuth，不询问
2. **记住选择:** 使用 localStorage 记住用户的选择
3. **批量处理:** 批量注册后批量生成授权
4. **配置化:** 在设置中添加自动登录开关

---

**当前触发位置:** `src/components/AccountsTable.tsx:88-110`  
**状态:** ✅ 已实现并可用
