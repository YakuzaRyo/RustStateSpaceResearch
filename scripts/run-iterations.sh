#!/bin/bash
# Run Iterations - 迭代控制主入口
# 按方向顺序遍历，每个方向完成所有步骤后评估得分

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
LIB_DIR="$SCRIPT_DIR/lib"

cd "$PROJECT_DIR"

# 解析参数
MAX_ROUNDS=3
SKIP_EVAL=false
DRY_RUN=false

usage() {
    echo "Usage: $0 [options]"
    echo ""
    echo "Options:"
    echo "  --rounds <n>      最大轮数 (default: 3)"
    echo "  --skip-eval      跳过评估步骤"
    echo "  --dry-run        模拟运行，不实际调用 Claude"
    echo "  -h, --help       显示帮助"
    exit 1
}

while [[ $# -gt 0 ]]; do
    case $1 in
        --rounds)
            MAX_ROUNDS="$2"
            shift 2
            ;;
        --skip-eval)
            SKIP_EVAL=true
            shift
            ;;
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        -h|--help)
            usage
            ;;
        *)
            echo "Unknown option: $1"
            usage
            ;;
    esac
done

echo "=========================================="
echo "Research Iteration Runner"
echo "=========================================="
echo "Max rounds: $MAX_ROUNDS"
echo "Skip eval: $SKIP_EVAL"
echo "Dry run: $DRY_RUN"
echo "=========================================="

# 初始化日志目录
mkdir -p "$PROJECT_DIR/logs"

# 步骤列表
STEPS=("step1_priority" "step2_literature" "step3_hypotheses" "step4_verified" "step5_code")

# 辅助函数：运行研究
run_research() {
    local direction_id=$1
    local step=$2

    if [ "$DRY_RUN" = true ]; then
        echo "[DRY RUN] Would run research for direction $direction_id, step $step"
        return 0
    fi

    "$SCRIPT_DIR/claude-research.sh" --direction "$direction_id" --step "$step"
    return $?
}

# 辅助函数：运行评估
run_evaluate() {
    if [ "$SKIP_EVAL" = true ]; then
        echo "[SKIP] Evaluation skipped"
        return 0
    fi

    if [ "$DRY_RUN" = true ]; then
        echo "[DRY RUN] Would run evaluation"
        return 0
    fi

    python3 "$SCRIPT_DIR/evaluate.py" .
    return $?
}

# 辅助函数：获取方向当前步骤
get_current_step() {
    local direction_id=$1

    python3 << EOF
import sys
sys.path.insert(0, '$LIB_DIR')
from research_state import ResearchState

state = ResearchState('$PROJECT_DIR')
step = state.get_current_step('$direction_id')
print(step)
EOF
}

# 辅助函数：检查方向是否完成
is_direction_complete() {
    local direction_id=$1

    python3 << EOF
import sys
sys.path.insert(0, '$LIB_DIR')
from research_state import ResearchState

state = ResearchState('$PROJECT_DIR')
complete = state.is_direction_complete('$direction_id')
print("true" if complete else "false")
EOF
}

# 辅助函数：获取方向列表（按优先级）
get_directions() {
    python3 << EOF
import sys
sys.path.insert(0, '$LIB_DIR')
from research_state import ResearchState

state = ResearchState('$PROJECT_DIR')
directions = state.get_directions_by_priority()
for d in directions:
    print(f"{d['id']}:{d['direction_name']}")
EOF
}

# 辅助函数：记录评分
record_score() {
    local direction_id=$1
    local score=$2
    local round=$3

    python3 << EOF
import sys
sys.path.insert(0, '$LIB_DIR')
from research_state import ResearchState

state = ResearchState('$PROJECT_DIR')
state.record_score('$direction_id', $score, $round)
print(f"Recorded score $score for direction $direction_id")
EOF
}

# 辅助函数：获取评分
get_score() {
    local direction_id=$1

    python3 << EOF
import sys
sys.path.insert(0, '$LIB_DIR')
from research_state import ResearchState

state = ResearchState('$PROJECT_DIR')
plan = state.load_plan()
for d in plan.get('research_directions', []):
    if d['id'] == '$direction_id':
        # 重新评估
        import subprocess
        result = subprocess.run(
            ['python3', '$SCRIPT_DIR/evaluate.py', '$PROJECT_DIR'],
            capture_output=True,
            text=True
        )
        # 从 results.tsv 读取
        import re
        for line in result.stdout.split('\n'):
            if '$direction_id' in line or '核心原则' in line or '实现技术' in line:
                match = re.search(r'(\d+)/250', line)
                if match:
                    print(match.group(1))
                    break
        break
EOF
}

# 主循环
for round in $(seq 1 $MAX_ROUNDS); do
    echo ""
    echo "=========================================="
    echo "第 $round 轮研究"
    echo "=========================================="

    # 获取方向列表
    while IFS=: read -r dir_id dir_name; do
        echo ""
        echo "--- 研究方向: $dir_id - $dir_name ---"

        # 当前方向必须完成所有步骤
        step_index=0
        while [ $step_index -lt ${#STEPS[@]} ]; do
            step="${STEPS[$step_index]}"
            current_step=$(get_current_step "$dir_id")

            # 检查是否需要执行当前步骤
            needs_run=false
            for i in $(seq $step_index ${#STEPS[@]}); do
                if [ "$step" = "${STEPS[$i]}" ]; then
                    needs_run=true
                    break
                fi
            done

            if [ "$needs_run" = false ]; then
                step_index=$((step_index + 1))
                continue
            fi

            echo "执行步骤: $step"
            run_research "$dir_id" "$step"

            # 检查步骤是否完成
            complete=$(is_direction_complete "$dir_id")
            if [ "$complete" = "true" ]; then
                echo "方向 $dir_id 已完成"
                break
            fi

            step_index=$((step_index + 1))
        done

        # 方向完成，评估得分
        echo ""
        echo "评估方向 $dir_id..."
        run_evaluate

        # 提取得分（从 results.tsv）
        if [ "$DRY_RUN" = false ]; then
            score=$(python3 << EOF
import re
with open('$PROJECT_DIR/results.tsv', 'r') as f:
    for line in f:
        if '$dir_id' in line or '$dir_name' in line:
            match = re.search(r'(\d+)\s*$', line.strip())
            if match:
                print(match.group(1))
                break
EOF
)
            last_score=$(python3 -c "
import sys
sys.path.insert(0, '$LIB_DIR')
from research_state import ResearchState
state = ResearchState('$PROJECT_DIR')
print(state.get_last_score('$dir_id'))
")
            echo "当前得分: $score, 上次得分: $last_score"

            # 记录评分
            record_score "$dir_id" "$score" "$round"

            # 比较分数
            if [ "$score" -ge "$last_score" ]; then
                echo "✓ 得分 >= 上次，提交 GitHub"

                # Git 提交
                if [ -d ".git" ]; then
                    git add directions/ framework/ verification/ logs/
                    git commit -m "研究方向 $dir_id: $dir_name 得分 $score" 2>/dev/null || true

                    # 尝试创建 PR（如果 gh 可用）
                    if command -v gh &> /dev/null; then
                        gh pr create --title "研究方向 $dir_id: $dir_name" \
                            --body "得分: $score" 2>/dev/null || true
                    fi
                fi
            else
                echo "✗ 得分 < 上次，继续研究"
            fi
        fi

    done < <(get_directions)

    echo ""
    echo "=========================================="
    echo "第 $round 轮结束"
    echo "=========================================="
done

echo ""
echo "所有轮次完成！"
