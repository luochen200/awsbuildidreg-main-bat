import { useState, useMemo } from "react";
import {
  Trash2,
  Edit,
  Eye,
  Play,
  Loader2,
  Zap,
  Shield,
  Info,
  Copy,
  Check,
  Download,
} from "lucide-react";
import { Account, OAuthInfo } from "../store";
import { api, kiroAuthApi } from "../api";
import { showConfirm, showSuccess, showError } from "../utils/dialog";
import "./AccountsTable.css";

interface AccountsTableProps {
  accounts: Account[];
  onRefresh: () => void;
}

export function AccountsTable({ accounts, onRefresh }: AccountsTableProps) {
  const [search, setSearch] = useState("");
  const [sortField, setSortField] = useState<keyof Account>("created_at");
  const [sortDirection, setSortDirection] = useState<"asc" | "desc">("desc");
  const [currentPage, setCurrentPage] = useState(1);
  const [selectedAccount, setSelectedAccount] = useState<Account | null>(null);
  const [isDetailModalOpen, setIsDetailModalOpen] = useState(false);
  const [isEditModalOpen, setIsEditModalOpen] = useState(false);
  const [isOAuthDetailModalOpen, setIsOAuthDetailModalOpen] = useState(false);
  const [oauthInfo, setOAuthInfo] = useState<OAuthInfo | null>(null);
  const [processingId, setProcessingId] = useState<number | null>(null);
  const [selectedIds, setSelectedIds] = useState<Set<number>>(new Set());
  const [exportingBatch, setExportingBatch] = useState(false);

  const itemsPerPage = 20;

  const filteredAccounts = useMemo(() => {
    return accounts.filter(
      (account) =>
        account.email.toLowerCase().includes(search.toLowerCase()) ||
        account.status.toLowerCase().includes(search.toLowerCase()),
    );
  }, [accounts, search]);

  const sortedAccounts = useMemo(() => {
    return [...filteredAccounts].sort((a, b) => {
      const aValue = a[sortField];
      const bValue = b[sortField];

      if (aValue === null || aValue === undefined) return 1;
      if (bValue === null || bValue === undefined) return -1;

      if (typeof aValue === "string" && typeof bValue === "string") {
        return sortDirection === "asc"
          ? aValue.localeCompare(bValue)
          : bValue.localeCompare(aValue);
      }

      return sortDirection === "asc"
        ? aValue > bValue
          ? 1
          : -1
        : bValue > aValue
          ? 1
          : -1;
    });
  }, [filteredAccounts, sortField, sortDirection]);

  const paginatedAccounts = useMemo(() => {
    const start = (currentPage - 1) * itemsPerPage;
    const end = start + itemsPerPage;
    return sortedAccounts.slice(start, end);
  }, [sortedAccounts, currentPage]);

  const totalPages = Math.ceil(sortedAccounts.length / itemsPerPage);

  const handleSort = (field: keyof Account) => {
    if (sortField === field) {
      setSortDirection(sortDirection === "asc" ? "desc" : "asc");
    } else {
      setSortField(field);
      setSortDirection("asc");
    }
  };

  const handleDelete = async (id: number) => {
    const confirmed = await showConfirm("确定要删除这条记录吗?", "确认删除");
    if (confirmed) {
      try {
        await api.deleteAccount(id);
        onRefresh();
      } catch (error) {
        await showError("删除失败: " + error);
      }
    }
  };

  const handleStartRegistration = async (id: number) => {
    if (processingId) {
      return;
    }

    setProcessingId(id);
    try {
      const result = await api.startRegistration(id);
      onRefresh();

      await showSuccess(result);
    } catch (error) {
      onRefresh();
    } finally {
      setProcessingId(null);
    }
  };

  // 注册并自动完成 OAuth 授权
  const handleStartRegistrationWithOAuth = async (id: number) => {
    if (processingId) {
      return;
    }

    const confirmed = await showConfirm(
      "将执行注册并自动完成 Builder ID OAuth 授权。\n\n整个过程约需 80-120 秒，期间请勿关闭窗口。",
      "注册并授权",
    );

    if (!confirmed) {
      return;
    }

    setProcessingId(id);
    try {
      const result = await api.startRegistrationWithOAuth(id);
      onRefresh();

      await showSuccess(result);
    } catch (error) {
      onRefresh();
    } finally {
      setProcessingId(null);
    }
  };

  // 手动执行 OAuth 授权
  const handleManualOAuth = async (id: number) => {
    if (processingId) {
      return;
    }

    const confirmed = await showConfirm(
      "将执行 Builder ID OAuth 授权。\n\n整个过程约需 60-90 秒，期间请勿关闭窗口。",
      "OAuth 授权",
    );

    if (!confirmed) {
      return;
    }

    setProcessingId(id);
    try {
      const result = await kiroAuthApi.manualOAuthAuthorization(id);
      onRefresh();
      await showSuccess(result);
    } catch (error) {
      onRefresh();
      await showError("OAuth 授权失败: " + error);
    } finally {
      setProcessingId(null);
    }
  };

  // 查看 OAuth 授权详情
  const handleViewOAuthInfo = async (account: Account) => {
    try {
      const info = await kiroAuthApi.getOAuthInfo(account.id);
      if (info) {
        setOAuthInfo(info);
        setSelectedAccount(account);
        setIsOAuthDetailModalOpen(true);
      } else {
        await showError("未找到 OAuth 授权信息");
      }
    } catch (error) {
      await showError("获取 OAuth 信息失败: " + error);
    }
  };

  const getStatusText = (status: string) => {
    const statusMap: Record<string, string> = {
      not_registered: "未注册",
      in_progress: "进行中",
      registered: "已注册",
      error: "异常",
    };
    return statusMap[status] || status;
  };

  const getOAuthStatusText = (status: string) => {
    const statusMap: Record<string, string> = {
      not_authorized: "未授权",
      in_progress: "授权中",
      authorized: "已授权",
      error: "授权异常",
    };
    return statusMap[status] || status;
  };

  const getStatusClass = (status: string) => {
    return `status-badge status-${status.replace("_", "-")}`;
  };

  const getOAuthStatusClass = (status: string) => {
    return `oauth-status-badge oauth-status-${status.replace("_", "-")}`;
  };

  // 全选/取消全选
  const handleSelectAll = (checked: boolean) => {
    if (checked) {
      const allIds = new Set(paginatedAccounts.map((acc) => acc.id));
      setSelectedIds(allIds);
    } else {
      setSelectedIds(new Set());
    }
  };

  // 单选
  const handleSelectOne = (id: number, checked: boolean) => {
    const newSelected = new Set(selectedIds);
    if (checked) {
      newSelected.add(id);
    } else {
      newSelected.delete(id);
    }
    setSelectedIds(newSelected);
  };

  // 批量导出已授权账号
  const handleBatchExport = async () => {
    if (selectedIds.size === 0) {
      await showError("请先选择要导出的账号");
      return;
    }

    setExportingBatch(true);
    try {
      const selectedAccounts = accounts.filter((acc) =>
        selectedIds.has(acc.id),
      );
      const authorizedAccounts = selectedAccounts.filter(
        (acc) => acc.oauth_status === "authorized",
      );

      if (authorizedAccounts.length === 0) {
        await showError("所选账号中没有已授权的账号");
        setExportingBatch(false);
        return;
      }

      // 获取所有已授权账号的 OAuth 信息
      const exportData = [];
      for (const account of authorizedAccounts) {
        try {
          const info = await kiroAuthApi.getOAuthInfo(account.id);
          if (info) {
            exportData.push({
              refreshToken: info.refresh_token || "",
              clientId: info.client_id || "",
              clientSecret: info.client_secret || "",
              region: info.region || "us-east-1",
              provider: "BuilderId",
              machineId: "",
            });
          }
        } catch (error) {
          console.error(`获取账号 ${account.email} 的 OAuth 信息失败:`, error);
        }
      }

      if (exportData.length === 0) {
        await showError("无法获取 OAuth 信息");
        setExportingBatch(false);
        return;
      }

      // 使用文件保存对话框而不是剪贴板
      const { save } = await import('@tauri-apps/plugin-dialog');
      const { writeTextFile } = await import('@tauri-apps/plugin-fs');
      
      const filePath = await save({
        filters: [{
          name: 'JSON Files',
          extensions: ['json']
        }],
        defaultPath: 'selected-accounts.json'
      });

      if (filePath) {
        await writeTextFile(filePath, JSON.stringify(exportData, null, 2));
        await showSuccess(`已导出 ${exportData.length} 个已授权账号到文件！`);
      }

      // 清空选择
      setSelectedIds(new Set());
    } catch (error) {
      await showError("批量导出失败: " + error);
    } finally {
      setExportingBatch(false);
    }
  };

  // 导出所有已授权账号
  const handleExportAll = async () => {
    const authorizedAccounts = accounts.filter(
      (acc) => acc.oauth_status === "authorized",
    );

    if (authorizedAccounts.length === 0) {
      await showError("没有已授权的账号");
      return;
    }

    const confirmed = await showConfirm(
      `将导出所有 ${authorizedAccounts.length} 个已授权账号，是否继续？`,
      "批量导出",
    );

    if (!confirmed) {
      return;
    }

    setExportingBatch(true);
    try {
      const exportData = [];
      for (const account of authorizedAccounts) {
        try {
          const info = await kiroAuthApi.getOAuthInfo(account.id);
          if (info) {
            exportData.push({
              refreshToken: info.refresh_token || "",
              clientId: info.client_id || "",
              clientSecret: info.client_secret || "",
              region: info.region || "us-east-1",
              provider: "BuilderId",
              machineId: "",
            });
          }
        } catch (error) {
          console.error(`获取账号 ${account.email} 的 OAuth 信息失败:`, error);
        }
      }

      if (exportData.length === 0) {
        await showError("无法获取 OAuth 信息");
        setExportingBatch(false);
        return;
      }

      // 使用文件保存对话框而不是剪贴板
      const { save } = await import('@tauri-apps/plugin-dialog');
      const { writeTextFile } = await import('@tauri-apps/plugin-fs');
      
      const filePath = await save({
        filters: [{
          name: 'JSON Files',
          extensions: ['json']
        }],
        defaultPath: 'authorized-accounts.json'
      });

      if (filePath) {
        await writeTextFile(filePath, JSON.stringify(exportData, null, 2));
        await showSuccess(`已导出 ${exportData.length} 个已授权账号到文件！`);
      }
    } catch (error) {
      await showError("批量导出失败: " + error);
    } finally {
      setExportingBatch(false);
    }
  };

  const isAllSelected =
    paginatedAccounts.length > 0 &&
    paginatedAccounts.every((acc) => selectedIds.has(acc.id));
  const isSomeSelected =
    paginatedAccounts.some((acc) => selectedIds.has(acc.id)) && !isAllSelected;
  const authorizedCount = accounts.filter(
    (acc) => acc.oauth_status === "authorized",
  ).length;

  return (
    <div className="accounts-table-container">
      <div className="table-header">
        <input
          type="text"
          placeholder="搜索邮箱或状态..."
          value={search}
          onChange={(e) => {
            setSearch(e.target.value);
            setCurrentPage(1);
          }}
          className="search-input"
        />
        <div className="table-actions">
          {selectedIds.size > 0 && (
            <button
              onClick={handleBatchExport}
              disabled={exportingBatch}
              className="button-primary batch-export-button"
            >
              {exportingBatch ? (
                <>
                  <Loader2 size={16} className="spin" />
                  导出中...
                </>
              ) : (
                <>
                  <Download size={16} />
                  导出选中 ({selectedIds.size})
                </>
              )}
            </button>
          )}
          <button
            onClick={handleExportAll}
            disabled={exportingBatch || authorizedCount === 0}
            className="button-secondary batch-export-button"
          >
            {exportingBatch ? (
              <>
                <Loader2 size={16} className="spin" />
                导出中...
              </>
            ) : (
              <>
                <Download size={16} />
                导出全部已授权 ({authorizedCount})
              </>
            )}
          </button>
          <div className="table-stats">共 {sortedAccounts.length} 条记录</div>
        </div>
      </div>

      <div className="table-wrapper">
        <table className="accounts-table">
          <thead>
            <tr>
              <th className="checkbox-cell">
                <input
                  type="checkbox"
                  checked={isAllSelected}
                  ref={(input) => {
                    if (input) {
                      input.indeterminate = isSomeSelected;
                    }
                  }}
                  onChange={(e) => handleSelectAll(e.target.checked)}
                />
              </th>
              <th onClick={() => handleSort("id")}>
                序号{" "}
                {sortField === "id" && (sortDirection === "asc" ? "↑" : "↓")}
              </th>
              <th onClick={() => handleSort("email")}>
                注册邮箱{" "}
                {sortField === "email" && (sortDirection === "asc" ? "↑" : "↓")}
              </th>
              <th>邮箱密码</th>
              <th onClick={() => handleSort("status")}>
                状态{" "}
                {sortField === "status" &&
                  (sortDirection === "asc" ? "↑" : "↓")}
              </th>
              <th onClick={() => handleSort("oauth_status")}>
                授权状态{" "}
                {sortField === "oauth_status" &&
                  (sortDirection === "asc" ? "↑" : "↓")}
              </th>
              <th>异常原因</th>
              <th>操作</th>
            </tr>
          </thead>
          <tbody>
            {paginatedAccounts.map((account, index) => (
              <tr key={account.id}>
                <td className="checkbox-cell">
                  <input
                    type="checkbox"
                    checked={selectedIds.has(account.id)}
                    onChange={(e) =>
                      handleSelectOne(account.id, e.target.checked)
                    }
                  />
                </td>
                <td>{(currentPage - 1) * itemsPerPage + index + 1}</td>
                <td className="email-cell">{account.email}</td>
                <td>
                  <span className="password-hidden">••••••••</span>
                </td>
                <td>
                  <span className={getStatusClass(account.status)}>
                    {getStatusText(account.status)}
                  </span>
                </td>
                <td>
                  <div className="oauth-status-cell">
                    <span className={getOAuthStatusClass(account.oauth_status)}>
                      {getOAuthStatusText(account.oauth_status)}
                    </span>
                    {account.oauth_status === "authorized" &&
                      account.oauth_info && (
                        <button
                          className="oauth-info-button"
                          onClick={() => handleViewOAuthInfo(account)}
                          title="查看授权详情"
                        >
                          <Info size={14} />
                        </button>
                      )}
                  </div>
                </td>
                <td className="error-cell">
                  {account.error_reason && (
                    <span className="error-text" title={account.error_reason}>
                      {account.error_reason.substring(0, 50)}
                      {account.error_reason.length > 50 && "..."}
                    </span>
                  )}
                </td>
                <td>
                  <div className="action-buttons">
                    <button
                      className="action-button"
                      onClick={() => {
                        setSelectedAccount(account);
                        setIsDetailModalOpen(true);
                      }}
                      title="查看详情"
                    >
                      <Eye size={16} />
                    </button>
                    <button
                      className="action-button"
                      onClick={() => {
                        setSelectedAccount(account);
                        setIsEditModalOpen(true);
                      }}
                      title="编辑"
                      disabled={account.status === "in_progress"}
                    >
                      <Edit size={16} />
                    </button>
                    {account.status === "not_registered" && (
                      <>
                        <button
                          className="action-button action-button-primary"
                          onClick={() => handleStartRegistration(account.id)}
                          title="仅注册"
                          disabled={
                            processingId === account.id || processingId !== null
                          }
                        >
                          {processingId === account.id ? (
                            <Loader2 size={16} className="spin" />
                          ) : (
                            <Play size={16} />
                          )}
                        </button>
                        <button
                          className="action-button action-button-warning"
                          onClick={() =>
                            handleStartRegistrationWithOAuth(account.id)
                          }
                          title="注册并授权（推荐）"
                          disabled={
                            processingId === account.id || processingId !== null
                          }
                        >
                          {processingId === account.id ? (
                            <Loader2 size={16} className="spin" />
                          ) : (
                            <Zap size={16} />
                          )}
                        </button>
                      </>
                    )}
                    {account.status === "registered" &&
                      account.kiro_password &&
                      account.oauth_status === "not_authorized" && (
                        <button
                          className="action-button action-button-warning"
                          onClick={() => handleManualOAuth(account.id)}
                          title="OAuth 授权"
                          disabled={
                            processingId === account.id || processingId !== null
                          }
                        >
                          {processingId === account.id ? (
                            <Loader2 size={16} className="spin" />
                          ) : (
                            <Shield size={16} />
                          )}
                        </button>
                      )}
                    {account.status === "registered" &&
                      account.kiro_password &&
                      account.oauth_status === "in_progress" && (
                        <button
                          className="action-button action-button-warning"
                          title="OAuth 授权中..."
                          disabled={true}
                        >
                          <Loader2 size={16} className="spin" />
                        </button>
                      )}
                    <button
                      className="action-button action-button-danger"
                      onClick={() => handleDelete(account.id)}
                      title="删除"
                      disabled={account.status === "in_progress"}
                    >
                      <Trash2 size={16} />
                    </button>
                  </div>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      {totalPages > 1 && (
        <div className="pagination">
          <button
            onClick={() => setCurrentPage((p) => Math.max(1, p - 1))}
            disabled={currentPage === 1}
            className="pagination-button"
          >
            上一页
          </button>
          <span className="pagination-info">
            第 {currentPage} / {totalPages} 页
          </span>
          <button
            onClick={() => setCurrentPage((p) => Math.min(totalPages, p + 1))}
            disabled={currentPage === totalPages}
            className="pagination-button"
          >
            下一页
          </button>
        </div>
      )}

      {isDetailModalOpen && selectedAccount && (
        <DetailModal
          account={selectedAccount}
          onClose={() => {
            setIsDetailModalOpen(false);
            setSelectedAccount(null);
          }}
        />
      )}

      {isEditModalOpen && selectedAccount && (
        <EditModal
          account={selectedAccount}
          onClose={() => {
            setIsEditModalOpen(false);
            setSelectedAccount(null);
          }}
          onSave={() => {
            setIsEditModalOpen(false);
            setSelectedAccount(null);
            onRefresh();
          }}
        />
      )}

      {isOAuthDetailModalOpen && selectedAccount && oauthInfo && (
        <OAuthDetailModal
          account={selectedAccount}
          oauthInfo={oauthInfo}
          onClose={() => {
            setIsOAuthDetailModalOpen(false);
            setSelectedAccount(null);
            setOAuthInfo(null);
          }}
        />
      )}
    </div>
  );
}

function DetailModal({
  account,
  onClose,
}: {
  account: Account;
  onClose: () => void;
}) {
  const getOAuthStatusText = (status: string) => {
    const statusMap: Record<string, string> = {
      not_authorized: "未授权",
      in_progress: "授权中",
      authorized: "已授权",
      error: "授权异常",
    };
    return statusMap[status] || status;
  };

  const getOAuthStatusClass = (status: string) => {
    return `oauth-status-badge oauth-status-${status.replace("_", "-")}`;
  };

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal-content" onClick={(e) => e.stopPropagation()}>
        <div className="modal-header">
          <h2>账号详情</h2>
          <button className="modal-close" onClick={onClose}>
            ×
          </button>
        </div>
        <div className="modal-body">
          <div className="detail-item">
            <label>ID:</label>
            <span>{account.id}</span>
          </div>
          <div className="detail-item">
            <label>注册邮箱:</label>
            <span>{account.email}</span>
          </div>
          <div className="detail-item">
            <label>邮箱密码:</label>
            <span>{account.email_password}</span>
          </div>
          <div className="detail-item">
            <label>客户端ID:</label>
            <span className="monospace">{account.client_id}</span>
          </div>
          <div className="detail-item">
            <label>Refresh Token:</label>
            <span className="monospace break-all">{account.refresh_token}</span>
          </div>
          {account.kiro_password && (
            <div className="detail-item">
              <label>Kiro密码:</label>
              <span>{account.kiro_password}</span>
            </div>
          )}
          <div className="detail-item">
            <label>状态:</label>
            <span>{account.status}</span>
          </div>
          {account.error_reason && (
            <div className="detail-item">
              <label>异常原因:</label>
              <span className="error-text">{account.error_reason}</span>
            </div>
          )}
          <div className="detail-item">
            <label>OAuth 状态:</label>
            <span className={getOAuthStatusClass(account.oauth_status)}>
              {getOAuthStatusText(account.oauth_status)}
            </span>
          </div>
          <div className="detail-item">
            <label>创建时间:</label>
            <span>{new Date(account.created_at).toLocaleString("zh-CN")}</span>
          </div>
          <div className="detail-item">
            <label>更新时间:</label>
            <span>{new Date(account.updated_at).toLocaleString("zh-CN")}</span>
          </div>
        </div>
      </div>
    </div>
  );
}

function EditModal({
  account,
  onClose,
  onSave,
}: {
  account: Account;
  onClose: () => void;
  onSave: () => void;
}) {
  const [formData, setFormData] = useState({
    email: account.email,
    email_password: account.email_password,
    client_id: account.client_id,
    refresh_token: account.refresh_token,
  });

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    try {
      await api.updateAccount({
        id: account.id,
        ...formData,
      });
      await showSuccess("更新成功");
      onSave();
    } catch (error) {
      await showError("更新失败: " + error);
    }
  };

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal-content" onClick={(e) => e.stopPropagation()}>
        <div className="modal-header">
          <h2>编辑账号</h2>
          <button className="modal-close" onClick={onClose}>
            ×
          </button>
        </div>
        <form onSubmit={handleSubmit}>
          <div className="modal-body">
            <div className="form-group">
              <label>注册邮箱:</label>
              <input
                type="email"
                value={formData.email}
                onChange={(e) =>
                  setFormData({ ...formData, email: e.target.value })
                }
                required
              />
            </div>
            <div className="form-group">
              <label>邮箱密码:</label>
              <input
                type="text"
                value={formData.email_password}
                onChange={(e) =>
                  setFormData({ ...formData, email_password: e.target.value })
                }
                required
              />
            </div>
            <div className="form-group">
              <label>客户端ID:</label>
              <input
                type="text"
                value={formData.client_id}
                onChange={(e) =>
                  setFormData({ ...formData, client_id: e.target.value })
                }
                required
              />
            </div>
            <div className="form-group">
              <label>Refresh Token:</label>
              <textarea
                value={formData.refresh_token}
                onChange={(e) =>
                  setFormData({ ...formData, refresh_token: e.target.value })
                }
                required
                rows={3}
              />
            </div>
          </div>
          <div className="modal-footer">
            <button
              type="button"
              onClick={onClose}
              className="button-secondary"
            >
              取消
            </button>
            <button type="submit" className="button-primary">
              保存
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}

function OAuthDetailModal({
  account,
  oauthInfo,
  onClose,
}: {
  account: Account;
  oauthInfo: OAuthInfo;
  onClose: () => void;
}) {
  const [copied, setCopied] = useState(false);

  const formatDate = (dateString: string) => {
    try {
      return new Date(dateString).toLocaleString("zh-CN");
    } catch {
      return dateString;
    }
  };

  const isExpired = () => {
    try {
      return new Date(oauthInfo.expires_at) < new Date();
    } catch {
      return false;
    }
  };

  const handleCopyFormat = async () => {
    try {
      const formatData = [
        {
          refreshToken: oauthInfo.refresh_token || "",
          clientId: oauthInfo.client_id || "",
          clientSecret: oauthInfo.client_secret || "",
          region: oauthInfo.region || "us-east-1",
          provider: "BuilderId",
          machineId: "",
        },
      ];

      await navigator.clipboard.writeText(JSON.stringify(formatData, null, 2));
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
      await showSuccess("已复制到剪贴板！");
    } catch (error) {
      await showError("复制失败: " + error);
    }
  };

  const handleExportJson = async () => {
    try {
      await kiroAuthApi.exportKiroAuthJson(account.id);
      await showSuccess("导出成功！");
    } catch (error) {
      await showError("导出失败: " + error);
    }
  };

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div
        className="modal-content oauth-detail-modal"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="modal-header">
          <h2>OAuth 授权详情</h2>
          <button className="modal-close" onClick={onClose}>
            ×
          </button>
        </div>
        <div className="modal-body">
          <div className="detail-section">
            <h3>基本信息</h3>
            <div className="detail-item">
              <label>账号邮箱:</label>
              <span>{account.email}</span>
            </div>
            {/* <div className="detail-item">
              <label>授权提供商:</label>
              <span>{oauthInfo.provider}</span>
            </div> */}
            <div className="detail-item">
              <label>授权方式:</label>
              <span>{oauthInfo.auth_method}</span>
            </div>
            <div className="detail-item">
              <label>授权时间:</label>
              <span>{formatDate(oauthInfo.authorized_at)}</span>
            </div>
          </div>

          <div className="detail-section">
            <h3>Token 信息</h3>
            <div className="detail-item">
              <label>Access Token:</label>
              <span className="monospace token-display">
                {oauthInfo.access_token.substring(0, 20)}...
              </span>
            </div>
            <div className="detail-item">
              <label>Refresh Token:</label>
              <span className="monospace token-display">
                {oauthInfo.refresh_token.substring(0, 20)}...
              </span>
            </div>
            <div className="detail-item">
              <label>过期时间:</label>
              <span className={isExpired() ? "expired-token" : "valid-token"}>
                {formatDate(oauthInfo.expires_at)}
                {isExpired() && " (已过期)"}
              </span>
            </div>
          </div>

          {(oauthInfo.profile_arn ||
            oauthInfo.client_id_hash ||
            oauthInfo.region ||
            oauthInfo.client_id) && (
            <div className="detail-section">
              <h3>扩展信息</h3>
              {oauthInfo.profile_arn && (
                <div className="detail-item">
                  <label>Profile ARN:</label>
                  <span className="monospace break-all">
                    {oauthInfo.profile_arn}
                  </span>
                </div>
              )}
              {oauthInfo.client_id_hash && (
                <div className="detail-item">
                  <label>Client ID Hash:</label>
                  <span className="monospace">{oauthInfo.client_id_hash}</span>
                </div>
              )}
              {oauthInfo.region && (
                <div className="detail-item">
                  <label>区域:</label>
                  <span>{oauthInfo.region}</span>
                </div>
              )}
              {oauthInfo.client_id && (
                <div className="detail-item">
                  <label>Client ID:</label>
                  <span className="monospace token-display">
                    {oauthInfo.client_id.substring(0, 20)}...
                  </span>
                </div>
              )}
              {oauthInfo.client_secret && (
                <div className="detail-item">
                  <label>Client Secret:</label>
                  <span className="monospace token-display">
                    {oauthInfo.client_secret.substring(0, 20)}...
                  </span>
                </div>
              )}
              {oauthInfo.client_expires_at && (
                <div className="detail-item">
                  <label>客户端过期时间:</label>
                  <span className="valid-token">
                    {formatDate(oauthInfo.client_expires_at)}
                  </span>
                </div>
              )}
            </div>
          )}
        </div>
        <div className="modal-footer">
          <button onClick={handleCopyFormat} className="button-secondary">
            {copied ? (
              <>
                <Check size={16} /> 已复制
              </>
            ) : (
              <>
                <Copy size={16} /> 一键复制
              </>
            )}
          </button>
          <button onClick={handleExportJson} className="button-secondary">
            导出 JSON
          </button>
          <button onClick={onClose} className="button-primary">
            关闭
          </button>
        </div>
      </div>
    </div>
  );
}
