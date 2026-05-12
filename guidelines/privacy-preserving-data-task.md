---
name: privacy-preserving-data-task
description: Design privacy-preserving data pipelines — differential privacy (ε budgeting), k-anonymity, synthetic replacement thresholds, federated access, and secure computation boundaries. Engineering implementation, not just policy.
recommended_guidelines:
  - dataset-governance-task
  - synthetic-data-generation-task
  - training-data-design-principles
---

## Overview

Privacy is not a checkbox. It's a quantitative guarantee backed by mathematics. This guideline covers the engineering implementation of privacy-preserving data pipelines: DP budget management, anonymization verification, secure access patterns, and audit logging.

## Phase 1: Differential Privacy (DP)

### The Core Concept

A mechanism M satisfies (ε, δ)-differential privacy if for any two datasets differing by one record and any output S:

```
P[M(D) ∈ S] ≤ e^ε × P[M(D') ∈ S] + δ
```

- **ε (epsilon)** = privacy budget. Lower = more privacy. Typical: 0.1-10.
- **δ (delta)** = failure probability. Must be ≪ 1/n (n = dataset size). Typical: 10^-5 to 10^-7.

### ε Budget Selection

| ε Value | Privacy Level | Utility Cost | Use Case |
|-------|-------|-------|-------|
| 0.1-1.0 | Strong | High | Publishing aggregate statistics |
| 1.0-4.0 | Moderate | Medium | Training data release |
| 4.0-10.0 | Weak | Low | Internal analytics |
| > 10 | Minimal | Very Low | Debugging, exploration |

### DP Training Pipeline

```python
from opacus import PrivacyEngine

# DP-SGD training
model = MyModel()
optimizer = torch.optim.SGD(model.parameters(), lr=0.01)

privacy_engine = PrivacyEngine()
model, optimizer, data_loader = privacy_engine.make_private(
    module=model,
    optimizer=optimizer,
    data_loader=data_loader,
    noise_multiplier=1.1,    # higher = more privacy, less utility
    max_grad_norm=1.0,        # clip gradients to bound sensitivity
)

for epoch in range(epochs):
    for batch in data_loader:
        optimizer.zero_grad()
        loss = criterion(model(batch["x"]), batch["y"])
        loss.backward()
        optimizer.step()
    
    epsilon = privacy_engine.accountant.get_epsilon(delta=1e-5)
    print(f"Epoch {epoch}: ε = {epsilon:.2f}")
    if epsilon > target_epsilon:
        break  # Stop before exceeding budget
```

### ε Budget Accounting

- **Sequential composition**: Query 1 (ε₁) + Query 2 (ε₂) = total ε = ε₁ + ε₂.
- **Parallel composition**: Queries on DISJOINT data subsets don't sum.
- **Advanced composition**: For k queries, total ε ≈ √(2k ln(1/δ')) × ε + k × ε × (e^ε - 1).

**Never exceed the total budget**. Track it per-user, not just per-query.

## Phase 2: k-Anonymity

### Concept

A dataset satisfies k-anonymity if each record is indistinguishable from at least k-1 other records with respect to quasi-identifiers.

```python
def check_k_anonymity(df, quasi_identifiers, k=5):
    """
    Quasi-identifiers: columns that could identify someone when combined
    (age, zip code, gender — not name or SSN)
    """
    group_sizes = df.groupby(quasi_identifiers).size()
    
    violations = group_sizes[group_sizes < k]
    
    return {
        "k": k,
        "n_groups": len(group_sizes),
        "n_violations": len(violations),
        "violation_fraction": len(violations) / len(group_sizes),
        "min_group_size": group_sizes.min(),
        "max_group_size": group_sizes.max(),
        "is_k_anonymous": len(violations) == 0,
        "violation_examples": violations.head(5).to_dict() if len(violations) > 0 else {},
    }
```

### Limitations of k-Anonymity

- **Homogeneity attack**: All k records have the same sensitive value → attribute still revealed.
- **Background knowledge attack**: Attacker knows some attributes, narrows down possibilities.
- **Not sufficient alone**: Combine with l-diversity (sensitive values diverse) and t-closeness (distribution matches population).

## Phase 3: Synthetic Data for Privacy

### When Synthetic Data Provides Privacy

| Method | Privacy Guarantee | Utility |
|-------|-------|-------|
| DP-GAN (Differentially Private GAN) | (ε, δ)-DP | Medium-High |
| DP-CTGAN (for tabular) | (ε, δ)-DP | High for correlations |
| Standard GAN (no DP) | No formal guarantee | High |
| SDV synthesizers | No formal guarantee | High for statistics |

**Rule**: Without DP training, synthetic data has NO formal privacy guarantee. It CAN memorize training examples.

### Membership Inference Attack (Validate Your Privacy)

```python
def membership_inference_attack(target_model, train_data, holdout_data):
    """
    Can an adversary tell if a record was in the training set?
    If AUC > 0.6, the model memorized training data.
    """
    train_scores = target_model.get_confidence(train_data)
    holdout_scores = target_model.get_confidence(holdout_data)
    
    # Train attack classifier
    X_attack = np.concatenate([train_scores, holdout_scores])
    y_attack = np.concatenate([np.ones(len(train_scores)), np.zeros(len(holdout_scores))])
    
    from sklearn.metrics import roc_auc_score
    from sklearn.model_selection import cross_val_score
    from sklearn.ensemble import RandomForestClassifier
    
    attack_model = RandomForestClassifier()
    auc = cross_val_score(attack_model, X_attack.reshape(-1, 1), y_attack, cv=5, 
                          scoring="roc_auc").mean()
    
    return {
        "attack_auc": auc,
        "privacy_risk": "high" if auc > 0.7 else "moderate" if auc > 0.6 else "low",
    }
```

## Phase 4: Secure Access Patterns

### Tiered Access

| Tier | Who | What | How |
|------|-------|-------|-------|
| **Raw** | No one (for sensitive data) | Full PHI/PII | N/A |
| **De-identified** | Data engineers, selected researchers | Identifiers removed, dates shifted | VPC + access logs |
| **Aggregated** | Analysts | Counts, means, distributions | SQL views, dashboards |
| **DP query interface** | External partners | ε-budgeted queries | API with rate limiting |
| **Synthetic** | Everyone | Generated data | Standard download |

### Secure Enclave Pattern

```
┌─────────────────────────────────────┐
│  Secure Enclave (TEE)               │
│  ┌─────────┐  ┌─────────┐          │
│  │ Raw Data │→ │ Model   │→ Output  │  ← Only approved code
│  └─────────┘  │ Training│          │     enters this boundary
│               └─────────┘          │
└─────────────────────────────────────┘
          No raw data leaves
```

## Phase 5: Audit Logging

```python
import hashlib
import json
from datetime import datetime, timezone

def log_data_access(user_id, action, dataset_id, records_accessed, purpose):
    entry = {
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "user_id": user_id,
        "action": action,  # "train", "query", "export", "delete"
        "dataset_id": dataset_id,
        "n_records": records_accessed,
        "purpose": purpose,
        "dp_epsilon_consumed": None,  # filled if DP query
        "entry_hash": None,
    }
    entry["entry_hash"] = hashlib.sha256(json.dumps(entry, sort_keys=True).encode()).hexdigest()
    # Append to immutable audit log
    return entry
```

## Quality Gate

- ε budget defined and enforced by the training pipeline (hard stop).
- k-anonymity checked for any published data.
- Membership inference attack AUC < 0.6 for any released model.
- Access tier appropriate for data sensitivity.
- All data access logged with purpose justification.
- Re-identification risk assessed and documented.
