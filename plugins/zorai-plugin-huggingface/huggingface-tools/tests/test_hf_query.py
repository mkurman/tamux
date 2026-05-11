import json
import os
import subprocess
import sys
from pathlib import Path

import pyarrow as pa
import pyarrow.parquet as pq

SCRIPT = Path(__file__).resolve().parents[1] / "scripts" / "hf_query.py"


def make_fixture(tmp_path: Path) -> Path:
    """Create a minimal parquet file and a fake parquet-URL index."""
    parquet_path = tmp_path / "data.parquet"
    table = pa.table({
        "id": pa.array([1, 2, 3], type=pa.int64()),
        "text": pa.array(["a", "b", "c"]),
    })
    pq.write_table(table, parquet_path)
    index_path = tmp_path / "index.json"
    index_path.write_text(json.dumps({"default": {"train": [str(parquet_path)]}}))
    return index_path


def run(args, env=None):
    return subprocess.run(
        [sys.executable, str(SCRIPT), *args],
        capture_output=True,
        text=True,
        env={**os.environ, **(env or {})},
    )


def test_schema_lists_columns(tmp_path):
    index = make_fixture(tmp_path)
    result = run(
        ["schema", "--dataset", "fake/ds"],
        env={"HF_FAKE_PARQUET_INDEX": str(index), "HF_QUERY_WORKDIR": str(tmp_path)},
    )
    assert result.returncode == 0, result.stderr
    assert "id" in result.stdout
    assert "text" in result.stdout
    assert "|" in result.stdout  # markdown table


def test_query_runs_sql_and_renders_markdown(tmp_path):
    index = make_fixture(tmp_path)
    result = run(
        ["query", "--dataset", "fake/ds", "--sql", "SELECT id, text FROM data WHERE id > 1 ORDER BY id"],
        env={"HF_FAKE_PARQUET_INDEX": str(index), "HF_QUERY_WORKDIR": str(tmp_path)},
    )
    assert result.returncode == 0, result.stderr
    assert "| 2 | b |" in result.stdout
    assert "| 3 | c |" in result.stdout
    assert "| 1 |" not in result.stdout


def test_query_truncates_to_limit(tmp_path):
    index = make_fixture(tmp_path)
    result = run(
        ["query", "--dataset", "fake/ds", "--sql", "SELECT id FROM data", "--limit", "2"],
        env={"HF_FAKE_PARQUET_INDEX": str(index), "HF_QUERY_WORKDIR": str(tmp_path)},
    )
    assert result.returncode == 0, result.stderr
    assert "rows shown" in result.stdout
