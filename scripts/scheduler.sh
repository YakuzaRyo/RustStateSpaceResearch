#!/bin/bash
# Research Scheduler - 定时任务脚本
# 每 20 分钟运行一轮研究

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_DIR"

# 日志文件
LOG_FILE="$PROJECT_DIR/logs/cron.log"

# 记录开始
echo "$(date '+%Y-%m-%d %H:%M:%S') - Scheduler: Starting research iteration..." >> "$LOG_FILE"

# 运行一轮研究
./scripts/run-iterations.sh --rounds 1 >> "$LOG_FILE" 2>&1

# 记录结束
echo "$(date '+%Y-%m-%d %H:%M:%S') - Scheduler: Research iteration completed" >> "$LOG_FILE"
