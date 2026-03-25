# 代理问题说明

## 问题描述

使用 **TUN 模式代理** 时，IMAP OAuth2 认证会失败，错误信息：

```
[IMAP] Authentication failed: ConnectionLost
```

## 原因分析

### 1. TUN 模式的工作原理

TUN 模式代理会创建虚拟网络接口，所有流量都通过这个接口路由。从日志可以看到：

```
addr: 198.18.0.1:54408, peer: 198.18.0.6:993
```

`198.18.0.0/15` 是 TUN/TAP 虚拟网络的地址段。

### 2. 为什么会失败

IMAP 使用 **SASL (Simple Authentication and Security Layer)** 进行认证，这是一个多步骤的握手过程：

1. 客户端发送 `AUTHENTICATE XOAUTH2`
2. 服务器返回挑战（challenge）
3. 客户端发送认证字符串
4. 服务器验证并返回结果

**TUN 模式代理可能会：**
- 干扰 SASL 握手过程
- 修改或延迟数据包
- 导致认证超时或连接断开

### 3. 为什么 Graph API 可以工作

Graph API 使用标准的 HTTPS (443端口)，代理对 HTTPS 的支持更成熟：
- 使用 HTTP CONNECT 隧道
- 不会干扰应用层协议
- 更稳定可靠

## 解决方案

### 方案 1: 临时关闭代理（最简单）

**步骤：**
1. 关闭 TUN 模式代理
2. 测试 IMAP 连接
3. 如果成功，确认是代理问题

**优点：**
- 最快速的验证方法
- 100% 确定是否是代理问题

**缺点：**
- 需要临时关闭代理

### 方案 2: 配置代理直连规则（推荐）

**步骤：**

在代理软件中添加以下域名的直连规则：

```
# Microsoft 认证和邮件服务
outlook.office365.com
*.office365.com
*.outlook.com
login.microsoftonline.com
login.live.com
```

**常见代理软件配置：**

#### Clash
编辑配置文件，在 `rules` 部分添加：
```yaml
rules:
  - DOMAIN-SUFFIX,outlook.office365.com,DIRECT
  - DOMAIN-SUFFIX,office365.com,DIRECT
  - DOMAIN-SUFFIX,outlook.com,DIRECT
  - DOMAIN-SUFFIX,microsoftonline.com,DIRECT
  - DOMAIN-SUFFIX,live.com,DIRECT
```

#### V2Ray
在路由规则中添加：
```json
{
  "routing": {
    "rules": [
      {
        "type": "field",
        "domain": [
          "outlook.office365.com",
          "office365.com",
          "outlook.com",
          "microsoftonline.com",
          "live.com"
        ],
        "outboundTag": "direct"
      }
    ]
  }
}
```

#### Surge
在配置文件中添加：
```
[Rule]
DOMAIN-SUFFIX,outlook.office365.com,DIRECT
DOMAIN-SUFFIX,office365.com,DIRECT
DOMAIN-SUFFIX,outlook.com,DIRECT
DOMAIN-SUFFIX,microsoftonline.com,DIRECT
DOMAIN-SUFFIX,live.com,DIRECT
```

**优点：**
- 不需要关闭代理
- 只对特定域名直连
- 其他流量仍走代理

**缺点：**
- 需要修改代理配置
- 不同代理软件配置方式不同

### 方案 3: 使用 Graph API 模式（最稳定）

**步骤：**
1. 在系统设置中选择 "Microsoft Graph API" 模式
2. 使用 Graph API 获取验证码

**优点：**
- 不受代理影响
- 速度更快
- 更稳定

**缺点：**
- 需要不同的 refresh_token scope
- 如果已经有 IMAP token，需要重新生成

### 方案 4: 使用 HTTP 代理而不是 TUN 模式

**步骤：**
1. 将代理模式从 TUN 改为 HTTP/SOCKS5
2. 配置系统代理
3. 应用会自动使用系统代理

**优点：**
- HTTP 代理对应用层协议更友好
- 不会干扰 SASL 认证

**缺点：**
- 需要改变代理模式
- 可能影响其他应用

## 验证方法

### 1. 检查是否使用了代理

从日志中查看连接地址：

```
[IMAP] Connecting to outlook.office365.com:993
addr: 198.18.0.1:54408, peer: 198.18.0.6:993
```

如果看到 `198.18.x.x` 地址，说明使用了 TUN 代理。

正常直连应该显示：
```
addr: 192.168.x.x:xxxxx, peer: 40.97.x.x:993
```
（40.97.x.x 是 Microsoft 的真实 IP）

### 2. 测试直连

**Windows:**
```powershell
# 临时禁用代理
$env:NO_PROXY = "outlook.office365.com,*.office365.com"

# 测试连接
Test-NetConnection outlook.office365.com -Port 993
```

**Linux/Mac:**
```bash
# 临时禁用代理
export NO_PROXY="outlook.office365.com,*.office365.com"

# 测试连接
telnet outlook.office365.com 993
```

### 3. 对比测试

| 测试场景 | 结果 |
|---------|------|
| 代理开启 + IMAP | ❌ 失败 |
| 代理关闭 + IMAP | ✅ 成功 |
| 代理开启 + Graph API | ✅ 成功 |
| 直连规则 + IMAP | ✅ 成功 |

## 技术细节

### IMAP SASL 认证流程

```
Client: A001 AUTHENTICATE XOAUTH2
Server: +
Client: <base64_encoded_auth_string>
Server: A001 OK AUTHENTICATE completed
```

**TUN 代理可能的干扰点：**
1. 延迟 `+` 响应
2. 修改数据包内容
3. 中断连接
4. 超时

### Graph API 认证流程

```
Client: POST https://graph.microsoft.com/v1.0/me/messages
        Authorization: Bearer <access_token>
Server: 200 OK
        { "value": [...] }
```

**为什么不受影响：**
1. 标准 HTTPS 协议
2. 单次请求/响应
3. 代理使用 HTTP CONNECT 隧道
4. 不涉及复杂的握手

## 推荐配置

### 最佳实践

1. **开发/测试环境：**
   - 关闭代理或使用直连规则
   - 使用 Graph API 模式

2. **生产环境：**
   - 配置代理直连规则
   - 优先使用 Graph API 模式
   - IMAP 作为备用

3. **代理配置：**
   ```
   # 直连规则
   outlook.office365.com
   *.office365.com
   *.outlook.com
   login.microsoftonline.com
   
   # 或使用 IP 段（Microsoft Azure）
   40.96.0.0/13
   52.96.0.0/14
   ```

### 性能对比

| 模式 | 延迟 | 稳定性 | 代理兼容性 |
|------|------|--------|-----------|
| Graph API | 低 | 高 | 好 |
| IMAP (直连) | 中 | 高 | N/A |
| IMAP (TUN代理) | 高 | 低 | 差 |
| IMAP (HTTP代理) | 中 | 中 | 中 |

## 常见问题

### Q: 为什么 server.js 可以工作？

**A:** Node.js 的网络库可能对代理的处理方式不同，或者：
- 使用了不同的 IMAP 库（imapflow）
- 有更好的错误恢复机制
- 超时设置不同

### Q: 可以在代码中绕过代理吗？

**A:** 理论上可以，但：
- 需要修改底层网络库
- 可能影响其他功能
- 不如配置代理规则简单

### Q: 为什么不自动检测代理？

**A:** 
- 代理检测不可靠
- 用户可能有特殊需求
- 最好由用户明确配置

## 总结

**如果你使用 TUN 模式代理：**

1. ✅ **推荐：** 使用 Graph API 模式
2. ✅ **推荐：** 配置代理直连规则
3. ⚠️ **可选：** 临时关闭代理测试
4. ❌ **不推荐：** 继续使用 IMAP + TUN 代理

**最简单的解决方案：**
在系统设置中切换到 "Microsoft Graph API" 模式，这样就不会受代理影响了。
