#!/usr/bin/env python3
"""
GitHub Helper
GitHub 提交辅助工具
"""

import subprocess
import os
from pathlib import Path
from typing import Optional, Dict, Any


class GitHelper:
    def __init__(self, project_dir: str):
        self.project_dir = Path(project_dir)

    def run(self, cmd: list, capture_output: bool = True) -> subprocess.CompletedProcess:
        """运行 git 命令"""
        return subprocess.run(
            cmd,
            cwd=str(self.project_dir),
            capture_output=capture_output,
            text=True
        )

    def status(self) -> Dict[str, Any]:
        """获取 git 状态"""
        result = self.run(["git", "status", "--porcelain"])
        files = result.stdout.strip().split('\n') if result.stdout.strip() else []
        return {
            "has_changes": len(files) > 0,
            "files": [f for f in files if f]
        }

    def add(self, patterns: list) -> bool:
        """添加文件到暂存区"""
        cmd = ["git", "add"] + patterns
        result = self.run(cmd)
        return result.returncode == 0

    def commit(self, message: str) -> bool:
        """提交更改"""
        cmd = ["git", "commit", "-m", message]
        result = self.run(cmd)
        return result.returncode == 0

    def get_current_branch(self) -> str:
        """获取当前分支"""
        result = self.run(["git", "branch", "--show-current"])
        return result.stdout.strip()

    def create_branch(self, branch_name: str) -> bool:
        """创建新分支"""
        cmd = ["git", "checkout", "-b", branch_name]
        result = self.run(cmd)
        return result.returncode == 0

    def checkout(self, branch_name: str) -> bool:
        """切换分支"""
        cmd = ["git", "checkout", branch_name]
        result = self.run(cmd)
        return result.returncode == 0

    def push(self, remote: str = "origin", branch: Optional[str] = None) -> bool:
        """推送代码"""
        cmd = ["git", "push", remote]
        if branch:
            cmd.append(branch)
        result = self.run(cmd)
        return result.returncode == 0

    def has_remote(self) -> bool:
        """检查是否有远程仓库"""
        result = self.run(["git", "remote", "-v"])
        return result.returncode == 0 and bool(result.stdout.strip())

    def create_pr(self, title: str, body: str, branch: Optional[str] = None) -> Optional[str]:
        """创建 GitHub PR"""
        if not self.has_remote():
            print("No remote repository configured")
            return None

        branch = branch or self.get_current_branch()

        cmd = [
            "gh", "pr", "create",
            "--title", title,
            "--body", body,
            "--base", "main"
        ]

        result = self.run(cmd)
        if result.returncode == 0:
            # 提取 PR URL
            output = result.stdout.strip()
            if "http" in output:
                return output
            return f"https://github.com/{output}"
        else:
            print(f"Failed to create PR: {result.stderr}")
            return None

    def commit_direction(self, direction_id: str, direction_name: str,
                        score: int, changes: list) -> bool:
        """提交方向研究结果"""
        print(f"\n提交方向 {direction_id}: {direction_name} (得分: {score})")

        # 检查是否有更改
        status = self.status()
        if not status["has_changes"]:
            print("没有需要提交的更改")
            return False

        # 添加相关文件
        patterns = changes if changes else ["directions/", "framework/", "verification/"]
        self.add(patterns)

        # 提交
        message = f"研究方向 {direction_id}: {direction_name} 得分 {score}"
        success = self.commit(message)

        if success:
            print(f"✓ 提交成功: {message}")
        else:
            print(f"✗ 提交失败")

        return success


def main():
    """测试 GitHub 辅助工具"""
    import sys

    helper = GitHelper(".")

    if len(sys.argv) < 2:
        print("Usage: git_helper.py <command>")
        sys.exit(1)

    cmd = sys.argv[1]

    if cmd == "status":
        status = helper.status()
        print(f"Has changes: {status['has_changes']}")
        for f in status['files']:
            print(f"  {f}")

    elif cmd == "branch":
        print(f"Current branch: {helper.get_current_branch()}")


if __name__ == "__main__":
    main()
