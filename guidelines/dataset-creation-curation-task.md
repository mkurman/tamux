---
name: dataset-creation-curation-task
description: Use for creating, cleaning, curating, versioning, splitting, or documenting datasets — whether from raw sources, synthetic generation, or existing collections. Covers the full modern stack: HuggingFace datasets, embedding-based dedup, LLM-assisted quality filtering, and 2025-2026 literature techniques.
recommended_skills:
  - dataset-cleaning
  - dataset-splitting
  - dataset-versioning
  - hf-datasets
  - embedding-analysis
  - llm-assisted-curation
  - exploratory-data-analysis
  - sdv
recommended_guidelines:
  - training-data-design-principles
  - annotation-management-task
  - data-pipeline-monitoring-task
  - dataset-governance-task
  - dataset-release-checklist
  - data-analysis-task
  - scientific-data-analysis-task
  - evidence-quality-task
---

## Overview

Dataset creation and curation is a first-class engineering discipline. It requires provenance, reproducibility, quality gates, and documentation — not ad-hoc scripting. This guideline covers the full lifecycle from sourcing through cleaning, embedding-based deduplication, LLM-assisted quality filtering, splitting, versioning, and documentation.

## Workflow

1. **Define the dataset specification** before touching data:
   - Purpose, intended use, and explicit non-use cases.
   - Target schema: columns, types, constraints, allowed values.
   - Acceptable missingness thresholds and quality criteria.
   - Licensing, consent, and regulatory constraints (GDPR, HIPAA, IRB).

2. **Source and load** the raw data:
   - Use `hf-datasets` to load from the HuggingFace Hub, local Parquet/JSONL/Arrow files, or stream datasets larger than RAM.
   - Prefer authoritative, versioned sources over scraped or manual exports.
   - Record provenance: origin URL, query, timestamp, credentials used.
   - For synthetic tabular data, apply `sdv` for statistical generation.

3. **Initial inspection** with `exploratory-data-analysis`:
   - Shape, dtypes, cardinality, missingness, duplicates.
   - Distribution skew, class imbalance, outlier detection.
   - Flag anything that violates the specification.

4. **Clean and normalize** with `dataset-cleaning`:
   - Handle missing values with an explicit, documented strategy (not silent `dropna()`).
   - Deduplicate with clear identity rules.
   - Normalize formats, encodings, units, and categorical values.
   - Remove or cap outliers per domain knowledge, not arbitrary percentiles.
   - Document every transformation — what was changed, why, and how many rows affected.

5. **Advanced deduplication** with `embedding-analysis`:
   - Apply semantic deduplication (NeMo Curator SemDedup, LSHBloom) to remove meaning-equivalent near-duplicates that exact matching misses.
   - Compute embedding-based quality scores (DataRater-style neighborhood coherence).
   - For very large datasets (>100M), use connected-components with approximate nearest neighbors.
   - Measure dataset diversity and redundancy in embedding space before and after dedup.

6. **LLM-assisted quality filtering** with `llm-assisted-curation`:
   - Host a quality-scoring model with `vllm` or `sglang`.
   - Apply LLM-as-judge scoring (clarity, correctness, usefulness) per example.
   - Use perplexity-based filtering (GRAPE score, 2025) to remove gibberish and noise.
   - Optionally: extract structured labels, classify difficulty for curriculum learning, or rewrite noisy text.
   - Flag synthetic examples with a `synthetic: true` field.

7. **Split** with `dataset-splitting`:
   - Train / validation / test split before any modeling.
   - Stratify on target and any protected attributes.
   - For time-series: chronological split, no future leakage.
   - For grouped data: group-level split to avoid cross-contamination.
   - Never use test set for any decision, including imputation or feature selection.

8. **Validate** the curated dataset:
   - Schema conformance: every column matches specification.
   - Embedding distribution comparison: use `embedding-analysis` to measure JS divergence and Wasserstein distance between splits.
   - Integrity checks: no duplicate rows across splits, no nulls where forbidden.
   - Label quality audit if labels are human- or model-generated.

9. **Version** with `dataset-versioning`:
   - Write a `manifest.json` with file checksums, provenance, and transformation log.
   - Tag releases with semantic versioning (`v1.0.0`). Never overwrite a released version.
   - Push to DVC remote or HuggingFace Hub for sharing.

10. **Document** the dataset:
    - Data card or datasheet: purpose, composition, collection process, preprocessing, limitations.
    - Include a usage license and citation instructions.
    - Reference `data-analysis-task` guidance for structuring documentation.

## Quality Gate

A dataset is ready when:
- The specification is written and every column is accounted for.
- Every transformation is documented with rationale and row counts.
- Semantic dedup is applied and results are reviewed.
- LLM quality scores are saved alongside the data for auditability.
- Splits are reproducible (fixed seed, deterministic logic).
- The dataset is versioned, checksummed, and tagged.
- A data card exists with intended use, limitations, and license.
- Validation passes: schema, embedding distributions, integrity, and label quality checks all pass.

## 2025-2026 Literature Foundation

This guideline integrates techniques from:

| Paper | Venue | Key Insight |
||--------|--------|-------|
| **DataRater** (Calian et al.) | NeurIPS 2025 | Meta-learned quality scoring from embedding neighborhoods |
| **Why Less is More** (Dohmatob et al.) | 2025 | Theoretical justification for aggressive filtering thresholds |
| **GRAPE Score** | 2025 | Student-model perplexity as a scalable quality signal |
| **NeMo Curator SemDedup** (NVIDIA) | 2024-2025 | Clustering-based semantic deduplication at scale |
| **LSHBloom** (Khan et al.) | 2025 | Internet-scale text dedup via LSH + union-find |
| **Blu-WERP** (Rupesh et al.) | 2025 | Scalable streaming pipeline for LLM dataset preprocessing |
| **TBDFiltering** (Busa-Fekete et al.) | 2025 | Sample-efficient tree-based data filtering |
| **Ensembled Multimodal Curation** (Xu et al.) | 2025 | Multi-signal quality fusion across modalities |
