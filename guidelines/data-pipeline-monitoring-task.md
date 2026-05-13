---
name: data-pipeline-monitoring-task
description: Monitor data pipelines for drift, schema evolution, freshness, and quality regressions — and operationalize with automated alerts and backfill protocols.
recommended_skills:
  - data-diff
  - embedding-analysis
  - dataset-versioning
recommended_guidelines:
  - dataset-creation-curation-task
  - training-data-design-principles
---

## Overview

Data pipelines rot. Schemas drift. Distributions shift. Upstream sources change format without notice. Monitoring catches these failures before they silently corrupt downstream models.

## Monitoring Signals

### Schema Drift

| Signal | Detection | Alert |
|------|-------|-------|
| New column appears | Schema diff | Warn — may be intentional |
| Column removed | Schema diff | Alert — downstream breakage |
| Type change (int→float) | Schema diff | Warn — check semantics |
| Type change (int→string) | Schema diff | Alert — likely error |
| Null rate spike (> 5pp increase) | Per-column null count | Alert — upstream change |

### Distribution Drift

```python
from scipy.stats import ks_2samp, wasserstein_distance

def detect_drift(reference, current, threshold=0.01):
    """KS test for distribution drift."""
    # Compare reference window (e.g., last 7 days) to current window
    drift_metrics = {}
    for col in reference.select_dtypes(include=np.number).columns:
        ks_stat, p_val = ks_2samp(
            reference[col].dropna(), current[col].dropna()
        )
        w_dist = wasserstein_distance(
            reference[col].dropna(), current[col].dropna()
        )
        drift_metrics[col] = {
            "ks_statistic": ks_stat,
            "p_value": p_val,
            "wasserstein": w_dist,
            "drift_detected": p_val < threshold,
        }
    return drift_metrics
```

### Freshness

| Metric | How to Measure | Alert |
|------|-------|-------|
| Time since last update | `now - max(timestamp)` | > SLA |
| Staleness of source | Last source commit timestamp | > 2x expected interval |
| Pipeline latency | `completion_time - start_time` | > 2x historical p95 |

### Volume

| Signal | Detection | Alert |
|------|-------|-------|
| Row count drop | `n_rows / rolling_avg` < 0.5 | Alert — upstream failure |
| Row count spike | `n_rows / rolling_avg` > 2.0 | Warn — check for duplicates |
| Empty partitions | `n_rows == 0` for any partition | Alert — data outage |

## Operationalization

### Automated Checks (Every Pipeline Run)

```python
PIPELINE_CHECKS = {
    "schema_match": lambda ref, cur: list(ref.columns) == list(cur.columns),
    "no_empty_rows": lambda df: len(df) > 0,
    "null_rate_below_threshold": lambda df, threshold=0.5: 
        (df.isnull().mean() < threshold).all(),
    "key_uniqueness": lambda df, key: df[key].is_unique,
    "timestamp_monotonic": lambda df, ts_col: 
        df[ts_col].is_monotonic_increasing,
}
```

### Backfill Protocol

1. Detect gap (missing date range).
2. Check upstream source availability for that range.
3. Replay pipeline for gap range with same code version.
4. Validate backfilled data against expected row counts.
5. Append to dataset; bump patch version.

## Quality Gate

- Schema diff runs on every pipeline execution.
- Distribution drift checked weekly on key numeric columns.
- Freshness alerts trigger within 2x expected interval.
- Backfill protocol documented and tested.
- Pipeline run metadata (version, start/end time, row counts) is logged.
