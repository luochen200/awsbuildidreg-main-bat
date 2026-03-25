# 查找 Tauri 应用的数据库文件位置
# Windows 系统下的 Tauri 应用数据目录

Write-Host "正在查找数据库文件..." -ForegroundColor Cyan
Write-Host ""

# 从 tauri.conf.json 读取应用标识符
$configPath = "src-tauri\tauri.conf.json"
if (Test-Path $configPath) {
    $config = Get-Content $configPath | ConvertFrom-Json
    $identifier = $config.identifier
    $productName = $config.productName
    
    Write-Host "应用标识符: $identifier" -ForegroundColor Green
    Write-Host "产品名称: $productName" -ForegroundColor Green
    Write-Host ""
}

# Tauri 2.x 在 Windows 上的数据目录位置
# 格式: %APPDATA%\{identifier}\
$appDataDir = "$env:APPDATA\$identifier"

Write-Host "数据库可能的位置:" -ForegroundColor Yellow
Write-Host ""

# 位置 1: APPDATA (最常见)
$path1 = "$env:APPDATA\$identifier\database.db"
Write-Host "1. $path1" -ForegroundColor White
if (Test-Path $path1) {
    Write-Host "   ✅ 找到了！" -ForegroundColor Green
    Write-Host "   文件大小: $((Get-Item $path1).Length) 字节"
    Write-Host "   最后修改: $((Get-Item $path1).LastWriteTime)"
} else {
    Write-Host "   ❌ 不存在" -ForegroundColor Red
}
Write-Host ""

# 位置 2: LOCALAPPDATA
$path2 = "$env:LOCALAPPDATA\$identifier\database.db"
Write-Host "2. $path2" -ForegroundColor White
if (Test-Path $path2) {
    Write-Host "   ✅ 找到了！" -ForegroundColor Green
    Write-Host "   文件大小: $((Get-Item $path2).Length) 字节"
    Write-Host "   最后修改: $((Get-Item $path2).LastWriteTime)"
} else {
    Write-Host "   ❌ 不存在" -ForegroundColor Red
}
Write-Host ""

# 位置 3: 开发模式下可能在项目目录
$path3 = "src-tauri\target\debug\database.db"
Write-Host "3. $path3 (开发模式)" -ForegroundColor White
if (Test-Path $path3) {
    Write-Host "   ✅ 找到了！" -ForegroundColor Green
    Write-Host "   文件大小: $((Get-Item $path3).Length) 字节"
    Write-Host "   最后修改: $((Get-Item $path3).LastWriteTime)"
} else {
    Write-Host "   ❌ 不存在" -ForegroundColor Red
}
Write-Host ""

# 搜索所有可能的位置
Write-Host "正在搜索所有 database.db 文件..." -ForegroundColor Cyan
Write-Host ""

$searchPaths = @(
    "$env:APPDATA",
    "$env:LOCALAPPDATA",
    "$env:USERPROFILE\AppData",
    "src-tauri\target"
)

$foundFiles = @()
foreach ($searchPath in $searchPaths) {
    if (Test-Path $searchPath) {
        $files = Get-ChildItem -Path $searchPath -Filter "database.db" -Recurse -ErrorAction SilentlyContinue
        $foundFiles += $files
    }
}

if ($foundFiles.Count -gt 0) {
    Write-Host "找到 $($foundFiles.Count) 个 database.db 文件:" -ForegroundColor Green
    Write-Host ""
    
    foreach ($file in $foundFiles) {
        Write-Host "📁 $($file.FullName)" -ForegroundColor Yellow
        Write-Host "   大小: $($file.Length) 字节"
        Write-Host "   创建时间: $($file.CreationTime)"
        Write-Host "   最后修改: $($file.LastWriteTime)"
        Write-Host ""
    }
} else {
    Write-Host "❌ 未找到任何 database.db 文件" -ForegroundColor Red
    Write-Host ""
    Write-Host "可能的原因:" -ForegroundColor Yellow
    Write-Host "1. 应用还没有运行过（数据库还未创建）"
    Write-Host "2. 应用标识符不正确"
    Write-Host "3. 数据库文件名不是 database.db"
    Write-Host ""
    Write-Host "建议:" -ForegroundColor Cyan
    Write-Host "1. 先运行一次应用: npm run tauri dev"
    Write-Host "2. 然后再运行此脚本"
}

Write-Host ""
Write-Host "快速打开数据库目录:" -ForegroundColor Cyan
Write-Host "explorer `"$appDataDir`"" -ForegroundColor White
Write-Host ""

# 询问是否打开目录
$response = Read-Host "是否打开数据库目录? (Y/N)"
if ($response -eq "Y" -or $response -eq "y") {
    if (Test-Path $appDataDir) {
        explorer $appDataDir
    } else {
        Write-Host "目录不存在，正在创建..." -ForegroundColor Yellow
        New-Item -ItemType Directory -Path $appDataDir -Force | Out-Null
        explorer $appDataDir
    }
}
