# Kiro IDE 自动登录流程说明

## 当前实现的触发流程

### 流程图

```
用户点击"开始注册"
    ↓
执行自动化注册流程
    ↓
注册成功 ✓
    ↓
显示成功提示
    ↓
弹出确认对话框：
"注册成功！是否立即登录 Kiro IDE 并生成授权文件？"
    ↓
用户选择：
    ├─ 是 → 打开 Kiro 授权对话框
    │         ↓
    │      用户选择登录方式：
    │      - 真实 OAuth 登录（推荐）
    │        - Google
    │        - GitHub
    │        - AWS Builder ID
    │      - 模拟登录（测试用）
    │         ↓
    │      点击"开始登录"
    │         ↓
    │      浏览器自动打开
    │         ↓
    │      用户完成授权
    │         ↓
    │      自动返回
    │         ↓
    │      授权文件已保存 ✓
    │
    └─ 否 → 跳过授权，稍后可手动操作
```

## 代码实现位置

### 1. 注册成功处理

**文件:** `src/components/AccountsTable.tsx`

**函数:** `handleStartRegistration`

```typescript
const handleStartRegistration = async (id: number) => {
  // ... 执行注册 ...
  
  const result = await api.startRegistration(id);
  await showSuccess(result);
  
  // 注册成功后询问
  const account = accounts.find(a => a.id === id);
  if (account && account.kiro_password) {
    const shouldLogin = await showConfirm(
      '注册成功！是否立即登录 Kiro IDE 并生成授权文件？\n\n这将打开浏览器完成 OAuth 授权流程。',
      'Kiro IDE 授权'
    );
    
    if (shouldLogin) {
      setSelectedAccount(account);
      setIsKiroAuthDialogOpen(true);
    }
  }
};
```

### 2. 授权对话框

**文件:** `src/components/KiroAuthDialog.tsx`

**功能:**
- 选择登录模式（真实/模拟）
- 选择授权方式（Social/IdC）
- 选择提供商（Google/GitHub）
- 点击"开始登录"执行 OAuth 流程

### 3. OAuth 登录执行

**文件:** `src/api.ts`

**API 方法:**
```typescript
kiroAuthApi.oauthLoginGoogle()    // Google 登录
kiroAuthApi.oauthLoginGithub()    // GitHub 登录
kiroAuthApi.oauthLoginBuilderId() // Builder ID 登录
```

## 触发时机

### 当前触发点

✅ **注册成功后** - 在 `handleStartRegistration` 函数中
- 位置: `src/components/AccountsTable.tsx:88-110`
- 条件: `account.kiro_password` 存在（注册成功）
- 行为: 弹出确认对话框

### 其他可能的触发点

❌ **应用启动时** - 未实现
❌ **批量注册完成后** - 未实现
❌ **手动触发** - 未实现（除了通过对话框）

## 自动化程度

### 当前实现（半自动）

```
注册 → 询问 → 用户确认 → 打开对话框 → 用户选择 → 点击按钮 → OAuth 流程
  ↑                ↑                          ↑              ↑
自动              手动                       手动           手动
```

### 可选的完全自动方案

```
注册 → 自动执行 OAuth 登录 → 完成
  ↑                ↑
自动              自动
```

**实现方式:**

```typescript
// 在 handleStartRegistration 中
if (account && account.kiro_password) {
  // 直接执行自动登录，不询问
  await autoLoginKiro(account);
}

const autoLoginKiro = async (account: Account) => {
  try {
    // 使用默认配置（Google）
    const result = await kiroAuthApi.oauthLoginGoogle();
    await showSuccess('✓ Google 登录成功！授权文件已保存');
  } catch (error) {
    await showError('自动登录失败: ' + error);
  }
};
```

## 配置选项

### 方案 1: 在设置中添加开关

**文件:** `src/models.ts` (Settings 接口)

```typescript
export interface Settings {
  // ... 现有设置 ...
  auto_kiro_login: boolean;           // 是否自动登录
  kiro_login_provider: 'google' | 'github' | 'builder_id'; // 默认提供商
}
```

**使用:**

```typescript
const settings = await api.getSettings();

if (settings.auto_kiro_login) {
  // 自动登录
  if (settings.kiro_login_provider === 'google') {
    await kiroAuthApi.oauthLoginGoogle();
  } else if (settings.kiro_login_provider === 'github') {
    await kiroAuthApi.oauthLoginGithub();
  } else {
    await kiroAuthApi.oauthLoginBuilderId();
  }
} else {
  // 询问用户
  const shouldLogin = await showConfirm(...);
}
```

### 方案 2: 每次询问用户

**当前实现** - 每次注册成功后都询问

### 方案 3: 记住用户选择

使用 localStorage 记住用户的选择：

```typescript
const lastChoice = localStorage.getItem('kiro_auto_login_choice');

if (lastChoice === 'always') {
  // 总是自动登录
  await autoLoginKiro(account);
} else if (lastChoice === 'never') {
  // 从不自动登录
  return;
} else {
  // 询问并记住选择
  const shouldLogin = await showConfirm(
    '注册成功！是否立即登录 Kiro IDE？',
    'Kiro IDE 授权',
    {
      showRememberChoice: true, // 显示"记住我的选择"复选框
    }
  );
}
```

## 批量注册的处理

### 当前实现

**文件:** `src/components/AccountsTable.tsx`

**函数:** `handleStartBatchRegistration` (未实现 Kiro 登录)

### 建议实现

```typescript
const handleStartBatchRegistration = async () => {
  // ... 执行批量注册 ...
  
  const result = await api.startBatchRegistration();
  await showSuccess(result);
  
  // 批量注册完成后，询问是否批量登录
  const shouldBatchLogin = await showConfirm(
    '批量注册完成！是否为所有成功注册的账号生成 Kiro IDE 授权？',
    'Kiro IDE 批量授权'
  );
  
  if (shouldBatchLogin) {
    // 获取所有已注册的账号
    const registeredAccounts = accounts.filter(
      a => a.status === 'registered' && a.kiro_password
    );
    
    // 逐个执行 OAuth 登录
    for (const account of registeredAccounts) {
      try {
        await kiroAuthApi.oauthLoginGoogle();
        console.log(`✓ ${account.email} 授权成功`);
      } catch (error) {
        console.error(`✗ ${account.email} 授权失败:`, error);
      }
    }
    
    await showSuccess('批量授权完成！');
  }
};
```

## 用户体验优化

### 1. 进度提示

在 OAuth 登录过程中显示进度：

```typescript
const [loginProgress, setLoginProgress] = useState('');

const handleOAuthLogin = async () => {
  setLoginProgress('正在打开浏览器...');
  
  try {
    const result = await kiroAuthApi.oauthLoginGoogle();
    setLoginProgress('授权成功！正在保存文件...');
    // ...
    setLoginProgress('完成！');
  } catch (error) {
    setLoginProgress('失败: ' + error);
  }
};
```

### 2. 超时提示

OAuth 流程有 5 分钟超时，应该提示用户：

```typescript
const handleOAuthLogin = async () => {
  const timeoutWarning = setTimeout(() => {
    showWarning('请在浏览器中完成授权，5 分钟后将自动超时');
  }, 60000); // 1 分钟后提示
  
  try {
    const result = await kiroAuthApi.oauthLoginGoogle();
    clearTimeout(timeoutWarning);
    // ...
  } catch (error) {
    clearTimeout(timeoutWarning);
    // ...
  }
};
```

### 3. 失败重试

OAuth 失败后提供重试选项：

```typescript
const handleOAuthLogin = async () => {
  try {
    const result = await kiroAuthApi.oauthLoginGoogle();
    // ...
  } catch (error) {
    const shouldRetry = await showConfirm(
      'OAuth 登录失败，是否重试？\n\n错误: ' + error,
      '登录失败'
    );
    
    if (shouldRetry) {
      await handleOAuthLogin(); // 递归重试
    }
  }
};
```

## 总结

### 当前触发流程

1. ✅ 用户点击"开始注册"
2. ✅ 注册成功
3. ✅ 弹出确认对话框
4. ⚠️ 用户需要确认
5. ⚠️ 打开授权对话框
6. ⚠️ 用户选择登录方式
7. ⚠️ 用户点击"开始登录"
8. ✅ 自动执行 OAuth 流程

### 建议改进

**选项 A: 完全自动（最简单）**
```typescript
// 注册成功后直接执行，不询问
await kiroAuthApi.oauthLoginGoogle();
```

**选项 B: 询问一次（当前实现）**
```typescript
// 注册成功后询问，用户确认后打开对话框
const shouldLogin = await showConfirm(...);
if (shouldLogin) {
  setIsKiroAuthDialogOpen(true);
}
```

**选项 C: 配置化（最灵活）**
```typescript
// 在设置中配置，根据配置决定行为
if (settings.auto_kiro_login) {
  await autoLogin();
} else {
  await showDialog();
}
```

### 推荐方案

**对于单个注册:** 使用选项 B（当前实现）- 询问用户
**对于批量注册:** 使用选项 A - 自动执行（避免多次询问）

---

**当前状态:** ✅ 已实现选项 B（询问用户）

**触发位置:** `src/components/AccountsTable.tsx:88-110`
