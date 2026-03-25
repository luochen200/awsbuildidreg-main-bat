# 数据库文件位置

## 📍 数据库路径

### Windows 系统

```
C:\Users\{你的用户名}\AppData\Roaming\com.awsbuilder.automation\database.db
```

**当前用户的完整路径：**
```
C:\Users\Administrator\AppData\Roaming\com.awsbuilder.automation\database.db
```

### 快速访问

#### 方法 1：使用 Windows 资源管理器
1. 按 `Win + R` 打开运行对话框
2. 输入：`%APPDATA%\com.awsbuilder.automation`
3. 按回车，即可打开数据库所在目录

#### 方法 2：使用 PowerShell
```powershell
# 打开数据库目录
explorer "$env:APPDATA\com.awsbuilder.automation"

# 查看数据库文件信息
Get-Item "$env:APPDATA\com.awsbuilder.automation\database.db"
```

#### 方法 3：使用脚本
运行项目根目录下的脚本：
```powershell
.\find_database.ps1
```

## 🔍 查看数据库内容

### 使用 SQLite 工具

#### 推荐工具：
1. **DB Browser for SQLite** (免费，图形界面)
   - 下载：https://sqlitebrowser.org/
   - 安装后直接打开 `database.db` 文件

2. **SQLite Studio** (免费，功能强大)
   - 下载：https://sqlitestudio.pl/

3. **VS Code 扩展**
   - 安装扩展：`SQLite Viewer` 或 `SQLite`
   - 在 VS Code 中直接打开 `.db` 文件

### 使用命令行

```powershell
# 安装 SQLite (如果还没有)
# 下载：https://www.sqlite.org/download.html

# 打开数据库
sqlite3 "$env:APPDATA\com.awsbuilder.automation\database.db"

# 查看所有表
.tables

# 查看 accounts 表结构
.schema accounts

# 查询所有账号
SELECT * FROM accounts;

# 查询设置
SELECT * FROM settings;

# 退出
.quit
```

## 📊 数据库结构

### accounts 表
```sql
CREATE TABLE accounts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    email TEXT NOT NULL UNIQUE,
    email_password TEXT NOT NULL,
    client_id TEXT NOT NULL,
    refresh_token TEXT NOT NULL,
    kiro_password TEXT,
    status TEXT NOT NULL DEFAULT 'not_registered',
    error_reason TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
```

### settings 表
```sql
CREATE TABLE settings (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    browser_mode TEXT NOT NULL DEFAULT 'background',
    email_mode TEXT NOT NULL DEFAULT 'graph_api'
);
```

## 🛠️ 常用操作

### 备份数据库
```powershell
# 复制数据库文件到当前目录
Copy-Item "$env:APPDATA\com.awsbuilder.automation\database.db" -Destination ".\database_backup_$(Get-Date -Format 'yyyyMMdd_HHmmss').db"
```

### 恢复数据库
```powershell
# 从备份恢复
Copy-Item ".\database_backup.db" -Destination "$env:APPDATA\com.awsbuilder.automation\database.db" -Force
```

### 清空数据库
```powershell
# 删除数据库文件（应用会自动重新创建）
Remove-Item "$env:APPDATA\com.awsbuilder.automation\database.db"
```

### 导出数据
```powershell
# 使用 SQLite 导出为 SQL 文件
sqlite3 "$env:APPDATA\com.awsbuilder.automation\database.db" .dump > database_export.sql

# 导出为 CSV
sqlite3 "$env:APPDATA\com.awsbuilder.automation\database.db" -csv -header "SELECT * FROM accounts;" > accounts.csv
```

## 🔐 安全提示

⚠️ **数据库包含敏感信息！**

- `email_password` - 邮箱密码
- `client_id` - OAuth 客户端 ID
- `refresh_token` - OAuth 刷新令牌
- `kiro_password` - Kiro 账号密码

**请注意：**
1. 不要分享数据库文件
2. 备份时注意加密
3. 不要提交到 Git 仓库
4. 定期清理不需要的数据

## 📝 调试技巧

### 查看最近注册的账号
```sql
SELECT email, status, kiro_password, created_at 
FROM accounts 
ORDER BY created_at DESC 
LIMIT 10;
```

### 查看失败的账号
```sql
SELECT email, error_reason, updated_at 
FROM accounts 
WHERE status = 'error' 
ORDER BY updated_at DESC;
```

### 统计各状态的账号数量
```sql
SELECT status, COUNT(*) as count 
FROM accounts 
GROUP BY status;
```

### 查看当前设置
```sql
SELECT * FROM settings;
```

## 🚀 开发模式 vs 生产模式

### 开发模式 (npm run tauri dev)
数据库位置：
```
%APPDATA%\com.awsbuilder.automation\database.db
```

### 生产模式 (编译后的 .exe)
数据库位置相同：
```
%APPDATA%\com.awsbuilder.automation\database.db
```

**注意：** 开发模式和生产模式使用**同一个数据库文件**！

## 🔄 数据库迁移

如果需要更改数据库结构，在 `src-tauri/src/database.rs` 的 `init_database` 函数中添加迁移代码：

```rust
// 示例：添加新字段
let column_exists: Result<i32, _> = conn.query_row(
    "SELECT COUNT(*) FROM pragma_table_info('accounts') WHERE name='new_field'",
    [],
    |row| row.get(0),
);

if let Ok(0) = column_exists {
    conn.execute(
        "ALTER TABLE accounts ADD COLUMN new_field TEXT",
        [],
    )?;
}
```

## 📞 需要帮助？

如果找不到数据库文件：
1. 确保应用至少运行过一次
2. 检查应用标识符是否正确（在 `tauri.conf.json` 中）
3. 运行 `find_database.ps1` 脚本搜索所有可能的位置
