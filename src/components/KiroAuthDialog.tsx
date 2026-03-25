import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import './KiroAuthDialog.css';

interface KiroAuthDialogProps {
  isOpen: boolean;
  onClose: () => void;
  email: string;
  kiroPassword: string;
  accountId: number;
}

export function KiroAuthDialog({ isOpen, onClose, email, kiroPassword, accountId }: KiroAuthDialogProps) {
  const [loading, setLoading] = useState(false);
  const [message, setMessage] = useState('');
  const [error, setError] = useState('');

  const handleAutomatedLogin = async () => {
    setLoading(true);
    setMessage('');
    setError('');

    try {
      const result = await invoke<string>('builder_id_automated_login', {
        accountId,
      });
      setMessage(result);
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  };

  if (!isOpen) return null;

  return (
    <div className="kiro-auth-overlay" onClick={onClose}>
      <div className="kiro-auth-dialog" onClick={(e) => e.stopPropagation()}>
        <div className="kiro-auth-header">
          <h2 className="kiro-auth-title">AWS Builder ID 授权</h2>
          <button onClick={onClose} className="kiro-auth-close">
            ✕
          </button>
        </div>

        <div className="kiro-auth-content">
          <div>
            <p style={{ fontSize: '14px', color: '#6b7280', marginBottom: '8px' }}>
              使用浏览器自动化完成 AWS Builder ID 登录，无需手动操作。
            </p>
            <div className="kiro-auth-info">
              <p><strong>邮箱:</strong> {email}</p>
              <p><strong>密码:</strong> {kiroPassword}</p>
            </div>
          </div>

          <div className="kiro-auth-notice">
            <p className="kiro-auth-notice-title">自动化登录流程：</p>
            <ol>
              <li>自动注册 AWS SSO OIDC 客户端</li>
              <li>发起设备授权请求</li>
              <li>浏览器自动打开并完成登录</li>
              <li>自动输入邮箱和密码</li>
              <li>自动点击授权按钮</li>
              <li>轮询获取 Token</li>
              <li>生成并保存授权文件</li>
            </ol>
            <p style={{ marginTop: '8px', fontSize: '12px', color: '#6b7280' }}>
              整个过程完全自动化，预计需要 30-60 秒。
            </p>
          </div>

          {message && (
            <div className="kiro-auth-message success">
              {message.split('\n').map((line, i) => (
                <div key={i}>{line}</div>
              ))}
            </div>
          )}

          {error && (
            <div className="kiro-auth-message error">
              {error}
            </div>
          )}

          <div className="kiro-auth-buttons">
            <button
              onClick={handleAutomatedLogin}
              disabled={loading}
              className="kiro-auth-button kiro-auth-button-primary"
              style={{ flex: 1 }}
            >
              {loading ? '正在自动化登录...' : '开始自动化登录'}
            </button>
          </div>

          <div className="kiro-auth-footer">
            <p>📁 授权文件位置:</p>
            <p><code>~/.aws/sso/cache/kiro-auth-token.json</code></p>
            <p><code>~/.aws/sso/cache/[clientIdHash].json</code></p>
            <p style={{ marginTop: '8px' }}>
              生成后可直接启动 Kiro IDE 使用此账号登录。
            </p>
          </div>
        </div>
      </div>
    </div>
  );
}
