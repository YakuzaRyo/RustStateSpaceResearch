#!/usr/bin/env python3
"""
Prompt Builder
为每个研究方向和步骤构建 Claude Code prompt
"""

from typing import Dict, Any, List


class PromptBuilder:
    """Prompt 构建器"""

    # 步骤定义
    STEPS = {
        'step1_priority': {
            'name': '优先级排序',
            'description': '确定研究方向优先级并排序'
        },
        'step2_literature': {
            'name': '文献调研',
            'description': '调研相关文献，添加 literature 条目'
        },
        'step3_hypotheses': {
            'name': '提出假设',
            'description': '基于文献提出可验证的假设'
        },
        'step4_verified': {
            'name': '验证假设',
            'description': '编写验证代码确认假设成立'
        },
        'step5_code': {
            'name': '代码实现',
            'description': '实现核心框架和功能代码'
        }
    }

    def __init__(self, project_dir: str):
        self.project_dir = project_dir

    def build_research_prompt(self, direction: Dict[str, Any], step: str) -> str:
        """
        构建研究方向的研究 prompt

        Args:
            direction: 方向数据字典
            step: 当前步骤

        Returns:
            构建好的 prompt 字符串
        """
        direction_id = direction['id']
        direction_name = direction['direction_name']
        question = direction['question']
        topics = direction.get('topics', [])
        hypotheses = direction.get('hypotheses', [])
        literature_count = direction.get('literature_count', 0)

        step_info = self.STEPS.get(step, {'name': '未知步骤', 'description': ''})

        # 基础上下文
        prompt = f"""# 状态空间研究 - {direction_name}

## 任务背景
你正在研究"状态空间代数"这一主题，具体方向是「{direction_name}」。

## 研究问题
{direction_id}: {question}

## 相关主题
{', '.join(topics)}

## 当前步骤
**{step_info['name']}**: {step_info['description']}

"""

        # 根据步骤添加具体指令
        if step == 'step1_priority':
            prompt += self._build_step1_prompt(direction)
        elif step == 'step2_literature':
            prompt += self._build_step2_prompt(direction)
        elif step == 'step3_hypotheses':
            prompt += self._build_step3_prompt(direction)
        elif step == 'step4_verified':
            prompt += self._build_step4_prompt(direction)
        elif step == 'step5_code':
            prompt += self._build_step5_prompt(direction)

        # 添加执行指令
        prompt += """

## 执行指令
1. 分析当前研究方向的状态
2. 执行该步骤的研究任务
3. 更新 `directions/{id}.json` 文件中的相关字段
4. 如果需要编写代码，保存到 `framework/src/` 或 `verification/` 目录

## 输出要求
完成后，简要说明：
- 完成了什么工作
- 更新了哪些文件
- 下一步建议
"""

        return prompt

    def _build_step1_prompt(self, direction: Dict[str, Any]) -> str:
        """构建 Step 1: 优先级排序的 prompt"""
        return """## Step 1: 优先级排序任务
分析所有6个研究方向，确定它们的优先级顺序。
优先级标准：
- 与核心问题（"让错误在设计上不可能发生"）的相关性
- 对整体研究贡献的重要性
- 实现的可行性

请更新 `plan.json` 中的 priority 字段。
"""

    def _build_step2_prompt(self, direction: Dict[str, Any]) -> str:
        """构建 Step 2: 文献调研的 prompt"""
        direction_name = direction['direction_name']
        question = direction['question']
        current_count = direction.get('literature_count', 0)

        return f"""## Step 2: 文献调研任务
当前已有文献数量: {current_count}

研究问题: {question}

任务：
1. 使用 WebSearch 搜索相关文献（{direction_name} 方面）
2. 筛选高质量文献（学术论文、技术博客、权威资料）
3. 将文献添加到 `directions/{direction['id']}_*.json` 的 `literature` 字段
4. 更新 `literature_count` 和 `literature_increment`

文献格式：
```json
{{
  "title": "文献标题",
  "author": "作者",
  "year": 年份,
  "type": "paper|article|book|blog",
  "url": "链接",
  "summary": "摘要"
}}
```
"""

    def _build_step3_prompt(self, direction: Dict[str, Any]) -> str:
        """构建 Step 3: 提出假设的 prompt"""
        direction_name = direction['direction_name']
        question = direction['question']
        hypotheses = direction.get('hypotheses', [])
        literature = direction.get('literature', [])

        lit_list = ""
        if literature:
            lit_list = "\n已有关键文献：\n"
            for lit in literature[:5]:
                lit_list += f"- {lit.get('title', 'Unknown')}: {lit.get('summary', '')[:100]}...\n"

        return f"""## Step 3: 提出假设任务
当前已有假设数量: {len(hypotheses)}
{lit_list}

研究问题: {question}

任务：
1. 基于文献调研，提出可验证的假设
2. 假设应该：
   - 明确且具体
   - 可通过代码验证
   - 与状态空间代数相关
3. 将假设添加到 `directions/{direction['id']}_*.json` 的 `hypotheses` 字段

假设格式：
```json
{{
  "id": "h1",
  "statement": "假设陈述",
  "basis": "理论基础",
  "verification_method": "验证方法描述",
  "status": "pending"
}}
```
"""

    def _build_step4_prompt(self, direction: Dict[str, Any]) -> str:
        """构建 Step 4: 验证假设的 prompt"""
        direction_name = direction['direction_name']
        hypotheses = direction.get('hypotheses', [])

        pending_hypotheses = [h for h in hypotheses if h.get('status') != 'verified']

        hyp_list = ""
        if pending_hypotheses:
            hyp_list = "\n待验证假设：\n"
            for h in pending_hypotheses[:3]:
                hyp_list += f"- {h.get('id')}: {h.get('statement', '')}\n"

        return f"""## Step 4: 验证假设任务
{direction_name}
{hyp_list}

任务：
1. 选择一个或多个 pending 状态的假设
2. 编写验证代码到 `verification/` 目录
3. 运行验证，确认假设成立（或不成立）
4. 更新假设的 status 为 'verified' 或 'rejected'
5. 增加 `verified_count`

验证代码要求：
- 简洁、可运行
- 有明确的测试用例
- 输出验证结果
"""

    def _build_step5_prompt(self, direction: Dict[str, Any]) -> str:
        """构建 Step 5: 代码实现的 prompt"""
        direction_name = direction['direction_name']
        question = direction['question']
        topics = direction.get('topics', [])
        hypotheses = direction.get('hypotheses', [])
        features = direction.get('code_features', [])

        verified_hypotheses = [h for h in hypotheses if h.get('status') == 'verified']

        return f"""## Step 5: 代码实现任务
研究问题: {question}
相关主题: {', '.join(topics)}
已有功能数量: {len(features)}
已验证假设数量: {len(verified_hypotheses)}

任务：
1. 基于研究和验证结果，实现核心框架代码
2. 将代码保存到 `framework/src/` 目录
3. 更新 `directions/{direction['id']}_*.json` 的 `code_features` 字段

代码要求：
- 使用 Rust 语言
- 符合状态空间代数理念
- 类型安全、编译通过
- 包含必要的测试用例

功能格式：
```json
{{
  "name": "功能名称",
  "description": "功能描述",
  "file": "相对路径",
  "tested": true
}}
```
"""

    def build_evaluation_prompt(self, direction: Dict[str, Any]) -> str:
        """构建评估 prompt"""
        return f"""# 研究方向评估

请评估「{direction['direction_name']}」方向的当前状态：

1. 文献数量和质量
2. 假设的数量和质量
3. 已验证的假设数量
4. 代码实现程度

输出 JSON 格式：
```json
{{
  "literature_score": 0-30,
  "hypotheses_score": 0-10,
  "verified_score": 0-20,
  "code_score": 0-50,
  "total": 0-110,
  "comments": "评估意见"
}}
```
"""


def main():
    """测试 Prompt 构建"""
    import sys

    if len(sys.argv) < 3:
        print("Usage: prompt_builder.py <direction_id> <step>")
        sys.exit(1)

    from research_state import ResearchState

    direction_id = sys.argv[1]
    step = sys.argv[2]

    state = ResearchState(".")
    direction = state.load_direction(direction_id)

    builder = PromptBuilder(".")
    prompt = builder.build_research_prompt(direction, step)

    print(prompt)


if __name__ == "__main__":
    main()
