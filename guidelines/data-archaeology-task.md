---
name: data-archaeology-task
description: Reconstruct and validate legacy datasets with unknown provenance — schema archaeology, corruption recovery, bias discovery, historical context reconstruction, and migration trauma detection.
recommended_skills: [data-diff, exploratory-data-analysis, embedding-analysis, label-quality-audit]
recommended_guidelines: [dataset-creation-curation-task, data-contamination-task]
---

## Overview

Not all data comes with provenance. Legacy datasets arrive as mysterious CSV files with no documentation, corrupted encodings, and unknown collection methodologies. Data archaeology reconstructs what this data IS, how it was collected, and whether it's safe to use.

## Phase 1: Schema Archaeology

```python
def reconstruct_schema(df):
    """Infer schema from data when no documentation exists."""
    schema = {"inferred_types": {}, "inferred_constraints": {}, "anomalies": []}
    
    for col in df.columns:
        # Type inference
        if df[col].dtype == object:
            sample = df[col].dropna().head(100)
            if sample.str.match(r"^\d{4}-\d{2}-\d{2}").all(): detected_type = "date"
            elif sample.str.match(r"^[A-Z]\d{1,2}(\.\d+)?").all(): detected_type = "code"
            elif sample.str.len().mean() > 50: detected_type = "text"
            else: detected_type = "categorical"
        elif df[col].dtype in (np.int64, np.float64):
            if len(df[col].unique()) < 20: detected_type = "categorical_encoded"
            elif df[col].min() >= 0 and df[col].max() <= 1: detected_type = "binary_or_probability"
            else: detected_type = "numeric"
        schema["inferred_types"][col] = detected_type
        
        # Constraint inference
        if detected_type == "numeric":
            schema["inferred_constraints"][col] = {"min": float(df[col].min()), "max": float(df[col].max()),
                "mean": float(df[col].mean()), "null_rate": float(df[col].isnull().mean())}
    
    # Known unknown columns (high entropy, no structure)
    for col in df.columns:
        if df[col].nunique() / len(df) > 0.95 and df[col].dtype == object:
            schema["anomalies"].append({"column": col, "anomaly": "high_entropy_text", 
                                         "likely": "free_text_field_or_id"})
    
    return schema
```

## Phase 2: Corruption Detection & Recovery

```python
CORRUPTION_PATTERNS = {
    "double_encoded_utf8": lambda s: "Ã" in str(s) and "Â" in str(s),
    "mojibake": lambda s: any(ord(c) in range(0x80, 0xA0) for c in str(s)),
    "truncated": lambda col: col.str.len().value_counts().index[:3].tolist() if col.dtype == object else [],
    "encoding_mismatch": lambda df: _detect_encoding_mismatch(df),
}

def recover_corrupted_text(text):
    """Attempt to recover corrupted text through multiple encoding passes."""
    if CORRUPTION_PATTERNS["double_encoded_utf8"](text):
        try: return text.encode("latin1").decode("utf-8")
        except: pass
    if CORRUPTION_PATTERNS["mojibake"](text):
        try: return text.encode("cp1252").decode("utf-8")
        except: pass
    return text  # unrecoverable
```

## Phase 3: Legacy Bias Discovery

```python
def discover_legacy_bias(df, protected_attrs, target_col, reference_year):
    """What biases did original creators unknowingly embed?"""
    biases = []
    for attr in protected_attrs:
        if attr not in df.columns: continue
        groups = df.groupby(attr)
        rates = groups[target_col].mean()
        disparities = rates - rates.mean()
        
        for group, disparity in disparities.items():
            if abs(disparity) > 0.1:
                biases.append({"attribute": attr, "group": group, "disparity": float(disparity),
                                "direction": "advantaged" if disparity > 0 else "disadvantaged"})
    
    # Temporal bias: did collection methodology change over time?
    time_cols = [c for c in df.columns if "date" in c.lower() or "year" in c.lower()]
    for tc in time_cols:
        if tc in df.columns:
            early = df[df[tc] < df[tc].median()][target_col].mean()
            late = df[df[tc] >= df[tc].median()][target_col].mean()
            if abs(early - late) > 0.05:
                biases.append({"attribute": "collection_period", "early": float(early), "late": float(late),
                                "shift": float(late - early)})
    
    return biases
```

## Phase 4: Migration Trauma Detection

```python
def detect_migration_trauma(current_df, format_history):
    """What was lost in format conversions?"""
    trauma = []
    for fmt_from, fmt_to in zip(format_history[:-1], format_history[1:]):
        losses = MIGRATION_LOSS_PATTERNS.get((fmt_from, fmt_to), [])
        for loss in losses:
            if loss["column"] in current_df.columns:
                if loss["type"] == "precision_loss":
                    trauma.append({"from": fmt_from, "to": fmt_to, "loss": loss,
                                    "recoverable": False})
                elif loss["type"] == "encoding_loss":
                    # Check for replacement characters
                    affected = current_df[loss["column"]].astype(str).str.contains("�").sum()
                    if affected > 0:
                        trauma.append({"from": fmt_from, "to": fmt_to, "loss": loss,
                                        "affected_rows": int(affected), "recoverable": True if fmt_from == "csv" else False})
    return trauma

MIGRATION_LOSS_PATTERNS = {
    ("csv", "parquet"): [{"column": None, "type": "precision_loss", "detail": "float precision may degrade"}],
    ("excel", "csv"): [{"column": None, "type": "encoding_loss", "detail": "special characters may corrupt"}],
    ("json", "csv"): [{"column": None, "type": "structure_loss", "detail": "nested objects flattened"}],
}
```

## Quality Gate

- Schema reconstructed with documented uncertainty for every column.
- Corruption detected and recovery attempted; unrecoverable data flagged.
- Legacy biases discovered and documented.
- Migration trauma catalogued with affected rows counted.
- Dataset provenance documented as "reconstructed from [original format] via [migration path]" with known limitations.
