# Kiro IDE 授权 API 使用示例

## 前端调用示例

### 1. 生成 Social 授权

```typescript
import { kiroAuthApi } from './api';

async function generateGoogleAuth() {
  try {
    const result = await kiroAuthApi.generateSocialAuth(
      'eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...', // accessToken
      'eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...', // refreshToken
      'Google',                                      // provider
      'arn:aws:codewhisperer:us-east-1:699475941385:profile/EHGA3GRVQMUK' // profileArn (可选)
    );
    console.log(result); // "✓ Kiro Google Social 授权文件已生成"
  } catch (error) {
    console.error('生成失败:', error);
  }
}
```

### 2. 生成 IdC 授权

```typescript
import { kiroAuthApi } from './api';
import { sha256 } from 'crypto';

async function generateBuilderIdAuth() {
  const clientId = 'xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx';
  const clientSecret = 'xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx';
  
  // 生成 clientIdHash
  const hash = sha256(clientId);
  const clientIdHash = hash.toString('hex');
  
  try {
    const result = await kiroAuthApi.generateIdcAuth(
      'eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...', // accessToken
      'eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...', // refreshToken
      'BuilderId',                                   // provider
      clientId,
      clientSecret,
      clientIdHash,
      'us-east-1'                                    // region (可选，默认 us-east-1)
    );
    console.log(result); // "✓ Kiro BuilderId IdC 授权文件已生成"
  } catch (error) {
    console.error('生成失败:', error);
  }
}
```

### 3. 读取当前授权

```typescript
import { kiroAuthApi } from './api';

async function readCurrentAuth() {
  try {
    const token = await kiroAuthApi.readAuthToken();
    console.log('授权信息:', {
      provider: token.provider,
      authMethod: token.auth_method,
      expiresAt: token.expires_at,
      hasAccessToken: !!token.access_token,
      hasRefreshToken: !!token.refresh_token,
    });
  } catch (error) {
    console.error('读取失败:', error);
  }
}
```

### 4. 模拟登录（测试用）

```typescript
import { kiroAuthApi } from './api';

async function simulateLogin() {
  try {
    // Social 登录
    const result1 = await kiroAuthApi.simulateLogin(
      'user@example.com',
      'SecurePassword123!',
      'social'
    );
    console.log(result1); // "✓ 已为 user@example.com 生成 Kiro Social 授权"
    
    // IdC 登录
    const result2 = await kiroAuthApi.simulateLogin(
      'user@example.com',
      'SecurePassword123!',
      'idc'
    );
    console.log(result2); // "✓ 已为 user@example.com 生成 Kiro IdC 授权"
  } catch (error) {
    console.error('模拟登录失败:', error);
  }
}
```

## React 组件示例

### 完整的授权对话框

```tsx
import { useState } from 'react';
import { kiroAuthApi } from '../api';

interface KiroAuthPanelProps {
  email: string;
  onSuccess?: () => void;
}

export function KiroAuthPanel({ email, onSuccess }: KiroAuthPanelProps) {
  const [authMethod, setAuthMethod] = useState<'social' | 'idc'>('social');
  const [loading, setLoading] = useState(false);
  const [message, setMessage] = useState('');

  const handleGenerate = async () => {
    setLoading(true);
    setMessage('');

    try {
      const result = await kiroAuthApi.simulateLogin(
        email,
        'password',
        authMethod
      );
      setMessage(result);
      onSuccess?.();
    } catch (error) {
      setMessage('生成失败: ' + error);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="kiro-auth-panel">
      <h3>生成 Kiro IDE 授权</h3>
      
      <div className="auth-method-selector">
        <label>
          <input
            type="radio"
            value="social"
            checked={authMethod === 'social'}
            onChange={(e) => setAuthMethod(e.target.value as 'social')}
          />
          Social 登录
        </label>
        <label>
          <input
            type="radio"
            value="idc"
            checked={authMethod === 'idc'}
            onChange={(e) => setAuthMethod(e.target.value as 'idc')}
          />
          IdC 登录
        </label>
      </div>

      <button onClick={handleGenerate} disabled={loading}>
        {loading ? '生成中...' : '生成授权'}
      </button>

      {message && <div className="message">{message}</div>}
    </div>
  );
}
```

### 在注册流程中集成

```tsx
import { useState } from 'react';
import { api, kiroAuthApi } from '../api';
import { KiroAuthPanel } from './KiroAuthPanel';

export function RegistrationFlow() {
  const [step, setStep] = useState<'register' | 'auth'>('register');
  const [email, setEmail] = useState('');

  const handleRegister = async () => {
    try {
      // 执行注册
      await api.startRegistration(accountId);
      
      // 注册成功，进入授权步骤
      setStep('auth');
    } catch (error) {
      console.error('注册失败:', error);
    }
  };

  const handleAuthComplete = () => {
    console.log('授权完成！');
    // 可以跳转到其他页面或显示成功消息
  };

  return (
    <div>
      {step === 'register' && (
        <div>
          <h2>账号注册</h2>
          <button onClick={handleRegister}>开始注册</button>
        </div>
      )}

      {step === 'auth' && (
        <KiroAuthPanel
          email={email}
          onSuccess={handleAuthComplete}
        />
      )}
    </div>
  );
}
```

## Rust 后端示例

### 直接调用 Rust 函数

```rust
use crate::kiro_auth::{SocialAuthParams, IdcAuthParams};

// 生成 Social 授权
fn example_social_auth() -> Result<(), String> {
    let params = SocialAuthParams {
        access_token: "eyJ...".to_string(),
        refresh_token: "eyJ...".to_string(),
        provider: "Google".to_string(),
        profile_arn: Some("arn:aws:codewhisperer:us-east-1:699475941385:profile/EHGA3GRVQMUK".to_string()),
    };

    kiro_auth::generate_kiro_social_auth(params)?;
    println!("✓ Social 授权已生成");
    Ok(())
}

// 生成 IdC 授权
fn example_idc_auth() -> Result<(), String> {
    use sha2::{Sha256, Digest};
    
    let client_id = "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx".to_string();
    let client_secret = "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx".to_string();
    
    // 生成 clientIdHash
    let mut hasher = Sha256::new();
    hasher.update(client_id.as_bytes());
    let client_id_hash = hex::encode(hasher.finalize());
    
    let params = IdcAuthParams {
        access_token: "eyJ...".to_string(),
        refresh_token: "eyJ...".to_string(),
        provider: "BuilderId".to_string(),
        client_id,
        client_secret,
        client_id_hash,
        region: "us-east-1".to_string(),
    };

    kiro_auth::generate_kiro_idc_auth(params)?;
    println!("✓ IdC 授权已生成");
    Ok(())
}

// 读取当前授权
fn example_read_auth() -> Result<(), String> {
    let token = kiro_auth::read_kiro_auth_token()?;
    println!("当前授权:");
    println!("  Provider: {}", token.provider);
    println!("  Auth Method: {}", token.auth_method);
    println!("  Expires At: {}", token.expires_at);
    Ok(())
}
```

### 在 Tauri 命令中使用

```rust
#[tauri::command]
pub async fn custom_kiro_login(
    email: String,
    password: String,
) -> Result<String, String> {
    // 1. 执行实际的登录流程（调用 OAuth API）
    let (access_token, refresh_token) = perform_oauth_login(&email, &password).await?;
    
    // 2. 生成授权文件
    let params = SocialAuthParams {
        access_token,
        refresh_token,
        provider: "Google".to_string(),
        profile_arn: None,
    };
    
    kiro_auth::generate_kiro_social_auth(params)?;
    
    Ok(format!("✓ {} 登录成功并生成授权", email))
}

async fn perform_oauth_login(email: &str, password: &str) -> Result<(String, String), String> {
    // 实现真实的 OAuth 登录流程
    // 这里只是示例
    Ok((
        "access_token_here".to_string(),
        "refresh_token_here".to_string(),
    ))
}
```

## 测试示例

### 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_social_auth() {
        let params = SocialAuthParams {
            access_token: "test_access".to_string(),
            refresh_token: "test_refresh".to_string(),
            provider: "Google".to_string(),
            profile_arn: None,
        };

        let result = kiro_auth::generate_kiro_social_auth(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_read_auth_token() {
        // 先生成一个测试 token
        let params = SocialAuthParams {
            access_token: "test_access".to_string(),
            refresh_token: "test_refresh".to_string(),
            provider: "Google".to_string(),
            profile_arn: None,
        };
        kiro_auth::generate_kiro_social_auth(params).unwrap();

        // 读取并验证
        let token = kiro_auth::read_kiro_auth_token().unwrap();
        assert_eq!(token.provider, "Google");
        assert_eq!(token.auth_method, "social");
    }
}
```

### 集成测试

```typescript
describe('Kiro Auth API', () => {
  it('should generate social auth', async () => {
    const result = await kiroAuthApi.generateSocialAuth(
      'test_access',
      'test_refresh',
      'Google'
    );
    expect(result).toContain('授权文件已生成');
  });

  it('should read auth token', async () => {
    // 先生成
    await kiroAuthApi.simulateLogin('test@example.com', 'password', 'social');
    
    // 再读取
    const token = await kiroAuthApi.readAuthToken();
    expect(token.provider).toBe('Google');
    expect(token.auth_method).toBe('social');
  });
});
```

## 命令行工具示例

如果需要从命令行生成授权：

```bash
# 使用 Tauri CLI
npm run tauri dev

# 在开发者控制台中执行
window.__TAURI__.invoke('simulate_kiro_login', {
  email: 'user@example.com',
  kiroPassword: 'password',
  authMethod: 'social'
}).then(console.log).catch(console.error);
```

## 最佳实践

1. **错误处理**: 始终使用 try-catch 包裹 API 调用
2. **用户反馈**: 显示加载状态和结果消息
3. **验证输入**: 在调用 API 前验证必要参数
4. **日志记录**: 记录关键操作用于调试
5. **安全性**: 不要在日志中输出完整的 Token

```typescript
async function safeGenerateAuth(email: string, authMethod: 'social' | 'idc') {
  // 1. 验证输入
  if (!email || !email.includes('@')) {
    throw new Error('无效的邮箱地址');
  }

  // 2. 显示加载状态
  setLoading(true);
  setError(null);

  try {
    // 3. 调用 API
    const result = await kiroAuthApi.simulateLogin(email, 'password', authMethod);
    
    // 4. 记录成功（不输出敏感信息）
    console.log(`授权生成成功: ${email} (${authMethod})`);
    
    // 5. 用户反馈
    setMessage(result);
    
    return result;
  } catch (error) {
    // 6. 错误处理
    console.error('授权生成失败:', error);
    setError(error.message);
    throw error;
  } finally {
    // 7. 清理状态
    setLoading(false);
  }
}
```
