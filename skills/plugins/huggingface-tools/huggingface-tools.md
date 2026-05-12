---
name: huggingface-tools
description: >
  Use when importing a HuggingFace CLI token into Zorai or running DuckDB SQL
  over a HuggingFace dataset (joins, aggregations, or datasets the public
  datasets-server cannot index).
---

# HuggingFace Tools Plugin

Local helpers paired with the `huggingface` plugin.

## Auth flow

If the user has run `huggingface-cli login`:

```json
{"plugin_name": "huggingface-tools", "endpoint_name": "auth-import"}
```

This prints the on-disk token. Instruct the user to paste it into the `huggingface` plugin → `token` setting.

If the user has not logged in yet:

```json
{"plugin_name": "huggingface-tools", "endpoint_name": "auth-login"}
```

Runs `huggingface-cli login` interactively. Then run `auth-import`.

Use `auth-whoami` to confirm whose token is on disk before importing.

## DuckDB queries

The `query` and `query-schema` commands take their parameters as env vars (the plugin runtime wraps the command in `bash -c` with `set -euo pipefail`, and the agent sets `HF_DATASET`, `HF_CONFIG`, `HF_SPLIT`, `HF_SQL`, `HF_LIMIT` before execution).

The dataset's parquet files appear as the DuckDB view `data`. Always inspect schema first:

```json
{"plugin_name": "huggingface-tools", "endpoint_name": "query-schema", "params": {"HF_DATASET": "glue", "HF_CONFIG": "mrpc", "HF_SPLIT": "train"}}
```

Then run SQL:

```json
{"plugin_name": "huggingface-tools", "endpoint_name": "query", "params": {"HF_DATASET": "glue", "HF_CONFIG": "mrpc", "HF_SPLIT": "train", "HF_SQL": "SELECT label, COUNT(*) FROM data GROUP BY label"}}
```

Parquet files are cached after first download; subsequent queries are local.

## When to use this vs the `huggingface` plugin

- First reach for the `huggingface` plugin's `data-search` / `data-filter` / `data-stats`.
- Switch here when the user wants joins, GROUP BY, window functions, or `data-valid` reports `viewer: false`.
- For inference, deployment, or hub metadata, always use the `huggingface` plugin.
