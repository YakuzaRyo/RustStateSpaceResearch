#!/usr/bin/env python3
"""
State Space Research Results Analyzer
"""

import json
import sys
from pathlib import Path
from collections import defaultdict

def analyze_results(project_dir: str):
    project_path = Path(project_dir)
    results_path = project_path / "results.tsv"
    plan_path = project_path / "plan.json"

    if not results_path.exists():
        print("results.tsv 不存在，请先运行 evaluate.py")
        return

    # Load results
    with open(results_path, 'r', encoding='utf-8') as f:
        lines = f.readlines()

    if len(lines) < 2:
        print("没有结果数据")
        return

    header = lines[0].strip().split('\t')
    results = []
    for line in lines[1:]:
        values = line.strip().split('\t')
        row = dict(zip(header, values))
        results.append(row)

    # Analysis
    print("\n" + "=" * 60)
    print("研究进度分析")
    print("=" * 60)

    # Total scores
    total_score = sum(float(r.get('total', 0)) for r in results)
    max_score = 250 * len(results)
    print(f"\n总分: {total_score}/{max_score} ({total_score/max_score*100:.1f}%)")

    # Breakdown by step
    print("\n各步骤得分分布:")
    steps = ['step1_priority', 'step2_literature', 'step3_hypotheses',
             'step4_verified', 'step5_framework', 'step5_features',
             'step5_tests', 'step5_context']

    step_names = {
        'step1_priority': '优先级',
        'step2_literature': '文献调研',
        'step3_hypotheses': '新假设',
        'step4_verified': '可验证假设',
        'step5_framework': '框架搭建',
        'step5_features': '核心功能',
        'step5_tests': '测试验证',
        'step5_context': '上下文占用'
    }

    for step in steps:
        if step in header:
            scores = [float(r.get(step, 0)) for r in results]
            avg = sum(scores) / len(scores) if scores else 0
            max_val = max(scores) if scores else 0
            print(f"  {step_names.get(step, step)}: 平均 {avg:.1f}, 最高 {max_val}")

    # Best direction
    if results:
        best = max(results, key=lambda x: float(x.get('total', 0)))
        print(f"\n最佳方向: {best.get('direction')} ({best.get('total')}分)")

    # Progress by direction
    print("\n各方向进度:")
    for r in results:
        total = float(r.get('total', 0))
        pct = total / 250 * 100
        bar = '█' * int(pct / 5) + '░' * (20 - int(pct / 5))
        print(f"  {r.get('direction'):12s} [{bar}] {pct:.0f}%")

    # Load plan to check current direction
    if plan_path.exists():
        with open(plan_path, 'r', encoding='utf-8') as f:
            plan = json.load(f)

        current = plan.get('current_direction_id', '01')
        step = plan.get('current_step', 'step1_priority')
        print(f"\n当前方向: {current}, 步骤: {step}")

def main():
    if len(sys.argv) < 2:
        # Default to current directory
        project_dir = Path(__file__).parent.parent
    else:
        project_dir = Path(sys.argv[1])

    analyze_results(str(project_dir))

if __name__ == "__main__":
    main()
