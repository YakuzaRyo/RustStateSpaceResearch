#!/bin/bash
# Claude Research Runner
# 单次研究调用脚本，支持指定方向和步骤

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
LIB_DIR="$SCRIPT_DIR/lib"

cd "$PROJECT_DIR"

# 解析参数
DIRECTION=""
STEP=""
SESSION_ID=""

usage() {
    echo "Usage: $0 --direction <id> --step <step>"
    echo "  --direction <id>   研究方向 ID (01-06)"
    echo "  --step <step>      步骤 (step1_priority-step5_code)"
    echo ""
    echo "Examples:"
    echo "  $0 --direction 01 --step step2_literature"
    echo "  $0 --direction 03 --step step3_hypotheses"
    exit 1
}

while [[ $# -gt 0 ]]; do
    case $1 in
        --direction)
            DIRECTION="$2"
            shift 2
            ;;
        --step)
            STEP="$2"
            shift 2
            ;;
        --session-id)
            SESSION_ID="$2"
            shift 2
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

# 验证参数
if [ -z "$DIRECTION" ] || [ -z "$STEP" ]; then
    echo "Error: --direction and --step are required"
    usage
fi

echo "=========================================="
echo "Claude Research Runner"
echo "=========================================="
echo "Direction: $DIRECTION"
echo "Step: $STEP"
echo "Session: ${SESSION_ID:-new}"
echo "=========================================="

# 加载方向信息
DIRECTION_FILE=$(ls "$PROJECT_DIR/directions/${DIRECTION}_"*.json 2>/dev/null | head -1)

if [ -z "$DIRECTION_FILE" ]; then
    echo "Error: Direction file not found for $DIRECTION"
    exit 1
fi

DIRECTION_NAME=$(python3 -c "import json; d=json.load(open('$DIRECTION_FILE')); print(d.get('direction_name', 'Unknown'))")
QUESTION=$(python3 -c "import json; d=json.load(open('$DIRECTION_FILE')); print(d.get('question', ''))")

echo "研究方向: $DIRECTION_NAME"
echo "研究问题: $QUESTION"
echo ""

# 构建 prompt
PROMPT=$(python3 << EOF
import sys
sys.path.insert(0, '$LIB_DIR')
from prompt_builder import PromptBuilder
from research_state import ResearchState

state = ResearchState('$PROJECT_DIR')
direction = state.load_direction('$DIRECTION')

builder = PromptBuilder('$PROJECT_DIR')
print(builder.build_research_prompt(direction, '$STEP'))
EOF
)

# 调用 Claude CLI
echo "Calling Claude Code..."

if [ -n "$SESSION_ID" ]; then
    claude -p "$PROMPT" --session-id "$SESSION_ID" --dangerously-skip-permissions
else
    claude -p "$PROMPT" --no-session-persistence --dangerously-skip-permissions
fi

echo ""
echo "=========================================="
echo "Research completed for $DIRECTION_NAME"
echo "=========================================="

# 更新状态
python3 << EOF
import sys
sys.path.insert(0, '$LIB_DIR')
from research_state import ResearchState

state = ResearchState('$PROJECT_DIR')
state.update_direction_status('$DIRECTION', '$STEP')
print(f"Updated direction $DIRECTION status to $STEP")
EOF
