# 开发脚本和工具

这个目录包含开发和调试过程中使用的辅助脚本。

## 脚本说明

### AWS SSO 缓存检查工具

**check_aws_sso_cache.ps1** (Windows PowerShell)
- 检查 AWS SSO 缓存文件
- 显示 accessToken、refreshToken、clientId 等信息
- 检查 token 过期时间
- 查看 AWS 配置文件

**check_aws_sso_cache.sh** (Linux/macOS Bash)
- 功能同上，适用于 Unix 系统
- 支持 jq 美化 JSON 输出

使用方法：
```bash
# Windows
powershell -ExecutionPolicy Bypass -File check_aws_sso_cache.ps1

# Linux/macOS
bash check_aws_sso_cache.sh
```

### 数据库查找工具

**find_database.ps1** (Windows PowerShell)
- 查找 Tauri 应用的 SQLite 数据库位置
- 搜索所有可能的数据目录
- 显示数据库文件信息
- 快速打开数据库目录

使用方法：
```bash
powershell -ExecutionPolicy Bypass -File find_database.ps1
```

### 代码示例

**demo.rs**
- Kiro Auth Service 客户端示例代码
- 演示 OAuth 2.0 PKCE 流程
- 包含 login、create_token、refresh_token 方法
- 可作为参考实现

### 测试数据

**test_data.xlsx**
- 测试用的 Excel 数据文件
- 可能包含账号导入格式示例

## 注意事项

1. 这些脚本仅用于开发和调试
2. 不要在生产环境中使用
3. 包含敏感信息的输出请妥善保管
4. PowerShell 脚本可能需要调整执行策略

## 相关文档

- [调试指南](../docs/debug/)
- [开发文档](../docs/guides/)
