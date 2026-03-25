# AWS SSO 缓存检查脚本
# 用于查找和显示 AWS SSO 认证信息

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "AWS SSO 缓存文件检查工具" -ForegroundColor Cyan
Write-Host "========================================`n" -ForegroundColor Cyan

# 定义缓存目录路径
$ssoCache = Join-Path $env:USERPROFILE ".aws\sso\cache"
$cliCache = Join-Path $env:USERPROFILE ".aws\cli\cache"

Write-Host "检查目录：" -ForegroundColor Yellow
Write-Host "  SSO Cache: $ssoCache"
Write-Host "  CLI Cache: $cliCache`n"

# 检查 SSO 缓存目录
if (Test-Path $ssoCache) {
    Write-Host "=== SSO Cache 文件 ===" -ForegroundColor Green
    $ssoFiles = Get-ChildItem -Path $ssoCache -Filter "*.json" -ErrorAction SilentlyContinue
    
    if ($ssoFiles.Count -eq 0) {
        Write-Host "  未找到缓存文件" -ForegroundColor Red
    } else {
        foreach ($file in $ssoFiles) {
            Write-Host "`n文件: $($file.Name)" -ForegroundColor Cyan
            Write-Host "修改时间: $($file.LastWriteTime)" -ForegroundColor Gray
            
            try {
                $content = Get-Content $file.FullName -Raw | ConvertFrom-Json
                
                # 显示关键字段
                Write-Host "`n关键字段：" -ForegroundColor Yellow
                
                if ($content.accessToken) {
                    $tokenPreview = $content.accessToken.Substring(0, [Math]::Min(50, $content.accessToken.Length))
                    Write-Host "  ✓ accessToken: $tokenPreview..." -ForegroundColor Green
                }
                
                if ($content.refreshToken) {
                    $tokenPreview = $content.refreshToken.Substring(0, [Math]::Min(50, $content.refreshToken.Length))
                    Write-Host "  ✓ refreshToken: $tokenPreview..." -ForegroundColor Green
                }
                
                if ($content.clientId) {
                    Write-Host "  ✓ clientId: $($content.clientId)" -ForegroundColor Green
                }
                
                if ($content.clientSecret) {
                    $secretPreview = $content.clientSecret.Substring(0, [Math]::Min(20, $content.clientSecret.Length))
                    Write-Host "  ✓ clientSecret: $secretPreview..." -ForegroundColor Green
                }
                
                if ($content.expiresAt) {
                    Write-Host "  ⏰ expiresAt: $($content.expiresAt)" -ForegroundColor Cyan
                    
                    # 检查是否过期
                    $expiryDate = [DateTime]::Parse($content.expiresAt)
                    if ($expiryDate -lt (Get-Date)) {
                        Write-Host "    ⚠️  已过期！" -ForegroundColor Red
                    } else {
                        $timeLeft = $expiryDate - (Get-Date)
                        Write-Host "    ✓ 剩余时间: $($timeLeft.Hours)小时 $($timeLeft.Minutes)分钟" -ForegroundColor Green
                    }
                }
                
                if ($content.region) {
                    Write-Host "  🌍 region: $($content.region)" -ForegroundColor Cyan
                }
                
                if ($content.startUrl) {
                    Write-Host "  🔗 startUrl: $($content.startUrl)" -ForegroundColor Cyan
                }
                
                # 显示所有字段名
                Write-Host "`n所有字段：" -ForegroundColor Yellow
                $content.PSObject.Properties | ForEach-Object {
                    Write-Host "  - $($_.Name)" -ForegroundColor Gray
                }
                
                Write-Host "`n完整内容（JSON）：" -ForegroundColor Yellow
                Write-Host ($content | ConvertTo-Json -Depth 10) -ForegroundColor Gray
                
            } catch {
                Write-Host "  ❌ 无法解析 JSON: $_" -ForegroundColor Red
            }
            
            Write-Host "`n" + ("=" * 60)
        }
    }
} else {
    Write-Host "❌ SSO Cache 目录不存在: $ssoCache" -ForegroundColor Red
}

# 检查 CLI 缓存目录
Write-Host "`n=== CLI Cache 文件 ===" -ForegroundColor Green
if (Test-Path $cliCache) {
    $cliFiles = Get-ChildItem -Path $cliCache -Filter "*.json" -ErrorAction SilentlyContinue
    
    if ($cliFiles.Count -eq 0) {
        Write-Host "  未找到缓存文件" -ForegroundColor Red
    } else {
        foreach ($file in $cliFiles) {
            Write-Host "`n文件: $($file.Name)" -ForegroundColor Cyan
            Write-Host "修改时间: $($file.LastWriteTime)" -ForegroundColor Gray
            
            try {
                $content = Get-Content $file.FullName -Raw | ConvertFrom-Json
                Write-Host ($content | ConvertTo-Json -Depth 10) -ForegroundColor Gray
            } catch {
                Write-Host "  ❌ 无法解析 JSON: $_" -ForegroundColor Red
            }
            
            Write-Host "`n" + ("=" * 60)
        }
    }
} else {
    Write-Host "❌ CLI Cache 目录不存在: $cliCache" -ForegroundColor Red
}

# 检查 AWS 配置文件
Write-Host "`n=== AWS 配置文件 ===" -ForegroundColor Green
$awsConfig = Join-Path $env:USERPROFILE ".aws\config"
$awsCredentials = Join-Path $env:USERPROFILE ".aws\credentials"

if (Test-Path $awsConfig) {
    Write-Host "`nConfig 文件内容:" -ForegroundColor Cyan
    Get-Content $awsConfig | Write-Host -ForegroundColor Gray
} else {
    Write-Host "  未找到 config 文件" -ForegroundColor Red
}

if (Test-Path $awsCredentials) {
    Write-Host "`nCredentials 文件内容:" -ForegroundColor Cyan
    Get-Content $awsCredentials | Write-Host -ForegroundColor Gray
} else {
    Write-Host "  未找到 credentials 文件" -ForegroundColor Red
}

Write-Host "`n========================================" -ForegroundColor Cyan
Write-Host "检查完成！" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan

Write-Host "`n💡 提示：" -ForegroundColor Yellow
Write-Host "如果没有找到缓存文件，请先执行：" -ForegroundColor Yellow
Write-Host "  aws sso login --profile your-profile" -ForegroundColor White
Write-Host "`n如果需要配置 SSO，请执行：" -ForegroundColor Yellow
Write-Host "  aws configure sso" -ForegroundColor White
