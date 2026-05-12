---
name: time-series-data-task
description: Curate time-series datasets — irregular sampling, seasonality detection, stationarity tests, lag feature leakage prevention, walk-forward validation, and change-point detection.
recommended_skills:
  - darts
  - nixtla
  - prophet
  - dataset-splitting
  - embedding-analysis
recommended_guidelines:
  - training-data-design-principles
  - data-contamination-task
---

## Overview

Time series data destroys standard ML assumptions. IID? Gone. Random splits? Leakage. Stationarity? Maybe not. This guideline covers the unique curation challenges: temporal integrity, sampling irregularity, and splitting that respects causality.

## Phase 1: Temporal Integrity

### The Cardinal Rule

**Never shuffle. Never random-split. The past trains, the future tests.**

### Temporal Leakage Detection

```python
def audit_temporal_integrity(df, time_col, group_col=None):
    issues = []
    
    # 1. Check ordering
    if not df[time_col].is_monotonic_increasing:
        if group_col:
            for gid, grp in df.groupby(group_col):
                if not grp[time_col].is_monotonic_increasing:
                    issues.append({
                        "type": "non_monotonic",
                        "group": gid,
                        "severity": "critical",
                    })
        else:
            issues.append({"type": "non_monotonic", "severity": "critical"})
    
    # 2. Check gaps
    if group_col:
        for gid, grp in df.groupby(group_col):
            diffs = grp[time_col].diff().dropna()
            if len(diffs) > 0:
                expected_gap = diffs.mode().iloc[0] if len(diffs.mode()) > 0 else diffs.median()
                large_gaps = diffs[diffs > 5 * expected_gap]
                if len(large_gaps) > 0:
                    issues.append({
                        "type": "large_gaps",
                        "group": gid,
                        "n_gaps": len(large_gaps),
                    })
    
    # 3. Check duplicate timestamps
    if group_col:
        dups = df.duplicated(subset=[group_col, time_col]).sum()
    else:
        dups = df.duplicated(subset=[time_col]).sum()
    if dups > 0:
        issues.append({"type": "duplicate_timestamps", "n": dups})
    
    return issues
```

## Phase 2: Stationarity

```python
from statsmodels.tsa.stattools import adfuller, kpss

def test_stationarity(series, alpha=0.05):
    results = {}
    
    # ADF test: H0 = non-stationary (unit root)
    adf = adfuller(series.dropna(), autolag="AIC")
    results["adf_statistic"] = adf[0]
    results["adf_pvalue"] = adf[1]
    results["adf_stationary"] = adf[1] < alpha
    
    # KPSS test: H0 = stationary (trend-stationary)
    kp = kpss(series.dropna(), regression="c", nlags="auto")
    results["kpss_statistic"] = kp[0]
    results["kpss_pvalue"] = kp[1]
    results["kpss_stationary"] = kp[1] > alpha
    
    # Both agree = confident. Disagree = borderline.
    results["verdict"] = (
        "stationary" if results["adf_stationary"] and results["kpss_stationary"]
        else "non_stationary" if not results["adf_stationary"] and not results["kpss_stationary"]
        else "borderline"
    )
    return results
```

## Phase 3: Seasonality Detection

```python
from scipy.signal import periodogram

def detect_seasonality(series, candidates=[24, 168, 720]):
    """Detect seasonality at candidate periods (hourly, weekly, monthly)."""
    if len(series) < max(candidates) * 2:
        return {}
    
    f, Pxx = periodogram(series.dropna(), detrend="linear")
    
    results = {}
    for period in candidates:
        freq_idx = np.argmin(np.abs(f - 1/period))
        if freq_idx < len(Pxx):
            # Check if power at candidate frequency exceeds neighbors
            neighborhood = Pxx[max(0, freq_idx-3):freq_idx+4]
            peak_power = Pxx[freq_idx]
            mean_power = neighborhood.mean()
            results[period] = {
                "power_ratio": float(peak_power / (mean_power + 1e-10)),
                "has_seasonality": peak_power > 3 * mean_power,
            }
    return results
```

## Phase 4: Walk-Forward Validation

```python
from sklearn.model_selection import TimeSeriesSplit

def walk_forward_splits(df, time_col, n_splits=5, gap=0):
    """
    gap: number of timesteps between train and test (buffer zone).
    Essential for financial data where adjacent periods leak information.
    """
    df = df.sort_values(time_col).reset_index(drop=True)
    tscv = TimeSeriesSplit(n_splits=n_splits, gap=gap)
    
    splits = []
    for fold, (train_idx, test_idx) in enumerate(tscv.split(df)):
        splits.append({
            "fold": fold,
            "train_start": df[time_col].iloc[train_idx[0]],
            "train_end": df[time_col].iloc[train_idx[-1]],
            "test_start": df[time_col].iloc[test_idx[0]],
            "test_end": df[time_col].iloc[test_idx[-1]],
            "n_train": len(train_idx),
            "n_test": len(test_idx),
        })
        # Verify no temporal overlap
        assert df[time_col].iloc[train_idx].max() < df[time_col].iloc[test_idx].min()
    
    return splits

# Key: expanding window vs. sliding window
# Expanding: train set grows over folds (more data, higher compute)
# Sliding: fixed train window size (adapts to concept drift)
```

## Phase 5: Lag Feature Leakage

```python
def audit_lag_leakage(df, time_col, target_col, max_lag=48):
    suspicious = []
    
    for col in df.columns:
        if col == target_col or col == time_col:
            continue
        
        # Check if feature contains shifted target information
        for lag in range(1, max_lag + 1):
            corr = df[col].corr(df[target_col].shift(lag))
            if abs(corr) > 0.8:
                suspicious.append({
                    "feature": col,
                    "target_lag": lag,
                    "correlation": float(corr),
                    "suspicion": "feature_leaks_target",
                })
        
        # Check if feature name suggests future knowledge
        future_keywords = ["future", "next", "forward", "after", "lead", "t+"]
        if any(kw in col.lower() for kw in future_keywords):
            suspicious.append({
                "feature": col,
                "suspicion": "name_suggests_future_knowledge",
            })
    
    return suspicious
```

## Phase 6: Irregular Sampling

```python
def characterize_sampling(series, time_index):
    """Characterize the sampling pattern."""
    diffs = np.diff(time_index)
    
    return {
        "n_observations": len(series),
        "mean_interval": np.mean(diffs),
        "median_interval": np.median(diffs),
        "std_interval": np.std(diffs),
        "min_interval": np.min(diffs),
        "max_interval": np.max(diffs),
        "cv_interval": np.std(diffs) / np.mean(diffs),
        "is_regular": np.std(diffs) / np.mean(diffs) < 0.05,
        "missingness_periods": _find_gaps(time_index, expected_freq=pd.Timedelta(np.median(diffs))),
    }
```

## Phase 7: Change-Point Detection

```python
import ruptures as rpt

def detect_change_points(series, model="l2", min_size=10):
    algo = rpt.Pelt(model=model, min_size=min_size).fit(series.dropna().values)
    change_points = algo.predict(pen=10)
    return {
        "n_change_points": len(change_points) - 1,
        "positions": change_points[:-1],
        "segments": len(change_points),
    }
```

## Quality Gate

- Temporal integrity verified: monotonic, no gaps, no duplicate timestamps.
- Stationarity tested; non-stationary series documented with differencing strategy.
- Seasonality detected and documented.
- Walk-forward split verified: all train dates < all test dates.
- Lag leakage audit clean: no feature correlates with future target.
- Irregular sampling characterized; imputation or modeling strategy documented.
