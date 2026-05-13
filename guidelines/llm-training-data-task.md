---
name: llm-training-data-task
description: Curate datasets for LLM pre-training, continued pre-training (CPT), and post-training (SFT/instruction tuning). Covers data mixing ratios, deduplication at scale, perplexity filtering, instruction quality scoring, and benchmark contamination prevention.
recommended_skills:
  - hf-datasets
  - embedding-analysis
  - llm-assisted-curation
  - dataset-cleaning
  - dataset-versioning
recommended_guidelines:
  - dataset-creation-curation-task
  - training-data-design-principles
  - evidence-quality-task
---

## Overview

LLM training data curation is the highest-leverage activity in foundation model development. The difference between a good and great model often traces back to data decisions made months earlier. This guideline covers all three phases: pre-training, continued pre-training (CPT / mid-training), and post-training (SFT / instruction tuning).

## Phase 1: Pre-Training Data

Pre-training data is the foundation. It teaches the model language, world knowledge, reasoning patterns, and code. Quality here cascades through everything downstream.

### 1a. Data Mixing Strategy

Define explicit mixing ratios by domain, not by source:

| Domain | Suggested Range | Rationale |
|-------|---------|-------|
| Web text (filtered) | 40-60% | Broad knowledge, linguistic diversity |
| Code | 10-25% | Reasoning, structured thinking |
| Books / long-form | 5-15% | Coherence, narrative structure |
| Academic / scientific | 5-15% | Factual knowledge, formal reasoning |
| Conversational / dialogue | 3-8% | Instruction-following readiness |
| Multilingual | 5-15% | Cross-lingual transfer |

**Mixing rules:**
- Upsample high-quality domains (books, academic) relative to raw availability.
- Never mix by source count — mix by token count after tokenization.
- Validate mix ratios post-filtering, not pre-filtering.
- Re-weight periodically as new data arrives.

### 1b. Deduplication (Pre-Training Scale)

```
Pipeline order:
1. Exact dedup (document-level hash)          → MUST run first
2. Near-dedup (MinHash, 0.8 Jaccard)          → removes boilerplate variants
3. Cross-dataset dedup (vs benchmarks/tests)   → contamination prevention
4. Semantic dedup (embedding + clustering)      → last resort, expensive
```

Use `hf-datasets` streaming + `embedding-analysis` semantic dedup for datasets that don't fit in memory.

### 1c. Quality Filtering

Apply in this order (cheapest first):

| Filter | Method | Threshold |
|-------|-------|-------|
| Length | Character/word count | Min 100 chars, max context window |
| Language | fastText / langdetect | Confidence > 0.9 |
| Perplexity | GRAPE score via small LM | Keep bottom 80% (lowest ppl) |
| Heuristic | Mean word length, symbol ratio, bullet density | Domain-specific |
| Classifier | Lightweight quality classifier trained on human labels | Probability > 0.5 |

Use `llm-assisted-curation` for LLM-as-judge scoring on a sampled subset to validate heuristic filters.

### 1d. Personal & Toxic Content

- **PII**: Run regex + NER-based PII detection. Redact or remove.
- **Toxic content**: Classifier-based filtering. Tune threshold carefully — over-filtering removes legitimate content (medical text, news).
- **Copyright**: Maintain an opt-out / robots.txt exclusion list for web-sourced data.

### 1e. Contamination Prevention

```
Contamination exclusion list:
- All benchmark datasets (MMLU, HellaSwag, GSM8K, HumanEval, etc.)
- All evaluation sets you plan to use
- All canary strings from known datasets
- All held-out validation/test splits

Scan method:
1. 13-gram overlap (fast, high recall)
2. Embedding similarity for semantic contamination (slow, high precision)
3. Flag and remove ALL matches — no "close enough" threshold
```

## Phase 2: Continued Pre-Training (CPT / Mid-Training)

CPT adapts a base model to a specific domain (medicine, law, finance, code) or capability (long-context, multilingual).

### 2a. Domain Data Selection

- **Relevance scoring**: Use `embedding-analysis` to compute cosine similarity between candidate documents and a curated domain reference set.
- **Deduplicate against base pre-training data**: Don't train twice on the same content.
- **Quality over volume**: For CPT, 1B high-quality domain tokens often outperform 10B noisy ones.

### 2b. Mixing with Base Data

Always mix domain data with a replay buffer of general pre-training data (10-30%) to prevent catastrophic forgetting.

### 2c. Long-Context CPT

- Source documents with natural long-range dependencies: books, legal contracts, research papers.
- Avoid concatenating short documents — the model learns to ignore document boundaries.
- Validate that the average document length matches the target context window.

## Phase 3: Post-Training (SFT / Instruction Tuning)

Instruction tuning teaches the model to follow directions. Quality matters far more than quantity here — 1,000 excellent examples can outperform 100,000 mediocre ones.

### 3a. Instruction Quality Criteria

Every instruction example must satisfy:

| Criterion | Check |
|------|-------|
| **Clear task** | The instruction unambiguously describes one task |
| **Correct answer** | The response is factually correct and complete |
| **Helpful format** | The response follows the requested format |
| **Diverse** | The example is not a paraphrase of another |
| **Challenging** | The task requires reasoning, not just retrieval |

Use `llm-assisted-curation` to score examples on these criteria at scale.

### 3b. Instruction Diversity

Cover these axes intentionally:
- **Task types**: generation, classification, extraction, summarization, reasoning, coding, creative.
- **Difficulty**: easy → expert. Apply curriculum scoring with `llm-assisted-curation`.
- **Format**: chat, single-turn, multi-turn, structured output.
- **Domain**: general, code, math, science, creative writing, analysis.
- **Language**: target languages with proportional representation.

### 3c. Instruction Data Sources

| Source | Quality | Diversity | Cost |
|-------|-------|-------|-------|
| Human-written | Highest | Low per $ | High |
| LLM-generated (single model) | Medium | Low (model bias) | Low |
| LLM-generated (multi-model) | Medium-High | Medium | Medium |
| Template + human seed | High | Configurable | Medium |
| Evolved (self-instruct, evol-instruct) | Medium | High | Low |
| Curated from chat logs | High | High (organic) | Medium (privacy) |

### 3d. Response Quality Validation

For LLM-generated responses in instruction data:
- Run a second, stronger model as judge to verify correctness.
- Flag and remove examples where the judge disagrees.
- Human-review a stratified sample of judge-approved examples.

## Quality Gate

LLM training data is ready when:
- Pre-training mix ratios are defined and validated post-filtering.
- All three dedup passes (exact, near, cross-dataset) are complete.
- Quality filters are applied with documented thresholds.
- Contamination scan against all benchmarks is clean.
- CPT data is deduped against base pre-training data.
- Instruction data passes quality scoring (≥ 3/5 on all criteria for ≥ 90% of examples).
- All phases are versioned, checksummed, and provenance-tracked.
