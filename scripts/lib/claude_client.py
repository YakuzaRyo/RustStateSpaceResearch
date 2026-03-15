#!/usr/bin/env python3
"""
Claude CLI Client
封装 Claude Code CLI 调用
"""

import subprocess
import json
import uuid
from pathlib import Path
from typing import Optional, Dict, Any


class ClaudeClient:
    def __init__(self, project_dir: str):
        self.project_dir = Path(project_dir)

    def call(self, prompt: str, session_id: Optional[str] = None,
             output_format: str = "json") -> Dict[str, Any]:
        """
        调用 Claude CLI 执行 prompt

        Args:
            prompt: 要执行的 prompt
            session_id: 会话ID，如果为None则创建新会话
            output_format: 输出格式 (json/text)

        Returns:
            包含结果的字典
        """
        cmd = [
            "claude",
            "-p", prompt,
            "--output-format", output_format,
            "--dangerously-skip-permissions"
        ]

        if session_id:
            cmd.extend(["--session-id", session_id])
        else:
            cmd.append("--no-session-persistence")

        result = subprocess.run(
            cmd,
            cwd=str(self.project_dir),
            capture_output=True,
            text=True,
            timeout=300
        )

        if result.returncode != 0:
            return {
                "success": False,
                "error": result.stderr,
                "output": result.stdout
            }

        # 尝试解析 JSON 输出
        try:
            output = json.loads(result.stdout)
            return {
                "success": True,
                "output": output
            }
        except json.JSONDecodeError:
            return {
                "success": True,
                "output": result.stdout,
                "raw": True
            }

    def call_new_session(self, prompt: str) -> Dict[str, Any]:
        """
        创建新会话调用 Claude CLI
        """
        session_id = str(uuid.uuid4())
        return self.call(prompt, session_id=session_id)


def main():
    """测试 CLI 调用"""
    import sys
    client = ClaudeClient(".")

    if len(sys.argv) < 2:
        print("Usage: claude_client.py <prompt>")
        sys.exit(1)

    prompt = sys.argv[1]
    result = client.call_new_session(prompt)

    print(json.dumps(result, indent=2, ensure_ascii=False))


if __name__ == "__main__":
    main()
