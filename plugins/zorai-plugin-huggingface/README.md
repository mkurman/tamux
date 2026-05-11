# zorai-plugin-huggingface

HuggingFace integration for Zorai. Ships two plugins from one npm package:

- **`huggingface`** — API-backed. Search/fetch models, datasets, Spaces. Run serverless inference. Submit and poll async inference jobs. List/create/update/pause/resume/delete dedicated Inference Endpoints with full body control. List/create/start AutoTrain projects and runs. Query the public `datasets-server` (info, splits, size, rows, search, filter, statistics, validity).
- **`huggingface-tools`** — Python-backed (uses `uv`). Imports an existing `huggingface-cli login` token for pasting into the `huggingface` plugin. Runs DuckDB SQL over a dataset's parquet files (joins, aggregations, anything beyond the datasets-server query surface).

## Install

```bash
zorai plugin add ./plugins/zorai-plugin-huggingface
```

Both plugins are registered. Verify:

```bash
zorai plugin ls            # both huggingface and huggingface-tools listed
zorai plugin commands      # 37 commands total: 32 under /huggingface.*, 5 under /huggingface-tools.*
```

## Auth

The `huggingface` plugin's `token` setting authenticates HF API calls. Three ways to fill it in:

- **Already ran `huggingface-cli login`:** `/hf-tools auth-import` prints the on-disk token; paste it into the plugin settings.
- **Haven't logged in yet:** `/hf-tools auth-login` runs the CLI interactively, then run `/hf-tools auth-import`.
- **Direct paste:** generate a token at <https://huggingface.co/settings/tokens> and paste into settings. The `huggingface-tools` plugin is optional for this path.

Verify with `/hf health` after the token is set.

## Commands

See the agent skills for full guidance on each command — the agent reads them automatically when the plugins are enabled:

- `plugins/zorai-plugin-huggingface/huggingface/skills/huggingface.md`
- `plugins/zorai-plugin-huggingface/huggingface-tools/skills/huggingface-tools.md`

## Billing-sensitive operations

The following commands incur real charges on your HuggingFace account. **Always confirm with the user before invoking:**

- `/hf endpoint-create` — creates a dedicated Inference Endpoint. Accepts the full HF endpoint JSON via the `body` param (numeric scaling, custom hardware, all of it).
- `/hf endpoint-update` — updates an existing endpoint. Accepts a partial JSON patch via the `patch` param.
- `/hf endpoint-resume` — resumes a paused endpoint (compute resumes immediately).
- `/hf autotrain-create` — creates an AutoTrain project. Accepts the full AutoTrain project JSON via the `body` param.
- `/hf autotrain-start` — starts an AutoTrain run. Accepts the training-config JSON via the `body` param.

`/hf endpoint-pause` and `/hf endpoint-delete` stop billing but cannot be undone automatically — confirm with the user.

## Known caveats

- **AutoTrain and async-inference path stability** — HF has revised these surfaces over time. The current manifest reflects design-time assumptions (`api.autotrain.huggingface.co/projects/...` and `api-inference.huggingface.co/async/...`). If `/hf autotrain-list` or `/hf job-status` returns 404, verify against current HF docs and update the endpoint paths in `huggingface/plugin.json`.

## License

MIT.
