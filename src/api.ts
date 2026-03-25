import { invoke } from '@tauri-apps/api/core';
import { open, save } from '@tauri-apps/plugin-dialog';
import { readTextFile, writeTextFile } from '@tauri-apps/plugin-fs';
import { Account, Settings } from './store';

export interface NewAccount {
  email: string;
  email_password: string;
  client_id: string;
  refresh_token: string;
}

export interface AccountUpdate {
  id: number;
  email?: string;
  email_password?: string;
  client_id?: string;
  refresh_token?: string;
  kiro_password?: string;
  status?: 'not_registered' | 'in_progress' | 'registered' | 'error';
  error_reason?: string;
  oauth_status?: 'not_authorized' | 'in_progress' | 'authorized' | 'error';
  oauth_info?: string;
}

export interface ImportResult {
  success_count: number;
  error_count: number;
  errors: Array<{
    line_number: number;
    content: string;
    reason: string;
  }>;
}

export const api = {
  async getAccounts(statusFilter?: string): Promise<Account[]> {
    return invoke('get_accounts', { statusFilter });
  },

  async addAccount(account: NewAccount): Promise<number> {
    return invoke('add_account', { account });
  },

  async updateAccount(update: AccountUpdate): Promise<void> {
    return invoke('update_account', { update });
  },

  async deleteAccount(id: number): Promise<void> {
    return invoke('delete_account', { id });
  },

  async deleteAllAccounts(): Promise<void> {
    return invoke('delete_all_accounts');
  },

  async importAccounts(content: string): Promise<ImportResult> {
    return invoke('import_accounts', { content });
  },

  async getSettings(): Promise<Settings> {
    return invoke('get_settings');
  },

  async updateSettings(settings: Settings): Promise<void> {
    return invoke('update_settings', { settings });
  },

  async startRegistration(accountId: number): Promise<string> {
    return invoke('start_registration', { accountId });
  },

  async startRegistrationWithOAuth(accountId: number): Promise<string> {
    return invoke('start_registration_with_oauth', { accountId });
  },

  async startBatchRegistration(): Promise<string> {
    return invoke('start_batch_registration');
  },

  async exportAccounts(statusFilter?: string): Promise<void> {
    const content: string = await invoke('export_accounts', { statusFilter });

    if (!content) {
      throw new Error('没有可导出的数据');
    }

    const filePath = await save({
      filters: [{
        name: 'Text Files',
        extensions: ['txt']
      }],
      defaultPath: 'accounts.txt'
    });

    if (filePath) {
      await writeTextFile(filePath, content);
    }
  },

  async selectFile(): Promise<string | null> {
    const selected = await open({
      multiple: false,
      filters: [{
        name: 'Text Files',
        extensions: ['txt']
      }]
    });

    if (selected && typeof selected === 'string') {
      const content = await readTextFile(selected);
      return content;
    }

    return null;
  }
};

// Kiro IDE 授权相关 API
export interface KiroAuthToken {
  access_token: string;
  refresh_token: string;
  expires_at: string;
  auth_method: string;
  provider: string;
  profile_arn?: string;
  client_id_hash?: string;
  region?: string;
}

export interface KiroOAuthResult {
  access_token: string;
  refresh_token: string;
  provider: string;
  profile_arn?: string;
  expires_at: string;
}

export const kiroAuthApi = {
  async generateSocialAuth(
    accessToken: string,
    refreshToken: string,
    provider: string,
    profileArn?: string
  ): Promise<string> {
    return invoke('generate_kiro_social_auth', {
      accessToken,
      refreshToken,
      provider,
      profileArn,
    });
  },

  async generateIdcAuth(
    accessToken: string,
    refreshToken: string,
    provider: string,
    clientId: string,
    clientSecret: string,
    clientIdHash: string,
    region?: string
  ): Promise<string> {
    return invoke('generate_kiro_idc_auth', {
      accessToken,
      refreshToken,
      provider,
      clientId,
      clientSecret,
      clientIdHash,
      region,
    });
  },

  async readAuthToken(): Promise<KiroAuthToken> {
    return invoke('read_kiro_auth_token');
  },

  async simulateLogin(
    email: string,
    kiroPassword: string,
    authMethod: 'social' | 'idc'
  ): Promise<string> {
    return invoke('simulate_kiro_login', {
      email,
      kiroPassword,
      authMethod,
    });
  },

  // 真实 OAuth 登录
  async oauthLoginGoogle(): Promise<KiroOAuthResult> {
    return invoke('kiro_oauth_login_google');
  },

  async oauthLoginGithub(): Promise<KiroOAuthResult> {
    return invoke('kiro_oauth_login_github');
  },

  async oauthLoginBuilderId(): Promise<KiroOAuthResult> {
    return invoke('kiro_oauth_login_builder_id');
  },

  async handleOAuthCallback(url: string): Promise<boolean> {
    return invoke('handle_oauth_callback_url', { url });
  },

  // OAuth 授权管理
  async getOAuthInfo(accountId: number): Promise<import('./store').OAuthInfo | null> {
    return invoke('get_oauth_info', { accountId });
  },

  async manualOAuthAuthorization(accountId: number): Promise<string> {
    return invoke('manual_oauth_authorization', { accountId });
  },

  async exportKiroAuthJson(accountId: number): Promise<void> {
    const content: string = await invoke('export_kiro_auth_json', { accountId });

    if (!content) {
      throw new Error('没有可导出的数据');
    }

    const filePath = await save({
      filters: [{
        name: 'JSON Files',
        extensions: ['json']
      }],
      defaultPath: 'kiro-auth-token.json'
    });

    if (filePath) {
      await writeTextFile(filePath, content);
    }
  },
};
