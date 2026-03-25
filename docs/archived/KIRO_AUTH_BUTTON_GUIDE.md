# Kiro IDE 授权按钮使用指南

## 📍 新增功能

在账号列表的操作列中，已注册的账号现在会显示一个 **🔑 授权按钮**。

## 🎯 按钮位置

### 表格操作列

```
序号 | 邮箱 | 密码 | 状态 | 异常原因 | 操作
----|------|------|------|---------|-----
1   | xxx  | •••• | 已注册 |        | 👁️ ✏️ 🔑 🗑️
```

**按钮说明:**
- 👁️ **查看详情** - 查看账号完整信息
- ✏️ **编辑** - 编辑账号信息
- ▶️ **开始注册** - 仅在"未注册"状态显示
- 🔑 **Kiro IDE 授权** - 仅在"已注册"状态显示（新增）
- 🗑️ **删除** - 删除账号

## 🔑 授权按钮

### 显示条件

授权按钮只在以下条件下显示：
1. ✅ 账号状态为 `registered`（已注册）
2. ✅ 账号有 `kiro_password`（注册成功生成的密码）

### 点击后的行为

```
点击授权按钮
    ↓
打开 Kiro IDE 授权对话框
    ↓
用户选择登录方式：
  - 真实 OAuth 登录（推荐）
    - Google
    - GitHub
    - AWS Builder ID
  - 模拟登录（测试用）
    ↓
点击"开始登录"
    ↓
浏览器自动打开
    ↓
用户完成授权
    ↓
授权文件自动保存
    ↓
完成！可以启动 Kiro IDE
```

## 💻 代码实现

### 1. 按钮组件

**文件:** `src/components/AccountsTable.tsx`

```typescript
{account.status === 'registered' && account.kiro_password && (
  <button
    className="action-button action-button-success"
    onClick={() => handleOpenKiroAuth(account)}
    title="Kiro IDE 授权"
  >
    <Key size={16} />
  </button>
)}
```

### 2. 点击处理

```typescript
const handleOpenKiroAuth = (account: Account) => {
  if (!account.kiro_password) {
    showError('该账号尚未注册，请先完成注册');
    return;
  }
  setSelectedAccount(account);
  setIsKiroAuthDialogOpen(true);
};
```

### 3. 对话框显示

```typescript
{isKiroAuthDialogOpen && selectedAccount && selectedAccount.kiro_password && (
  <KiroAuthDialog
    isOpen={isKiroAuthDialogOpen}
    onClose={() => {
      setIsKiroAuthDialogOpen(false);
      setSelectedAccount(null);
    }}
    email={selectedAccount.email}
    kiroPassword={selectedAccount.kiro_password}
  />
)}
```

## 🎨 样式

### 按钮样式

```css
.action-button-success {
  color: var(--success);
  border-color: var(--success);
}

.action-button-success:hover:not(:disabled) {
  background-color: var(--success-light);
}
```

**效果:**
- 默认：绿色边框和图标
- 悬停：浅绿色背景

## 📋 使用流程

### 完整流程示例

```
1. 导入账号
   ↓
2. 点击"开始注册"按钮（▶️）
   ↓
3. 等待自动化注册完成
   ↓
4. 状态变为"已注册"
   ↓
5. 授权按钮（🔑）出现
   ↓
6. 点击授权按钮
   ↓
7. 在对话框中选择登录方式
   ↓
8. 点击"开始登录"
   ↓
9. 浏览器打开，完成 OAuth
   ↓
10. 授权文件保存成功
   ↓
11. 启动 Kiro IDE，自动登录
```

## 🔍 按钮状态

### 不同状态下的按钮显示

| 账号状态 | 显示的按钮 |
|---------|-----------|
| 未注册 (not_registered) | 👁️ ✏️ ▶️ 🗑️ |
| 进行中 (in_progress) | 👁️ ✏️ 🔄 🗑️ |
| 已注册 (registered) | 👁️ ✏️ 🔑 🗑️ |
| 异常 (error) | 👁️ ✏️ 🗑️ |

**说明:**
- ▶️ = 开始注册按钮
- 🔄 = 注册中（加载动画）
- 🔑 = Kiro IDE 授权按钮

## 🎯 优势

### 相比自动弹出的优势

1. **用户控制** - 用户可以选择何时进行授权
2. **批量处理** - 可以先批量注册，再批量授权
3. **重复授权** - 可以随时重新生成授权文件
4. **清晰直观** - 按钮位置固定，容易找到
5. **无干扰** - 不会在注册后自动弹出对话框

### 使用场景

1. **单个授权** - 注册完成后，点击授权按钮
2. **批量授权** - 批量注册完成后，逐个点击授权
3. **重新授权** - Token 过期后，重新点击授权
4. **切换账号** - 为不同账号生成不同的授权

## 🔧 自定义

### 修改按钮图标

```typescript
import { Key, Lock, Shield } from 'lucide-react';

// 使用不同的图标
<Key size={16} />      // 钥匙（当前）
<Lock size={16} />     // 锁
<Shield size={16} />   // 盾牌
```

### 修改按钮颜色

```css
/* 蓝色主题 */
.action-button-success {
  color: var(--accent-primary);
  border-color: var(--accent-primary);
}

/* 紫色主题 */
.action-button-success {
  color: #8b5cf6;
  border-color: #8b5cf6;
}
```

### 添加快捷键

```typescript
useEffect(() => {
  const handleKeyPress = (e: KeyboardEvent) => {
    if (e.ctrlKey && e.key === 'k') {
      // Ctrl+K 打开授权对话框
      if (selectedAccount && selectedAccount.kiro_password) {
        handleOpenKiroAuth(selectedAccount);
      }
    }
  };
  
  window.addEventListener('keydown', handleKeyPress);
  return () => window.removeEventListener('keydown', handleKeyPress);
}, [selectedAccount]);
```

## 📊 统计信息

### 可授权账号数量

可以在界面上显示可授权的账号数量：

```typescript
const authorizableCount = accounts.filter(
  a => a.status === 'registered' && a.kiro_password
).length;

// 在界面上显示
<div className="stats">
  可授权账号: {authorizableCount}
</div>
```

### 批量授权按钮

可以添加一个批量授权按钮：

```typescript
const handleBatchAuthorize = async () => {
  const authorizableAccounts = accounts.filter(
    a => a.status === 'registered' && a.kiro_password
  );
  
  for (const account of authorizableAccounts) {
    try {
      // 为每个账号生成授权
      await kiroAuthApi.oauthLoginGoogle();
      console.log(`✓ ${account.email} 授权成功`);
    } catch (error) {
      console.error(`✗ ${account.email} 授权失败:`, error);
    }
  }
};
```

## 🐛 故障排查

### 问题 1: 授权按钮不显示

**原因:**
- 账号状态不是 `registered`
- 账号没有 `kiro_password`

**解决:**
1. 检查账号状态
2. 确认注册是否成功
3. 查看数据库中的 `kiro_password` 字段

### 问题 2: 点击按钮没有反应

**原因:**
- JavaScript 错误
- 对话框组件未正确加载

**解决:**
1. 打开浏览器控制台查看错误
2. 检查 `KiroAuthDialog` 组件是否正确导入
3. 验证 `selectedAccount` 状态

### 问题 3: 对话框显示但无法登录

**原因:**
- OAuth 服务不可用
- 网络连接问题
- 浏览器被阻止

**解决:**
1. 检查网络连接
2. 查看控制台错误信息
3. 尝试手动打开浏览器
4. 检查防火墙设置

## 📝 总结

### 新增内容

✅ 在操作列添加授权按钮（🔑）  
✅ 只在已注册账号上显示  
✅ 点击打开授权对话框  
✅ 支持多种登录方式  
✅ 绿色主题样式  
✅ 悬停效果

### 使用方式

1. 注册账号
2. 等待状态变为"已注册"
3. 点击授权按钮（🔑）
4. 选择登录方式
5. 完成 OAuth 授权
6. 启动 Kiro IDE

### 优势

- ✅ 用户主动控制
- ✅ 界面清晰直观
- ✅ 支持重复授权
- ✅ 适合批量处理
- ✅ 无干扰体验

---

**状态:** ✅ 已实现并可用

**位置:** 账号列表 → 操作列 → 🔑 按钮
