// Kiro IDE 自动登录工具
// 在注册成功后自动执行 OAuth 登录

import { kiroAuthApi } from '../api';

export interface AutoLoginOptions {
  email: string;
  kiroPassword: string;
  authMode?: 'real' | 'simulate';
  authMethod?: 'social' | 'idc';
  socialProvider?: 'google' | 'github';
}

export interface AutoLoginResult {
  success: boolean;
  message: string;
  error?: string;
}

/**
 * 自动执行 Kiro OAuth 登录
 * 在注册成功后调用
 */
export async function autoLoginKiro(options: AutoLoginOptions): Promise<AutoLoginResult> {
  const {
    email,
    kiroPassword,
    authMode = 'real',
    authMethod = 'social',
    socialProvider = 'google',
  } = options;

  console.log(`[Auto Login] Starting Kiro ${authMethod} login for ${email}`);

  try {
    if (authMode === 'simulate') {
      // 模拟登录
      const result = await kiroAuthApi.simulateLogin(email, kiroPassword, authMethod);
      return {
        success: true,
        message: result,
      };
    } else {
      // 真实 OAuth 登录
      let result;
      let providerName = '';

      if (authMethod === 'social') {
        if (socialProvider === 'google') {
          result = await kiroAuthApi.oauthLoginGoogle();
          providerName = 'Google';
        } else {
          result = await kiroAuthApi.oauthLoginGithub();
          providerName = 'GitHub';
        }
      } else {
        result = await kiroAuthApi.oauthLoginBuilderId();
        providerName = 'AWS Builder ID';
      }

      console.log('[Auto Login] OAuth login successful:', result);

      return {
        success: true,
        message: `✓ ${providerName} 登录成功！授权文件已保存到 ~/.aws/sso/cache/`,
      };
    }
  } catch (error) {
    console.error('[Auto Login] Failed:', error);
    return {
      success: false,
      message: '自动登录失败',
      error: String(error),
    };
  }
}

/**
 * 询问用户是否要自动登录
 * 返回用户选择的配置
 */
export async function promptAutoLogin(): Promise<AutoLoginOptions | null> {
  // 这个函数可以在 UI 中实现
  // 返回 null 表示用户取消
  return null;
}
