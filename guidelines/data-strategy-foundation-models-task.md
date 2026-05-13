---
name: data-strategy-foundation-models-task
description: Design data strategy for foundation models — mixing ratios, scaling laws for data, redundancy saturation curves, and what to do with $100M for data.
recommended_guidelines:
  - training-data-design-principles
  - llm-training-data-task
  - dataset-governance-task
  - cost-model-task
---

## Overview

Foundation model data strategy is capital allocation disguised as data science. The question isn't "what's the best dataset" — it's "given budget B, compute C, and timeline T, what data mix maximizes downstream capability?"

## Scaling Laws for Data

### Chinchilla Optimal

Given a compute budget C:
- Optimal tokens = optimal parameters (roughly).
- D = N / 20: training tokens ≈ 20× model parameters.
- Over-training (more data than Chinchilla) produces smaller, more capable models for inference.

### Redundancy Saturation

Not all tokens are equal value:

| Data Type | Saturation Point | Value per Token |
|------|-------|-------|
| High-quality curated (books, papers) | Never saturates at practical scale | 10-100× web text |
| Filtered web (high quality) | Saturates at ~10 epochs | 1× (baseline) |
| Noisy web | Saturates at ~1-2 epochs | 0.1-0.5× |
| Code | Saturates slowly | 2-5× |
| Math/reasoning | Near-zero saturation | 5-20× |
| Synthetic | Saturates at ~1 epoch (distribution collapse) | 0.5-2× |

## Budget Allocation Framework ($100M Example)

| Category | % | $M | What It Buys |
|------|-------|-------|-------|
| **Web crawl + filter** | 15% | $15M | Common Crawl processing, 100-500B clean tokens |
| **Licensed high-quality** | 25% | $25M | Books, academic papers, code repos |
| **Annotation + curation** | 25% | $25M | Instruction data, RLHF preferences, domain experts |
| **Synthetic generation** | 5% | $5M | Compute for generation + validation |
| **Pipeline + infra** | 15% | $15M | Dedup, filtering, hosting, versioning |
| **Governance + legal** | 10% | $10M | Licensing, consent, EU AI Act compliance |
| **Contingency** | 5% | $5M | Unexpected license issues, re-collection |

## Mixing Strategy

### Pre-Training Mix (General Model)

```
Web text:       40-50%  (broad coverage, base language)
Code:           15-20%  (reasoning, structured thinking)
Books:          10-15%  (long-form coherence, narrative)
Academic:       10-15%  (factual knowledge, formal reasoning)
Conversation:    3-5%   (dialogue capability)
Multilingual:    5-10%  (cross-lingual transfer)
```

### Domain-Specific CPT Mix

```
Domain data:     70-80%  (the domain you're adapting to)
Replay buffer:   20-30%  (general data to prevent forgetting)
High-quality:     5-10%  (curated gems — disproportionate impact)
```

### Instruction Tuning Mix

```
Diverse tasks:   40%     (breadth of capabilities)
Difficult tasks: 30%     (reasoning, multi-step, expert)
Safety:          15%     (refusals, harmlessness)
Format following: 10%    (structured output, tool use)
Identity/chat:    5%     (persona, conversational style)
```

## Quality vs. Quantity

**The Pareto Principle of Data**: 20% of your data contributes 80% of model capability. The art is finding that 20%.

| Strategy | When | Risk |
|------|-------|-------|
| **Filter aggressively** | When downstream tasks need precision | May lose diversity |
| **Keep everything, upsample quality** | When coverage matters most | Noisy signal dilutes |
| **Iterative filtering** | Best: filter, train small model, use model to find more quality data | Computationally expensive |
| **Data pruning** (Sorscher et al.) | Remove easiest examples — they don't teach anything | Pruning metric must match goal |

## What NOT to Do

1. Don't scale noisy data hoping quality emerges from volume. It doesn't.
2. Don't mix sources without deduplication between them.
3. Don't train on benchmarks. Scan for contamination before every run.
4. Don't treat all tokens as equal value. Upsample high-quality domains.
5. Don't ignore data documentation. Without provenance, you can't debug failures.
6. Don't use synthetic data as more than 10-15% of pre-training mix.
7. Don't skip the replay buffer in CPT — catastrophic forgetting is real.
