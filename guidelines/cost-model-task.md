---
name: cost-model-task
description: Estimate dataset costs — collection, annotation, cleaning, hosting, versioning. Build vs. buy vs. license vs. generate synthetic. With dollar figures and ROI frameworks.
recommended_guidelines:
  - annotation-management-task
  - dataset-governance-task
  - synthetic-data-generation-task
---

## Overview

Data decisions are cost decisions. This guideline provides frameworks and ballpark figures.

## Cost Components

| Component | Unit | Range | Notes |
|------|-------|-------|-------|
| **Collection** | Per example | $0.01-$100+ | Web scrape ($0.001) → clinical trial ($100+) |
| **Annotation** | Per example | $0.10-$50+ | Binary label ($0.10) → medical segmentation ($50+) |
| **Expert review** | Per hour | $50-$300/hr | Domain-specific |
| **Cleaning / QC** | Per example | $0.001-$0.50 | Automated (cheap) → manual (expensive) |
| **Hosting (S3)** | Per TB/month | $23/TB | Standard tier |
| **Hosting (DVC remote)** | Per TB/month | $23-$50/TB | Cloud-dependent |
| **Versioning overhead** | Per version | 10-30% of base storage | Diffs + metadata |
| **Pipeline compute** | Per run | $10-$10K | Small CSV ($10) → 100M embeddings ($10K) |

## Decision Framework

### Build vs. Buy vs. License vs. Generate

| Option | Upfront Cost | Per-Example Cost | Quality | Time |
|------|-------|-------|-------|-------|
| **Build from scratch** | High (pipeline + infra) | Medium | Highest (controlled) | Months |
| **Buy off-the-shelf** | Low-Medium | $0.05-$5/example | Medium-High | Days |
| **License (academic)** | Free-$10K/year | $0 | Varies | Days |
| **Generate synthetic** | Medium (compute) | $0.001-$0.10/example | Medium (check quality) | Hours-Days |
| **Crowdsource** | Medium (platform) | $0.01-$0.50/example | Medium | Weeks |

### When Each Makes Sense

- **Build**: Unique domain, no existing dataset, high quality bar, regulatory requirements.
- **Buy**: Commodity task (OCR, sentiment, NER), time-constrained.
- **License**: Academic research, well-studied domain, great public datasets available.
- **Generate**: Class imbalance, privacy constraints, edge cases.
- **Crowdsource**: Large volume needed, task is clear and unambiguous, budget for QC.

## Annotation Cost Estimation

```
Total Cost = N_examples × (annotator_cost + reviewer_cost × reviewer_ratio + platform_fee) + management_overhead

Typical breakdown per 10,000 examples:
Annotator labor:   $0.50 × 10,000 = $5,000
Reviewer labor:    $1.00 × 2,000 (20% review) = $2,000
Platform/overhead: 30% = $2,100
Management:        15% = $1,365
─────────────────────────────────
Total:             ~$10,465 ($1.05/example)
```

## ROI Framework

```
ROI = (model_improvement × business_value) / dataset_cost

Model improvement: Δ in primary metric (e.g., +3% accuracy)
Business value: $ value of 1% improvement × number of decisions/year
Dataset cost: all-in cost of creating/maintaining the dataset

Example:
  +2% accuracy in fraud detection
  $500K saved per 1% accuracy gain
  Dataset cost: $50K
  → ROI = (2 × $500K) / $50K = 20x
```

## Budget Template

```markdown
# Dataset Budget
## Collection: $X
## Annotation: $X
  - Annotators (N × rate × hours): $X
  - Reviewers (N × rate × hours): $X
  - Platform fees: $X
## Cleaning & QC: $X
## Storage & Hosting (1 year): $X
## Governance & Legal: $X
## Pipeline Engineering: $X
## Contingency (20%): $X
---
Total: $X
```
