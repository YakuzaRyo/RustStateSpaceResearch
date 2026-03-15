#!/bin/bash
# Research Scheduler - 定时任务脚本
# 每 20 分钟运行一轮研究

# 使用绝对路径，确保 cron 环境下正常工作
PROJECT_DIR="/home/ume/RustStateSpaceResearch"
cd "$PROJECT_DIR"

# 日志文件
LOG_FILE="$PROJECT_DIR/logs/cron.log"

# 记录开始
echo "$(date '+%Y-%m-%d %H:%M:%S') - Scheduler: Starting research iteration..." >> "$LOG_FILE"

# 运行一轮研究
./scripts/run-iterations.sh --rounds 1 >> "$LOG_FILE" 2>&1

# 记录结束
echo "$(date '+%Y-%m-%d %H:%M:%S') - Scheduler: Research iteration completed" >> "$LOG_FILE"
