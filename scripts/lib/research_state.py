#!/usr/bin/env python3
"""
Research State Manager
研究方向状态管理
"""

import json
from pathlib import Path
from typing import Dict, Any, List, Optional


class ResearchState:
    def __init__(self, project_dir: str):
        self.project_dir = Path(project_dir)
        self.plan_path = self.project_dir / "plan.json"
        self.directions_dir = self.project_dir / "directions"
        self.history_path = self.project_dir / "logs" / "research_history.json"
        self.log_path = self.project_dir / "logs" / "research_log.csv"

    def load_plan(self) -> Dict[str, Any]:
        """加载 plan.json"""
        with open(self.plan_path, 'r', encoding='utf-8') as f:
            return json.load(f)

    def save_plan(self, plan: Dict[str, Any]):
        """保存 plan.json"""
        with open(self.plan_path, 'w', encoding='utf-8') as f:
            json.dump(plan, f, indent=2, ensure_ascii=False)

    def load_direction(self, direction_id: str) -> Dict[str, Any]:
        """加载指定方向的文件"""
        direction_file = self.directions_dir / f"{direction_id}_core_principles.json"
        if not direction_file.exists():
            # 尝试其他命名方式
            for f in self.directions_dir.glob(f"{direction_id}_*.json"):
                direction_file = f
                break

        with open(direction_file, 'r', encoding='utf-8') as f:
            return json.load(f)

    def save_direction(self, direction: Dict[str, Any]):
        """保存方向文件"""
        direction_id = direction['id']
        direction_file = self.directions_dir / f"{direction_id}_core_principles.json"

        # 查找实际文件名
        for f in self.directions_dir.glob(f"{direction_id}_*.json"):
            direction_file = f
            break

        with open(direction_file, 'w', encoding='utf-8') as f:
            json.dump(direction, f, indent=2, ensure_ascii=False)

    def get_directions_by_priority(self) -> List[Dict[str, Any]]:
        """按优先级获取所有方向"""
        plan = self.load_plan()
        directions = plan.get('research_directions', [])
        return sorted(directions, key=lambda x: x.get('priority', 6))

    def get_direction_by_id(self, direction_id: str) -> Optional[Dict[str, Any]]:
        """根据ID获取方向"""
        plan = self.load_plan()
        for d in plan.get('research_directions', []):
            if d['id'] == direction_id:
                return d
        return None

    def get_current_step(self, direction_id: str) -> str:
        """获取方向的当前步骤"""
        direction = self.load_direction(direction_id)
        steps = ['step1_priority', 'step2_literature', 'step3_hypotheses',
                 'step4_verified', 'step5_code']

        # 根据状态判断当前步骤
        status = direction.get('status', 'pending')

        # 如果literature有内容，进行到step2
        if direction.get('literature_count', 0) > 0:
            return 'step2_literature'
        # 如果有hypotheses，进行到step3
        if len(direction.get('hypotheses', [])) > 0:
            return 'step3_hypotheses'
        # 如果有verified，进行到step4
        if direction.get('verified_count', 0) > 0:
            return 'step4_verified'
        # 如果有code_features，进行到step5
        if len(direction.get('code_features', [])) > 0:
            return 'step5_code'

        return 'step1_priority'

    def is_direction_complete(self, direction_id: str) -> bool:
        """检查方向是否完成"""
        direction = self.load_direction(direction_id)
        # 完成条件：所有5个步骤都完成，且有足够的hypotheses和verified
        return (
            direction.get('literature_count', 0) > 0 and
            len(direction.get('hypotheses', [])) > 0 and
            direction.get('verified_count', 0) > 0 and
            len(direction.get('code_features', [])) > 0
        )

    def update_direction_status(self, direction_id: str, step: str):
        """更新方向的当前步骤状态"""
        direction = self.load_direction(direction_id)

        # 更新当前步骤
        direction['current_step'] = step

        # 根据步骤更新状态
        if step == 'step1_priority':
            direction['status'] = 'in_progress'
        elif step == 'step5_code':
            if self.is_direction_complete(direction_id):
                direction['status'] = 'completed'

        self.save_direction(direction)

    # 历史评分管理
    def load_history(self) -> Dict[str, Any]:
        """加载历史评分"""
        if self.history_path.exists():
            with open(self.history_path, 'r', encoding='utf-8') as f:
                return json.load(f)
        return {}

    def save_history(self, history: Dict[str, Any]):
        """保存历史评分"""
        self.history_path.parent.mkdir(parents=True, exist_ok=True)
        with open(self.history_path, 'w', encoding='utf-8') as f:
            json.dump(history, f, indent=2, ensure_ascii=False)

    def record_score(self, direction_id: str, score: int, round_num: int):
        """记录评分"""
        history = self.load_history()
        from datetime import datetime

        history[direction_id] = {
            "score": score,
            "timestamp": datetime.now().isoformat(),
            "round": round_num
        }

        self.save_history(history)
        self.append_log(direction_id, score, round_num)

    def append_log(self, direction_id: str, score: int, round_num: int):
        """追加日志到CSV"""
        from datetime import datetime

        self.log_path.parent.mkdir(parents=True, exist_ok=True)

        # 获取方向名称
        direction = self.get_direction_by_id(direction_id)
        direction_name = direction['direction_name'] if direction else "Unknown"

        # 检查文件是否存在
        header = "timestamp,round,direction_id,direction_name,score,status\n"
        status = "commit"  # 默认

        line = f"{datetime.now().isoformat()},{round_num},{direction_id},{direction_name},{score},{status}\n"

        if not self.log_path.exists():
            with open(self.log_path, 'w', encoding='utf-8') as f:
                f.write(header)
                f.write(line)
        else:
            with open(self.log_path, 'a', encoding='utf-8') as f:
                f.write(line)

    def get_last_score(self, direction_id: str) -> int:
        """获取上次评分"""
        history = self.load_history()
        if direction_id in history:
            return history[direction_id].get('score', 0)
        return 0


def main():
    """测试状态管理"""
    import sys

    if len(sys.argv) < 2:
        print("Usage: research_state.py <command>")
        sys.exit(1)

    state = ResearchState(".")

    cmd = sys.argv[1]

    if cmd == "list":
        for d in state.get_directions_by_priority():
            print(f"{d['id']}: {d['direction_name']} (优先级 {d['priority']})")

    elif cmd == "status":
        direction_id = sys.argv[2] if len(sys.argv) > 2 else "01"
        step = state.get_current_step(direction_id)
        complete = state.is_direction_complete(direction_id)
        print(f"Direction {direction_id}: {step}, complete={complete}")

    elif cmd == "history":
        history = state.load_history()
        print(json.dumps(history, indent=2))


if __name__ == "__main__":
    main()
