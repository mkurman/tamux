#!/usr/bin/env python3
"""hf_auth — surface and validate the huggingface-cli token."""
from __future__ import annotations

import argparse
import os
import re
import subprocess
import sys
from pathlib import Path

DEFAULT_TOKEN_FILE = "~/.cache/huggingface/token"
TOKEN_PATTERN = re.compile(r"^hf_[A-Za-z0-9]{20,}$")


def resolve_token_file() -> Path:
    raw = os.environ.get("HF_TOKEN_FILE", DEFAULT_TOKEN_FILE)
    return Path(os.path.expanduser(raw))


def cmd_import() -> int:
    token_file = resolve_token_file()
    if not token_file.exists():
        print(f"Token file not found at {token_file}.", file=sys.stderr)
        print(
            "Run `/hf-tools auth-login` first, or paste a token directly into the huggingface plugin settings.",
            file=sys.stderr,
        )
        return 1
    token = token_file.read_text().strip()
    if not TOKEN_PATTERN.match(token):
        print(
            f"Token at {token_file} does not look like an HF token (expected hf_...).",
            file=sys.stderr,
        )
        return 2
    print(f"Found HF token at {token_file}.")
    print()
    print("Paste this into the huggingface plugin → `token` setting:")
    print()
    print(token)
    return 0


def cmd_login() -> int:
    return subprocess.run(["huggingface-cli", "login"]).returncode


def cmd_whoami() -> int:
    result = subprocess.run(
        ["huggingface-cli", "whoami"], capture_output=True, text=True
    )
    sys.stdout.write(result.stdout)
    sys.stderr.write(result.stderr)
    return result.returncode


def main() -> int:
    parser = argparse.ArgumentParser(prog="hf_auth")
    sub = parser.add_subparsers(dest="cmd", required=True)
    sub.add_parser("import")
    sub.add_parser("login")
    sub.add_parser("whoami")
    args = parser.parse_args()
    return {"import": cmd_import, "login": cmd_login, "whoami": cmd_whoami}[args.cmd]()


if __name__ == "__main__":
    raise SystemExit(main())
