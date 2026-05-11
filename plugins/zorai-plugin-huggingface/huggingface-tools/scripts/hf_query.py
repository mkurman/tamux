#!/usr/bin/env python3
"""hf_query — DuckDB SQL over HuggingFace dataset parquet files."""
from __future__ import annotations

import argparse
import json
import os
import sys
import urllib.parse
import urllib.request
from pathlib import Path
from typing import Iterable

import duckdb

DEFAULT_WORKDIR = "~/.cache/zorai/hf-query"
HF_PARQUET_API = "https://huggingface.co/api/datasets/{dataset}/parquet"


def resolve_workdir() -> Path:
    raw = os.environ.get("HF_QUERY_WORKDIR", DEFAULT_WORKDIR)
    p = Path(os.path.expanduser(raw))
    p.mkdir(parents=True, exist_ok=True)
    return p


def fetch_parquet_urls(dataset: str, config: str | None, split: str | None) -> list[str]:
    """Return parquet URLs for (dataset, config, split). Honors HF_FAKE_PARQUET_INDEX in tests."""
    fake = os.environ.get("HF_FAKE_PARQUET_INDEX")
    if fake:
        index = json.loads(Path(fake).read_text())
    else:
        url = HF_PARQUET_API.format(dataset=urllib.parse.quote(dataset, safe="/"))
        token = os.environ.get("HF_TOKEN")
        req = urllib.request.Request(url)
        if token:
            req.add_header("Authorization", f"Bearer {token}")
        with urllib.request.urlopen(req, timeout=30) as resp:
            index = json.loads(resp.read())
    cfg = config or next(iter(index))
    splits = index[cfg]
    spl = split or next(iter(splits))
    return splits[spl]


def localize(urls: Iterable[str], workdir: Path, dataset: str, config: str, split: str) -> list[str]:
    """Download remote URLs to workdir; pass through local file paths unchanged."""
    out: list[str] = []
    target_dir = workdir / dataset.replace("/", "__") / config / split
    target_dir.mkdir(parents=True, exist_ok=True)
    token = os.environ.get("HF_TOKEN")
    for url in urls:
        if url.startswith(("http://", "https://")):
            name = Path(urllib.parse.urlparse(url).path).name or "part.parquet"
            target = target_dir / name
            if not target.exists():
                req = urllib.request.Request(url)
                if token:
                    req.add_header("Authorization", f"Bearer {token}")
                with urllib.request.urlopen(req, timeout=60) as resp, open(target, "wb") as f:
                    f.write(resp.read())
            out.append(str(target))
        else:
            out.append(url)
    return out


def open_view(dataset: str, config: str | None, split: str | None) -> duckdb.DuckDBPyConnection:
    urls = fetch_parquet_urls(dataset, config, split)
    workdir = resolve_workdir()
    files = localize(urls, workdir, dataset, config or "default", split or "default")
    con = duckdb.connect()
    files_sql = ", ".join(f"'{f}'" for f in files)
    con.execute(f"CREATE VIEW data AS SELECT * FROM read_parquet([{files_sql}])")
    return con


def render_markdown_table(con: duckdb.DuckDBPyConnection, sql: str, limit: int | None = None) -> str:
    rel = con.execute(sql)
    columns = [d[0] for d in rel.description]
    rows = rel.fetchall()
    total = len(rows)
    truncated = False
    if limit is not None and total > limit:
        rows = rows[:limit]
        truncated = True
    head = "| " + " | ".join(columns) + " |"
    sep = "| " + " | ".join(["---"] * len(columns)) + " |"
    body = "\n".join("| " + " | ".join(str(c) for c in row) + " |" for row in rows)
    out = "\n".join([head, sep, body]) if body else "\n".join([head, sep, "(no rows)"])
    if truncated:
        out += f"\n\n(... {len(rows)} rows shown of {total} total)"
    return out


def cmd_schema(args: argparse.Namespace) -> int:
    con = open_view(args.dataset, args.config, args.split)
    print(render_markdown_table(con, "DESCRIBE data"))
    return 0


def cmd_query(args: argparse.Namespace) -> int:
    con = open_view(args.dataset, args.config, args.split)
    print(render_markdown_table(con, args.sql, limit=args.limit))
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(prog="hf_query")
    sub = parser.add_subparsers(dest="cmd", required=True)
    s = sub.add_parser("schema")
    s.add_argument("--dataset", required=True)
    s.add_argument("--config")
    s.add_argument("--split")
    s.set_defaults(func=cmd_schema)
    q = sub.add_parser("query")
    q.add_argument("--dataset", required=True)
    q.add_argument("--config")
    q.add_argument("--split")
    q.add_argument("--sql", required=True)
    q.add_argument("--limit", type=int, default=50)
    q.set_defaults(func=cmd_query)
    args = parser.parse_args()
    return args.func(args)


if __name__ == "__main__":
    raise SystemExit(main())
