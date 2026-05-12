---
name: huggingface
description: >
  Use when the user wants to search, fetch, or operate on HuggingFace models,
  datasets, Spaces, dedicated Inference Endpoints, AutoTrain runs, or run
  queries on the HF datasets-server. Pair with the `huggingface-tools` plugin
  when DuckDB SQL or HF CLI token import is required.
---

# HuggingFace Plugin

Use the **HuggingFace plugin** for all HF Hub HTTP operations.

## Discovery

Use `search-models` / `search-datasets` / `search-spaces` to find assets.

```json
{"plugin_name": "huggingface", "endpoint_name": "search_models", "params": {"q": "bert", "filter": "pipeline_tag:fill-mask", "limit": 10}}
```

Common params: `q` (search string), `filter` (e.g. `pipeline_tag:text-classification`), `sort` (`downloads`, `likes`, `trending`), `limit` (default 20).

## Inspection

After a search, call `model` / `dataset` / `space` to get full metadata and the file list.

```json
{"plugin_name": "huggingface", "endpoint_name": "get_model", "params": {"id": "bert-base-uncased"}}
```

## Inference (serverless)

`run_inference` calls `api-inference.huggingface.co`. The `x-wait-for-model: true` header tells HF to block server-side instead of returning 503 while the model loads.

```json
{"plugin_name": "huggingface", "endpoint_name": "run_inference", "params": {"id": "distilbert-base-uncased-finetuned-sst-2-english", "inputs": "this is great"}}
```

`params.inputs` is a string. For complex payloads, use the model's documented `inputs` shape (e.g. a JSON string for tasks that accept structured input).

If you receive 503 after the wait-for-model header, retry once after 10 seconds.

## Async jobs

For long inference, submit via `job-submit` and poll `job-status`:

```json
{"plugin_name": "huggingface", "endpoint_name": "submit_async_inference", "params": {"id": "...", "inputs": "..."}}
```

## Dedicated endpoints

⚠️ **Confirm with the user before invoking `endpoint-create`, `endpoint-update`, `endpoint-resume`, or `endpoint-delete`. These incur charges or change billable state.**

`endpoint-create` and `endpoint-update` accept the full HF endpoint JSON via `params.body` and `params.patch` respectively — the plugin passes the JSON through verbatim, so the agent has full control over numeric scaling, custom hardware, etc.

Example body for `endpoint-create`:

```json
{"plugin_name": "huggingface", "endpoint_name": "create_endpoint", "params": {"namespace": "alice", "body": "{\"name\":\"my-endpoint\",\"type\":\"protected\",\"provider\":{\"vendor\":\"aws\",\"region\":\"us-east-1\"},\"model\":{\"repository\":\"bert-base-uncased\",\"revision\":\"main\",\"task\":\"fill-mask\",\"framework\":\"pytorch\"},\"compute\":{\"accelerator\":\"gpu\",\"instanceType\":\"nvidia-t4\",\"instanceSize\":\"small\",\"scaling\":{\"minReplica\":0,\"maxReplica\":1}}}"}}
```

Example patch for `endpoint-update` (scale up replicas):

```json
{"plugin_name": "huggingface", "endpoint_name": "update_endpoint", "params": {"namespace": "alice", "name": "my-endpoint", "patch": "{\"compute\":{\"scaling\":{\"minReplica\":1,\"maxReplica\":4}}}"}}
```

Lifecycle: `endpoint-create` → wait for `status.state == "running"` (poll with `endpoint`) → use the `status.url` for inference → `endpoint-pause` (cheap, fast resume) or `endpoint-delete` (frees the slot entirely).

Prefer `pause` over `delete` for short breaks.

## AutoTrain

⚠️ **Charges apply. Confirm before invoking `autotrain-create` or `autotrain-start`.**

`autotrain-create` and `autotrain-start` accept the full HF AutoTrain JSON via `params.body`. Construct the JSON for the desired task (`text-classification`, `llm-sft`, etc.) and pass it through.

Flow: `autotrain-create` (project + initial config) → `autotrain-start` (training run config) → poll `autotrain-run`.

## Datasets-server queries

Use `data-info` first to learn config/split names, then:

- `data-search` — full-text search across rows
- `data-filter` — predicate filter (SQL-like `WHERE` syntax, no joins/aggregations)
- `data-rows` — paginate raw rows
- `data-stats` — per-column statistics

Datasets-server caps result size. If you need real SQL (joins, GROUP BY, aggregations), or `data-valid` reports `viewer: false`, fall back to the **`huggingface-tools` plugin's `query` command** which runs DuckDB locally over the dataset's parquet files.

## Error patterns

- **401** — token missing or invalid. Run `/hf-tools auth-import` (or set `settings.token`).
- **429** — rate-limited. Respect `Retry-After`; retry at most once.
- **503 from inference** — model is cold-loading. Retry once after 10s.
- **404 with "gated"** — the asset requires acceptance on huggingface.co. Surface the message verbatim; do not retry.

## Usage guidance

- Always summarize results (don't dump the full JSON).
- Use `health` before write actions if you're uncertain about token validity.
- Quote model/dataset IDs with backticks in your replies.
