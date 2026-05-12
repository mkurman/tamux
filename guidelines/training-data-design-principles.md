---
name: training-data-design-principles
description: Meta-guideline for designing and curating training data across ML modalities — LLM pre/mid/post-training, RL alignment, CV, embeddings, NER, and audio TTS/STT. Covers universal principles plus modality-specific pipelines with their own dedicated guidelines.
recommended_guidelines:
  - llm-training-data-task
  - rl-alignment-data-task
  - cv-dataset-task
  - specialized-modality-data-task
  - specialized-training-data-task
  - rag-retrieval-data-task
  - synthetic-data-generation-task
  - multilingual-data-task
  - dataset-creation-curation-task
  - dataset-governance-task
  - cost-model-task
  - data-strategy-foundation-models-task
  - dataset-release-checklist
---

## Universal Principles (All Modalities)

These principles apply to every training dataset, regardless of modality.

### 1. Define the Data Specification First

Before collecting or filtering a single example, write down:
- **Task definition**: what is the model supposed to learn from this data?
- **Schema**: input/output format, required fields, allowed types.
- **Quality bar**: minimum acceptable quality, explicit rejection criteria.
- **Volume targets**: how many examples, with what diversity guarantees.
- **Coverage targets**: what concepts, domains, languages, or modalities must be represented.
- **Exclusion criteria**: what must NOT appear (PII, toxic content, benchmark contamination).

### 2. Provenance Is Not Optional

Every example must trace back to its origin. At minimum:
- Source URL or dataset identifier.
- Collection timestamp.
- Processing pipeline version.
- Filtering decisions applied.

Without provenance, you cannot debug data issues and cannot comply with emerging regulation (EU AI Act, etc.).

### 3. Dedup Aggressively, Then Validate

- **Exact dedup**: hash-based, always. Run first.
- **Near-dedup**: MinHash / SimHash for documents, embedding cosine for semantic similarity.
- **Cross-dataset dedup**: dedup against benchmarks, held-out sets, and other training corpora.
- **Validate**: spot-check dedup survivors for false positives.

### 4. Quality Filter, Don't Hoard

More data beats better data only at extreme scale. For most practical budgets:
- Filter aggressively: a 10x smaller clean dataset often beats a noisy one.
- Use automated quality signals: perplexity, classifier confidence, embedding coherence.
- Human-validate a stratified sample of the filtered set.
- Never filter on the test distribution — that's leakage.

### 5. Balance Diversity and Quality

- High quality + low diversity → overfitting, brittleness.
- High diversity + low quality → noise dominates signal.
- The sweet spot: quality filter first, then measure diversity, then fill gaps with targeted collection or synthetic generation.

### 6. Audit for Bias and Representativeness

- Measure demographic, linguistic, and domain representation.
- Over-sample underrepresented groups to a minimum threshold.
- Document known biases in the data card.
- Acknowledge that "representative" is domain-specific and contested.

### 7. Version Everything

- Every filtering pass produces a new dataset version.
- Semantic versioning: major = schema change, minor = new data, patch = bugfix.
- Never overwrite a released version.
- The version tag must appear in every downstream artifact (model card, paper, launch checklist).

### 8. Benchmark Contamination Is a First-Class Threat

- Maintain a contamination exclusion list (all benchmarks, evals, and held-out sets).
- Run n-gram overlap and embedding similarity against the exclusion list.
- Treat detected contamination as a blocking bug, not a footnote.

---

## Modality-Specific Guidelines

Each modality has its own dedicated guideline with pipeline details, quality metrics, and tool recommendations:

| Modality | Guideline | Key Focus |
|------|-----------|-------|
| **LLM Pre/Mid/Post-Training** | `llm-training-data-task` | Data mixing ratios, dedup at scale, perplexity filtering, instruction quality, benchmark contamination |
| **RL Alignment (RLHF/DPO/GRPO)** | `rl-alignment-data-task` | Preference pair quality, reward model training data, ranking consistency, chain-of-thought preference data |
| **Computer Vision** | `cv-dataset-task` | Augmentation strategy, annotation QC, class balance, resolution requirements, synthetic data |
| **Embeddings, NER, Audio TTS/STT** | `specialized-modality-data-task` | Triplet mining, BIO tagging quality, transcription accuracy, speaker diarization, alignment |

---

## Quality Gate (Universal)

A training dataset is ready when:
- The data specification document exists and is peer-reviewed.
- Provenance is recorded for every source.
- Exact and near-dedup are applied and validated.
- Quality filtering is applied with documented thresholds.
- Diversity, bias, and representation are measured and documented.
- Benchmark contamination scan is clean.
- The dataset is versioned, checksummed, and tagged.
