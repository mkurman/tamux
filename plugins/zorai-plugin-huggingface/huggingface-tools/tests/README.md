# Tests

Re-run the Python tests:

```bash
cd plugins/zorai-plugin-huggingface/huggingface-tools
python3 -m venv .venv
.venv/bin/pip install duckdb pyarrow pytest
.venv/bin/python -m pytest tests/ -v
```

Expected: 6 passed (3 in `test_hf_auth.py`, 3 in `test_hf_query.py`).

The `.venv/` directory is gitignored and **must not** be left in place when running `zorai plugin add` — the installer copies the plugin directory verbatim, and a 240+ MB venv inside the plugin dir bloats the install. Delete `.venv` before installing, or use a venv outside the plugin tree.
