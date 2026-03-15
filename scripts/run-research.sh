#!/bin/bash
# State Space Research Runner

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
cd "$PROJECT_DIR"

echo "=========================================="
echo "状态空间研究流程"
echo "=========================================="

# Load current state
CURRENT_DIRECTION=$(python3 -c "
import json
with open('plan.json') as f:
    plan = json.load(f)
    print(plan.get('current_direction_id', '01'))
")

STEP=$(python3 -c "
import json
with open('plan.json') as f:
    plan = json.load(f)
    print(plan.get('current_step', 'step1_priority'))
")

echo "当前方向: $CURRENT_DIRECTION"
echo "当前步骤: $STEP"

# Step 1: Priority ordering
if [ "$STEP" = "step1_priority" ]; then
    echo "执行 Step 1: 优先级排序"
    python3 << 'EOF'
import json

with open('plan.json') as f:
    plan = json.load(f)

directions = plan.get('research_directions', [])
# Sort by priority
directions_sorted = sorted(directions, key=lambda x: x.get('priority', 6))

for i, d in enumerate(directions_sorted, 1):
    d['priority'] = i

plan['research_directions'] = directions_sorted

with open('plan.json', 'w') as f:
    json.dump(plan, f, indent=2, ensure_ascii=False)

print("优先级已更新")
for d in directions_sorted:
    print(f"  {d['id']}: {d['direction_name']} (优先级 {d['priority']})")
EOF
    STEP="step2_literature"
fi

# Step 2: Literature research
if [ "$STEP" = "step2_literature" ]; then
    echo "执行 Step 2: 文献调研"
    echo "请使用 WebSearch 进行文献调研..."
    echo "完成后运行: update_direction.py --literature <数量>"
    STEP="step3_hypotheses"
fi

# Step 3: Generate hypotheses
if [ "$STEP" = "step3_hypotheses" ]; then
    echo "执行 Step 3: 提出假设"
    echo "请基于文献调研提出假设..."
    echo "完成后运行: update_direction.py --hypothesis <假设内容>"
    STEP="step4_verified"
fi

# Step 4: Verify hypotheses
if [ "$STEP" = "step4_verified" ]; then
    echo "执行 Step 4: 验证假设"
    echo "请编写验证代码到 verification/ 目录..."
    echo "完成后运行: update_direction.py --verified <假设ID>"
    STEP="step5_code"
fi

# Step 5: Code implementation
if [ "$STEP" = "step5_code" ]; then
    echo "执行 Step 5: 代码实现"
    echo "请编写代码到 framework/ 或 draft/ 目录..."
    echo "完成后运行: evaluate.py . 查看得分"
fi

# Update plan.json with current step
python3 << EOF
import json

with open('plan.json') as f:
    plan = json.load(f)

plan['current_step'] = '$STEP'

with open('plan.json', 'w') as f:
    json.dump(plan, f, indent=2, ensure_ascii=False)
EOF

echo ""
echo "当前步骤完成: $STEP"
echo "运行 evaluate.py . 查看当前得分"
