import os
import subprocess
import sys
from pathlib import Path

SCRIPT = Path(__file__).resolve().parents[1] / "scripts" / "hf_auth.py"


def run(args, env=None):
    return subprocess.run(
        [sys.executable, str(SCRIPT), *args],
        capture_output=True,
        text=True,
        env={**os.environ, **(env or {})},
    )


def test_import_prints_token_when_file_exists(tmp_path):
    token_file = tmp_path / "token"
    token_file.write_text("hf_abcdef1234567890123456\n")
    result = run(["import"], env={"HF_TOKEN_FILE": str(token_file)})
    assert result.returncode == 0, result.stderr
    assert "hf_abcdef1234567890123456" in result.stdout
    assert "huggingface plugin" in result.stdout.lower()


def test_import_fails_when_file_missing(tmp_path):
    missing = tmp_path / "nope"
    result = run(["import"], env={"HF_TOKEN_FILE": str(missing)})
    assert result.returncode != 0
    combined = (result.stdout + result.stderr).lower()
    assert "not found" in combined


def test_import_rejects_obviously_wrong_token(tmp_path):
    token_file = tmp_path / "token"
    token_file.write_text("not-a-real-token\n")
    result = run(["import"], env={"HF_TOKEN_FILE": str(token_file)})
    assert result.returncode != 0
