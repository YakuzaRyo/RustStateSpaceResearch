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
        self.directions_dir = self.project_dir / "directions"
        self.every_goal_path = self.project_dir / "every_goal.json"
        self.results_path = self.project_dir / "results.tsv"

    def load_json(self, path: Path) -> Dict[str, Any]:
        with open(path, 'r', encoding='utf-8') as f:
            return json.load(f)

    def load_direction_detail(self, direction_id: str) -> Dict[str, Any]:
        """Load detailed direction data from directions/*.json file"""
        # Find the direction file (e.g., 01_core_principles.json)
        if self.directions_dir.exists():
            for f in self.directions_dir.glob(f"{direction_id}_*.json"):
                return self.load_json(f)
        return {}

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
        """Score framework with refined 30-point system across 5 dimensions"""
        framework_dir = self.project_dir / "framework"
        score = 0
        details = {}

        # Dimension 1: Core trait definitions (6 points)
        trait_score = 0
        lib_rs = framework_dir / "src" / "lib.rs"
        if lib_rs.exists():
            lib_content = lib_rs.read_text(encoding='utf-8')
            # Check Invariant trait
            if 'trait Invariant' in lib_content:
                trait_score += 2
                details['invariant_trait'] = True
            else:
                details['invariant_trait'] = False
            # Check StateSpace trait
            if 'trait StateSpace' in lib_content:
                trait_score += 2
                details['statespace_trait'] = True
            else:
                details['statespace_trait'] = False
            # Check Transition/Guard
            if 'struct Guard' in lib_content or 'trait Transition' in lib_content:
                trait_score += 2
                details['transition_trait'] = True
            else:
                details['transition_trait'] = False
        else:
            details['invariant_trait'] = False
            details['statespace_trait'] = False
            details['transition_trait'] = False
        details['trait_score'] = trait_score
        score += trait_score

        # Dimension 2: Layered module completeness (6 points)
        module_score = 0
        src_dir = framework_dir / "src"
        modules = {
            'syntax': 'syntax.rs',
            'semantic': 'semantic.rs',
            'pattern': 'pattern.rs',
            'domain': 'domain.rs'
        }
        details['modules'] = {}
        for name, filename in modules.items():
            if (src_dir / filename).exists():
                module_score += 1.5
                details['modules'][name] = True
            else:
                details['modules'][name] = False
        details['module_score'] = module_score
        score += module_score

        # Dimension 3: Core functionality implementation (6 points)
        func_score = 0
        if lib_rs.exists():
            lib_content = lib_rs.read_text(encoding='utf-8')
            # StateSpaceAlgebra / safe/danger zones
            if 'StateSpaceAlgebra' in lib_content or ('safe' in lib_content.lower() and 'danger' in lib_content.lower()):
                func_score += 2
                details['state_algebra'] = True
            else:
                details['state_algebra'] = False
            # Guard mechanism
            if 'struct Guard' in lib_content or 'fn allows' in lib_content:
                func_score += 2
                details['guard_mechanism'] = True
            else:
                details['guard_mechanism'] = False
            # Example implementations (BankAccount, etc.)
            if 'BankAccount' in lib_content or 'struct' in lib_content:
                func_score += 2
                details['example_impl'] = True
            else:
                details['example_impl'] = False
        else:
            details['state_algebra'] = False
            details['guard_mechanism'] = False
            details['example_impl'] = False
        details['func_score'] = func_score
        score += func_score

        # Dimension 4: Code quality (6 points)
        quality_score = 0
        cargo_toml = framework_dir / "Cargo.toml"

        # cargo check passes (3 points)
        if cargo_toml.exists():
            try:
                result = subprocess.run(
                    ['cargo', 'check', '--manifest-path', str(framework_dir / 'Cargo.toml')],
                    capture_output=True,
                    text=True,
                    timeout=60
                )
                if result.returncode == 0:
                    quality_score += 3
                    details['cargo_check'] = True
                else:
                    details['cargo_check'] = False
                    details['cargo_error'] = result.stderr[:200]
            except Exception as e:
                details['cargo_check'] = False
                details['cargo_error'] = str(e)
        else:
            details['cargo_check'] = False

        # Code lines >= 500 (3 points)
        total_lines = 0
        if src_dir.exists():
            for rs_file in src_dir.glob('*.rs'):
                total_lines += len(rs_file.read_text(encoding='utf-8').splitlines())
        if total_lines >= 500:
            quality_score += 3
            details['code_lines'] = total_lines
            details['code_lines_ok'] = True
        else:
            details['code_lines'] = total_lines
            details['code_lines_ok'] = False
        details['quality_score'] = quality_score
        score += quality_score

        # Dimension 5: Tests (6 points)
        test_score = 0
        test_count = 0

        # Count #[test] annotations
        if src_dir.exists():
            for rs_file in src_dir.glob('*.rs'):
                content = rs_file.read_text(encoding='utf-8')
                test_count += content.count('#[test]')

        # Test count >= 5 (3 points)
        if test_count >= 5:
            test_score += 3
            details['test_count'] = test_count
            details['test_count_ok'] = True
        else:
            details['test_count'] = test_count
            details['test_count_ok'] = False

        # Core principle validation tests (3 points)
        # Look for tests that verify invariants/guards
        has_invariant_test = False
        if src_dir.exists():
            for rs_file in src_dir.glob('*.rs'):
                content = rs_file.read_text(encoding='utf-8')
                if 'invariant' in content.lower() or 'guard' in content.lower():
                    if '#[test]' in content:
                        has_invariant_test = True
                        break
        if has_invariant_test:
            test_score += 3
            details['invariant_test'] = True
        else:
            details['invariant_test'] = False

        details['test_score'] = test_score
        score += test_score

        details['total'] = score
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
            # Load detailed data from directions/*.json
            detail = self.load_direction_detail(direction.get('id', ''))
            # Merge: detail data overrides plan data
            merged = {**direction, **detail}
            result = self.evaluate_direction(merged)
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
