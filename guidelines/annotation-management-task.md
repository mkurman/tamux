---
name: annotation-management-task
description: Design and manage annotation workflows — team structure, inter-annotator agreement targets per modality, active learning loop design, tooling comparison, and quality control at scale.
recommended_skills:
  - label-quality-audit
  - bias-audit
  - llm-assisted-curation
recommended_guidelines:
  - training-data-design-principles
  - dataset-creation-curation-task
---

## Overview

Annotation is the most expensive and error-prone part of dataset creation. A well-designed annotation pipeline produces consistent labels; a poorly designed one produces garbage at scale. This guideline covers workflow design, quality control, and tooling.

## Team Structure

| Role | Responsibility | Ratio |
|------|-------|-------|
| Annotator | Labels data following guidelines | 5-20 per project |
| Reviewer / QA | Checks annotator work, resolves edge cases | 1 per 5-10 annotators |
| Guideline author | Writes annotation instructions + examples | 1 per project |
| Adjudicator | Resolves annotator-reviewer disagreements | 1 per project (part-time) |
| Project manager | Throughput, budget, scheduling | 1 per project |

## Inter-Annotator Agreement Targets

| Task | Metric | Minimum | Target |
|------|-------|-------|-------|
| Binary classification | Cohen's κ | > 0.6 | > 0.8 |
| Multi-class (≤ 10 classes) | Fleiss' κ | > 0.5 | > 0.7 |
| NER (entity-level) | F1 between annotators | > 0.75 | > 0.85 |
| Bounding box detection | IoU | > 0.7 | > 0.9 |
| Segmentation mask | Dice | > 0.7 | > 0.85 |
| Sentiment (5-point) | Quadratic κ | > 0.6 | > 0.8 |
| Ranking (pairwise) | Kendall's τ | > 0.5 | > 0.7 |

## Annotation Guidelines

Every annotation task must have a written guideline with:

1. **Task definition** with positive and negative examples.
2. **Edge case rules**: what to do with ambiguous, borderline, or missing data.
3. **Calibration examples** (20-50) with gold-standard answers for training and recurring calibration.
4. **Glossary** of domain terms.
5. **Revision history** (guidelines evolve — track changes).

## Active Learning Loop

```python
# Annotation prioritization
def select_for_annotation(unlabeled_pool, model, n_select=100, strategy="uncertainty"):
    if strategy == "uncertainty":
        proba = model.predict_proba(unlabeled_pool)
        # Maximum entropy — most uncertain
        entropy = -np.sum(proba * np.log(proba + 1e-10), axis=1)
        return np.argsort(entropy)[-n_select:]
    
    elif strategy == "disagreement":
        # Query by committee — ensemble disagreement
        predictions = [m.predict(unlabeled_pool) for m in committee]
        disagreement = np.std(predictions, axis=0).mean(axis=1)
        return np.argsort(disagreement)[-n_select:]
    
    elif strategy == "diversity":
        # Coreset: select diverse examples covering embedding space
        embeddings = model.encode(unlabeled_pool)
        # Greedy k-center selection
        ...
```

## LLM-Assisted Annotation

- LLM produces draft labels → human reviewer accepts/rejects/corrects.
- **Measure acceptance rate**: If > 80% accepted, increase LLM autonomy. If < 50%, fix prompts.
- **Review rejection reasons**: Categorize errors — fix the prompt systematically.
- **Never ship LLM labels without human review** — unless you've validated on your specific data.

## Tooling

| Tool | Best For | Scale |
|------|-------|-------|
| **Label Studio** | General purpose, self-hosted | Small-large |
| **Prodigy** (spaCy) | NLP, active learning | Small-medium |
| **CVAT** | Detection, segmentation, tracking | Medium-large |
| **Labelbox** | Enterprise, active learning | Large |
| **Doccano** | Text annotation, open source | Small-medium |
| **Argilla** | NLP feedback, RLHF | Medium |

## Quality Gate

- Inter-annotator agreement measured and above minimum for all tasks.
- Calibration examples scored by all annotators before production work.
- Guidelines versioned with revision history.
- Rejected annotations tracked with reasons categorized.
- Annotation time recorded per task (for cost estimation).
