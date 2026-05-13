---
name: industry-verticals-data-task
description: Curate datasets for industry-specific ML — security/threat intelligence, financial/trading (survivorship bias, look-ahead), supply chain/logistics, energy/grid management, and insurance/actuarial. Domain-specific failure modes that general guidelines miss.
recommended_skills: [dataset-splitting, data-contamination-task, embedding-analysis, bias-audit]
recommended_guidelines: [training-data-design-principles, evaluation-dataset-design-task, cost-model-task]
---

## Overview

General data curation guidelines fail in industry because every vertical has unique failure modes. Security data has false-positive tolerance tradeoffs. Financial data has survivorship bias and look-ahead leakage. Supply chain data has error propagation. Energy data has weather dependence. Insurance data has claim frequency tail risk. This guideline covers the patterns specific to each vertical.

## Phase 1: Security / Threat Intelligence

### Threat Data Validation

```python
def validate_threat_intel(threat_data, ref_window_days=30):
    """Validate threat intelligence freshness and accuracy."""
    now = datetime.now(timezone.utc)
    staleness = [(now - t.get("last_seen", now)).days for t in threat_data]
    
    return {
        "n_threats": len(threat_data),
        "fresh_rate": np.mean([s < ref_window_days for s in staleness]),
        "median_staleness_days": np.median(staleness),
        "stale_threats": sum(1 for s in staleness if s > 90),
        "action": "ARCHIVE_STALE" if np.mean(staleness) > 60 else "MONITOR",
    }

def false_positive_calibration(alerts, confirmed_threats, tolerance=0.05):
    """Security is a tradeoff: more detection = more false positives."""
    tp = len(set(a["id"] for a in alerts) & set(t["alert_id"] for t in confirmed_threats))
    fp = len(alerts) - tp
    fpr = fp / max(len(alerts), 1)
    
    return {"false_positive_rate": fpr, "acceptable": fpr <= tolerance,
            "recommendation": "RAISE_THRESHOLD" if fpr > tolerance * 2 
            else "MAINTAIN" if fpr <= tolerance else "REVIEW"}
```

## Phase 2: Financial / Trading

### Survivorship Bias Detection

```python
def audit_survivorship_bias(training_universe, current_universe, training_dates, current_date):
    """Assets that exist now but delisted between training and now = survivorship bias."""
    delisted = set(training_universe) - set(current_universe)
    
    if delisted:
        return {"bias": "SURVIVORSHIP", "severity": "CRITICAL",
                "delisted_assets": len(delisted), "pct_of_universe": len(delisted) / max(len(training_universe), 1),
                "action": "REMOVE_DELISTED_FROM_TRAINING"}
    return {"bias": "NONE", "severity": "clean"}

def detect_look_ahead_bias(features_df, target_col, feature_cols, max_lag=5):
    """Detect features that leak future target information."""
    suspicious = []
    for col in feature_cols:
        for lag in range(1, max_lag + 1):
            corr = features_df[col].corr(features_df[target_col].shift(-lag))
            if abs(corr) > 0.7:
                suspicious.append({"feature": col, "lag": lag, "correlation": float(corr)})
    return suspicious
```

### Market Regime Detection

```python
def detect_market_regime(returns, window=60):
    """Label market regimes for stratified evaluation."""
    vol = returns.rolling(window).std()
    regime = pd.cut(vol, bins=[0, vol.quantile(0.33), vol.quantile(0.67), float("inf")],
                     labels=["low_vol", "normal", "high_vol"])
    return {"regime_distribution": regime.value_counts(normalize=True).to_dict(),
            "current_regime": str(regime.iloc[-1])}
```

## Phase 3: Supply Chain / Logistics

### Error Propagation Modeling

```python
def supply_chain_error_propagation(nodes, edges, error_sources):
    """Model how errors at one node propagate downstream."""
    import networkx as nx
    G = nx.DiGraph()
    G.add_nodes_from(nodes)
    G.add_edges_from(edges)
    
    propagation = {}
    for source_node, error_rate in error_sources.items():
        descendants = nx.descendants(G, source_node)
        for desc in descendants:
            path_length = nx.shortest_path_length(G, source_node, desc)
            propagated_error = error_rate * (0.8 ** path_length)
            propagation[(source_node, desc)] = propagated_error
    
    return {"propagation_map": propagation,
            "most_vulnerable_node": max(propagation, key=propagation.get)[1]}
```

## Phase 4: Energy / Grid Management

### Weather-Dependent Load Validation

```python
def validate_load_forecast(actual_load, forecast_load, weather_data):
    """Separate forecast error into weather-dependent and weather-independent components."""
    total_error = np.mean(np.abs(actual_load - forecast_load))
    weather_corr = forecast_load.corr(weather_data["temperature"])
    
    return {"total_mape": float(total_error),
            "weather_sensitivity": float(weather_corr),
            "recommendation": "IMPROVE_WEATHER_MODEL" if abs(weather_corr) > 0.5 and total_error > 0.1 
            else "ACCEPTABLE"}
```

## Phase 5: Insurance / Actuarial

### Claim Frequency Tail Risk

```python
def assess_tail_risk(claim_amounts, threshold_pct=0.99):
    """Does the model capture extreme claims?"""
    threshold = np.quantile(claim_amounts, threshold_pct)
    tail = claim_amounts[claim_amounts > threshold]
    
    return {"tail_threshold": float(threshold), "n_tail_claims": len(tail),
            "tail_share_of_total": float(tail.sum() / claim_amounts.sum()),
            "tail_underestimation_risk": "HIGH" if tail.sum() / claim_amounts.sum() > 0.3 else "MODERATE"}
```

### Catastrophe Model Validation

```python
def validate_cat_model(predicted_losses, actual_losses, events):
    """Compare catastrophe model predictions to historical events."""
    errors = []
    for event_id, pred, actual in zip(events, predicted_losses, actual_losses):
        error = abs(pred - actual) / max(actual, 1)
        errors.append({"event": event_id, "error": float(error), 
                        "severity": "review" if error > 0.5 else "acceptable"})
    
    return {"mean_error": float(np.mean([e["error"] for e in errors])),
            "n_events": len(events), "requires_recalibration": np.mean([e["error"] for e in errors]) > 0.3}
```

## Quality Gate

- Security: FPR within tolerance for the threat type.
- Financial: Survivorship bias audit clean; look-ahead bias scan clean.
- Supply chain: Error propagation mapped; vulnerable nodes identified.
- Energy: Weather-dependent error component quantified.
- Insurance: Tail risk assessed; catastrophe model validated against ≥ 5 historical events.
