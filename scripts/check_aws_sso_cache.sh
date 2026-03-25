#!/bin/bash

# AWS SSO 缓存检查脚本
# 用于查找和显示 AWS SSO 认证信息

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
GRAY='\033[0;37m'
NC='\033[0m' # No Color

echo -e "${CYAN}========================================${NC}"
echo -e "${CYAN}AWS SSO 缓存文件检查工具${NC}"
echo -e "${CYAN}========================================${NC}\n"

# 定义缓存目录路径
SSO_CACHE="$HOME/.aws/sso/cache"
CLI_CACHE="$HOME/.aws/cli/cache"

echo -e "${YELLOW}检查目录：${NC}"
echo -e "  SSO Cache: $SSO_CACHE"
echo -e "  CLI Cache: $CLI_CACHE\n"

# 检查 SSO 缓存目录
if [ -d "$SSO_CACHE" ]; then
    echo -e "${GREEN}=== SSO Cache 文件 ===${NC}"
    
    shopt -s nullglob
    files=("$SSO_CACHE"/*.json)
    
    if [ ${#files[@]} -eq 0 ]; then
        echo -e "  ${RED}未找到缓存文件${NC}"
    else
        for file in "${files[@]}"; do
            echo -e "\n${CYAN}文件: $(basename "$file")${NC}"
            echo -e "${GRAY}修改时间: $(date -r "$file" 2>/dev/null || stat -f "%Sm" "$file" 2>/dev/null)${NC}"
            
            if command -v jq &> /dev/null; then
                # 使用 jq 美化输出
                echo -e "\n${YELLOW}关键字段：${NC}"
                
                if jq -e '.accessToken' "$file" &> /dev/null; then
                    TOKEN=$(jq -r '.accessToken' "$file" | cut -c1-50)
                    echo -e "  ${GREEN}✓ accessToken: ${TOKEN}...${NC}"
                fi
                
                if jq -e '.refreshToken' "$file" &> /dev/null; then
                    TOKEN=$(jq -r '.refreshToken' "$file" | cut -c1-50)
                    echo -e "  ${GREEN}✓ refreshToken: ${TOKEN}...${NC}"
                fi
                
                if jq -e '.clientId' "$file" &> /dev/null; then
                    CLIENT_ID=$(jq -r '.clientId' "$file")
                    echo -e "  ${GREEN}✓ clientId: ${CLIENT_ID}${NC}"
                fi
                
                if jq -e '.clientSecret' "$file" &> /dev/null; then
                    SECRET=$(jq -r '.clientSecret' "$file" | cut -c1-20)
                    echo -e "  ${GREEN}✓ clientSecret: ${SECRET}...${NC}"
                fi
                
                if jq -e '.expiresAt' "$file" &> /dev/null; then
                    EXPIRES=$(jq -r '.expiresAt' "$file")
                    echo -e "  ${CYAN}⏰ expiresAt: ${EXPIRES}${NC}"
                    
                    # 检查是否过期（简单比较）
                    EXPIRES_TS=$(date -d "$EXPIRES" +%s 2>/dev/null || date -j -f "%Y-%m-%dT%H:%M:%SZ" "$EXPIRES" +%s 2>/dev/null)
                    NOW_TS=$(date +%s)
                    
                    if [ ! -z "$EXPIRES_TS" ]; then
                        if [ $EXPIRES_TS -lt $NOW_TS ]; then
                            echo -e "    ${RED}⚠️  已过期！${NC}"
                        else
                            DIFF=$((EXPIRES_TS - NOW_TS))
                            HOURS=$((DIFF / 3600))
                            MINUTES=$(((DIFF % 3600) / 60))
                            echo -e "    ${GREEN}✓ 剩余时间: ${HOURS}小时 ${MINUTES}分钟${NC}"
                        fi
                    fi
                fi
                
                if jq -e '.region' "$file" &> /dev/null; then
                    REGION=$(jq -r '.region' "$file")
                    echo -e "  ${CYAN}🌍 region: ${REGION}${NC}"
                fi
                
                if jq -e '.startUrl' "$file" &> /dev/null; then
                    START_URL=$(jq -r '.startUrl' "$file")
                    echo -e "  ${CYAN}🔗 startUrl: ${START_URL}${NC}"
                fi
                
                echo -e "\n${YELLOW}所有字段：${NC}"
                jq -r 'keys[]' "$file" | while read key; do
                    echo -e "  ${GRAY}- $key${NC}"
                done
                
                echo -e "\n${YELLOW}完整内容（JSON）：${NC}"
                echo -e "${GRAY}$(jq '.' "$file")${NC}"
            else
                # 没有 jq，直接显示内容
                echo -e "\n${YELLOW}文件内容：${NC}"
                echo -e "${GRAY}$(cat "$file")${NC}"
                echo -e "\n${YELLOW}提示: 安装 jq 可以获得更好的显示效果${NC}"
            fi
            
            echo -e "\n$(printf '=%.0s' {1..60})"
        done
    fi
else
    echo -e "${RED}❌ SSO Cache 目录不存在: $SSO_CACHE${NC}"
fi

# 检查 CLI 缓存目录
echo -e "\n${GREEN}=== CLI Cache 文件 ===${NC}"
if [ -d "$CLI_CACHE" ]; then
    shopt -s nullglob
    files=("$CLI_CACHE"/*.json)
    
    if [ ${#files[@]} -eq 0 ]; then
        echo -e "  ${RED}未找到缓存文件${NC}"
    else
        for file in "${files[@]}"; do
            echo -e "\n${CYAN}文件: $(basename "$file")${NC}"
            echo -e "${GRAY}修改时间: $(date -r "$file" 2>/dev/null || stat -f "%Sm" "$file" 2>/dev/null)${NC}"
            
            if command -v jq &> /dev/null; then
                echo -e "${GRAY}$(jq '.' "$file")${NC}"
            else
                echo -e "${GRAY}$(cat "$file")${NC}"
            fi
            
            echo -e "\n$(printf '=%.0s' {1..60})"
        done
    fi
else
    echo -e "${RED}❌ CLI Cache 目录不存在: $CLI_CACHE${NC}"
fi

# 检查 AWS 配置文件
echo -e "\n${GREEN}=== AWS 配置文件 ===${NC}"
AWS_CONFIG="$HOME/.aws/config"
AWS_CREDENTIALS="$HOME/.aws/credentials"

if [ -f "$AWS_CONFIG" ]; then
    echo -e "\n${CYAN}Config 文件内容:${NC}"
    echo -e "${GRAY}$(cat "$AWS_CONFIG")${NC}"
else
    echo -e "  ${RED}未找到 config 文件${NC}"
fi

if [ -f "$AWS_CREDENTIALS" ]; then
    echo -e "\n${CYAN}Credentials 文件内容:${NC}"
    echo -e "${GRAY}$(cat "$AWS_CREDENTIALS")${NC}"
else
    echo -e "  ${RED}未找到 credentials 文件${NC}"
fi

echo -e "\n${CYAN}========================================${NC}"
echo -e "${CYAN}检查完成！${NC}"
echo -e "${CYAN}========================================${NC}"

echo -e "\n${YELLOW}💡 提示：${NC}"
echo -e "${YELLOW}如果没有找到缓存文件，请先执行：${NC}"
echo -e "  ${NC}aws sso login --profile your-profile${NC}"
echo -e "\n${YELLOW}如果需要配置 SSO，请执行：${NC}"
echo -e "  ${NC}aws configure sso${NC}"
