# 状态空间代数研究 (State Space Algebra Research)

让错误在设计上不可能发生。

## 概述

本项目探索"状态空间代数"这一核心理念，通过 6 个研究方向系统性地研究和实现类型安全的编程框架。

## 研究方向

| ID | 方向 | 研究问题 |
|----|------|----------|
| 01 | 核心原则 | 如何让错误在设计上不可能发生? |
| 02 | 实现技术 | 如何用 Rust 类型系统实现状态空间? |
| 03 | 工具设计 | 如何设计"无法产生错误"的工具? |
| 04 | LLM 导航器 | LLM 作为启发式函数的理论基础? |
| 05 | 分层设计 | Syntax→Semantic→Pattern→Domain 如何转换? |
| 06 | 对比分析 | Claude Code/OpenCode 的根本缺陷是什么? |

## 评估体系 (250 分制)

| 步骤 | 分数 | 说明 |
|------|------|------|
| Step1 | 10 | 优先级排序 |
| Step2 | 30 | 文献调研 |
| Step3 | 10 | 新假设 |
| Step4 | 20 | 可验证假设 |
| Step5.1 | 30 | 框架 |
| Step5.2 | 50 | 功能 |
| Step5.3 | 50 | 测试 |
| Step5.4 | 50 | 上下文 |

## 项目结构

```
.
├── directions/          # 研究方向详情
│   ├── 01_core_principles.json
│   ├── 02_implementation.json
│   └── ...
├── framework/          # Rust 框架实现
│   └── src/lib.rs
├── scripts/            # 研究自动化脚本
│   ├── run-iterations.sh    # 迭代控制主入口
│   ├── claude-research.sh   # 单次研究调用
│   ├── scheduler.sh          # 定时任务
│   ├── evaluate.py          # 评估脚本
│   └── lib/                 # 核心库
│       ├── research_state.py
│       ├── prompt_builder.py
│       ├── claude_client.py
│       └── git_helper.py
├── logs/               # 日志文件
│   ├── research_history.json
│   ├── research_log.csv
│   └── cron.log
└── plan.json           # 研究计划
```

## 使用方法

### 运行迭代研究

```bash
# 运行 3 轮迭代（默认）
./scripts/run-iterations.sh

# 运行 1 轮迭代
./scripts/run-iterations.sh --rounds 1

# 跳过评估
./scripts/run-iterations.sh --skip-eval

# 模拟运行（不实际调用 Claude）
./scripts/run-iterations.sh --dry-run
```

### 单次研究调用

```bash
# 执行特定方向的特定步骤
./scripts/claude-research.sh --direction 01 --step step2_literature

# 可用步骤: step1_priority, step2_literature, step3_hypotheses, step4_verified, step5_code
```

### 评估

```bash
# 评估当前研究状态
python3 scripts/evaluate.py .
```

### 定时任务

每 20 分钟自动运行一轮研究：

```bash
# 添加到 crontab
crontab -e
# 添加: */20 * * * * cd /home/ume/RustStateSpaceResearch && ./scripts/scheduler.sh >> logs/cron.log 2>&1
```

## 核心概念

### 状态空间代数

将程序状态建模为代数结构，通过类型系统保证状态转换的安全性。

### 不变量

- **编译时不变量**: 类型约束、生命周期、 trait 限定
- **运行时不变量**: 断言、检查、验证

### 状态转换

```
State --(Transition)--> State --(Transition)--> State
     + Invariant          + Invariant          + Invariant
```

## 依赖

- Python 3.8+
- Rust 1.70+
- Claude Code CLI
- GitHub CLI (gh)

## 许可证

MIT License
