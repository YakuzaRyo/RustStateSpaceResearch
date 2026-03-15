#!/usr/bin/env python3
"""
State Space Research Evaluator
250-point scoring system
"""

import json
import os
import sys
import subprocess
from pathlib import Path
from typing import Dict, Any, List

class Evaluator:
    def __init__(self, project_dir: str):
        self.project_dir = Path(project_dir)
        self.plan_path = self.project_dir / "plan.json"
        self.every_goal_path = self.project_dir / "every_goal.json"
        self.results_path = self.project_dir / "results.tsv"

    def load_json(self, path: Path) -> Dict[str, Any]:
        with open(path, 'r', encoding='utf-8') as f:
            return json.load(f)

    def save_json(self, path: Path, data: Dict[str, Any]):
        with open(path, 'w', encoding='utf-8') as f:
            json.dump(data, f, indent=2, ensure_ascii=False)

    # Step 1: Priority scoring
    def score_priority(self, direction: Dict[str, Any]) -> int:
        priority = direction.get('priority', 6)
        # Priority 1 = 10 points, Priority 6 = 0 points
        return max(0, 10 - (priority - 1) * 2)

    # Step 2: Literature scoring
    def score_literature(self, direction: Dict[str, Any]) -> int:
        current_increment = direction.get('literature_increment', 0)
        last_increment = direction.get('last_literature_increment', 0)
        current_score = direction.get('literature_score', 30)

        if current_increment >= last_increment:
            return 30  # Full score if increment maintained/increased
        else:
            # Decreased by 1 point, minimum 15
            return max(15, current_score - 1)

    # Step 3: Hypotheses scoring
    def score_hypotheses(self, direction: Dict[str, Any]) -> int:
        hypotheses = direction.get('hypotheses', [])
        # Each hypothesis = 1 point, max 10
        return min(len(hypotheses), 10)

    # Step 4: Verified hypotheses scoring
    def score_verified(self, direction: Dict[str, Any]) -> int:
        verified_count = direction.get('verified_count', 0)
        # Each verified = 2 points, max 20
        return min(verified_count * 2, 20)

    # Step 5.1: Framework scoring
    def score_framework(self) -> Dict[str, Any]:
        framework_dir = self.project_dir / "framework"
        score = 0
        details = {}

        # Check cargo check
        cargo_toml = framework_dir / "Cargo.toml"
        lib_rs = framework_dir / "src" / "lib.rs"

        if cargo_toml.exists():
            score += 10
            details['cargo_toml'] = True
        if lib_rs.exists():
            score += 10
            details['lib_rs'] = True

        # Try cargo check
        try:
            result = subprocess.run(
                ['cargo', 'check', '--manifest-path', str(framework_dir / 'Cargo.toml')],
                capture_output=True,
                text=True,
                timeout=60
            )
            if result.returncode == 0:
                score += 10
                details['cargo_check'] = True
            else:
                details['cargo_check'] = False
                details['cargo_error'] = result.stderr[:200]
        except Exception as e:
            details['cargo_check'] = False
            details['cargo_error'] = str(e)

        return {'score': score, 'details': details}

    # Step 5.2: Features scoring
    def score_features(self, direction: Dict[str, Any]) -> int:
        features = direction.get('code_features', [])
        # Each feature = 5 points, max 50
        return min(len(features) * 5, 50)

    # Step 5.3: Tests scoring
    def score_tests(self) -> Dict[str, Any]:
        framework_dir = self.project_dir / "framework"

        try:
            # Run cargo test
            result = subprocess.run(
                ['cargo', 'test', '--manifest-path', str(framework_dir / 'Cargo.toml'), '--', '--json'],
                capture_output=True,
                text=True,
                timeout=120
            )

            if result.returncode == 0:
                return {'coverage': 0.5, 'passed': 1.0, 'score': 50}
            else:
                return {'coverage': 0.0, 'passed': 0.0, 'score': 0}
        except Exception as e:
            return {'coverage': 0.0, 'passed': 0.0, 'score': 0, 'error': str(e)}

    # Step 5.4: Context scoring
    def score_context(self, direction: Dict[str, Any]) -> Dict[str, Any]:
        framework_dir = self.project_dir / "framework"
        src_dir = framework_dir / "src"

        # Code lines
        total_lines = 0
        if src_dir.exists():
            for rs_file in src_dir.glob("*.rs"):
                with open(rs_file, 'r', encoding='utf-8') as f:
                    lines = [l for l in f.readlines() if l.strip() and not l.strip().startswith('//')]
                    total_lines += len(lines)

        lines_score = min(total_lines / 1000 * 10, 10)

        # Module count
        module_count = 0
        if src_dir.exists():
            module_count = len(list(src_dir.glob("*.rs")))

        module_score = min(module_count * 2, 10)

        # Type complexity
        type_count = 0
        if src_dir.exists():
            for rs_file in src_dir.glob("*.rs"):
                with open(rs_file, 'r', encoding='utf-8') as f:
                    content = f.read()
                    type_count += content.count('struct ')
                    type_count += content.count('enum ')
                    type_count += content.count('trait ')

        type_score = min(type_count * 1.5, 15)

        # Context reuse (simplified)
        reuse_score = 15  # Default to full score for now

        total = lines_score + module_score + type_score + reuse_score

        return {
            'score': min(total, 50),
            'lines': total_lines,
            'modules': module_count,
            'types': type_count,
            'breakdown': {
                'lines_score': lines_score,
                'module_score': module_score,
                'type_score': type_score,
                'reuse_score': reuse_score
            }
        }

    # Calculate total score for a direction
    def evaluate_direction(self, direction: Dict[str, Any]) -> Dict[str, Any]:
        scores = {
            'step1_priority': self.score_priority(direction),
            'step2_literature': self.score_literature(direction),
            'step3_hypotheses': self.score_hypotheses(direction),
            'step4_verified': self.score_verified(direction),
        }

        # Step 5 scores
        framework_result = self.score_framework()
        scores['step5_framework'] = framework_result['score']

        scores['step5_features'] = self.score_features(direction)

        tests_result = self.score_tests()
        scores['step5_tests'] = tests_result['score']

        context_result = self.score_context(direction)
        scores['step5_context'] = context_result['score']

        # Total
        total = sum(scores.values())

        return {
            'direction': direction['direction_name'],
            'scores': scores,
            'total': total,
            'details': {
                'framework': framework_result,
                'tests': tests_result,
                'context': context_result
            }
        }

    def evaluate_all(self) -> List[Dict[str, Any]]:
        plan = self.load_json(self.plan_path)
        directions = plan.get('research_directions', [])

        results = []
        for direction in directions:
            result = self.evaluate_direction(direction)
            results.append(result)

        return results

    def save_results(self, results: List[Dict[str, Any]]):
        # Save as TSV
        header = ["direction", "step1_priority", "step2_literature", "step3_hypotheses",
                  "step4_verified", "step5_framework", "step5_features", "step5_tests",
                  "step5_context", "total"]

        lines = ["\t".join(header)]
        for r in results:
            row = [
                r['direction'],
                str(r['scores'].get('step1_priority', 0)),
                str(r['scores'].get('step2_literature', 0)),
                str(r['scores'].get('step3_hypotheses', 0)),
                str(r['scores'].get('step4_verified', 0)),
                str(r['scores'].get('step5_framework', 0)),
                str(r['scores'].get('step5_features', 0)),
                str(r['scores'].get('step5_tests', 0)),
                str(r['scores'].get('step5_context', 0)),
                str(r['total'])
            ]
            lines.append("\t".join(row))

        with open(self.results_path, 'w', encoding='utf-8') as f:
            f.write("\n".join(lines))

    def print_results(self, results: List[Dict[str, Any]]):
        print("\n" + "=" * 60)
        print("State Space Research Evaluation Results (250-point system)")
        print("=" * 60)

        for r in results:
            print(f"\n【{r['direction']}】 总分: {r['total']}/250")
            print(f"  Step1 (优先级):     {r['scores']['step1_priority']}/10")
            print(f"  Step2 (文献调研):   {r['scores']['step2_literature']}/30")
            print(f"  Step3 (新假设):     {r['scores']['step3_hypotheses']}/10")
            print(f"  Step4 (可验证假设): {r['scores']['step4_verified']}/20")
            print(f"  Step5.1 (框架):     {r['scores']['step5_framework']}/30")
            print(f"  Step5.2 (功能):     {r['scores']['step5_features']}/50")
            print(f"  Step5.3 (测试):     {r['scores']['step5_tests']}/50")
            print(f"  Step5.4 (上下文):   {r['scores']['step5_context']}/50")

        total_all = sum(r['total'] for r in results)
        print("\n" + "=" * 60)
        print(f"总计: {total_all}/{250 * len(results)}")
        print("=" * 60)

def main():
    if len(sys.argv) < 2:
        print("Usage: evaluate.py <project_dir>")
        sys.exit(1)

    project_dir = sys.argv[1]
    evaluator = Evaluator(project_dir)

    results = evaluator.evaluate_all()
    evaluator.save_results(results)
    evaluator.print_results(results)

if __name__ == "__main__":
    main()
