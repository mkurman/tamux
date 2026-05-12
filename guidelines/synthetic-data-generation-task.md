---
name: synthetic-data-generation-task
description: Generate synthetic training data — LLM generation, diffusion models for images, SDV for tabular, augmentation pipelines. Covers quality gates, hallucination detection, diversity measurement, and synthetic flagging requirements.
recommended_skills:
  - sdv
  - llm-assisted-curation
  - hf-datasets
  - embedding-analysis
  - dataset-versioning
recommended_guidelines:
  - training-data-design-principles
  - dataset-creation-curation-task
  - llm-training-data-task
---

## Overview

Synthetic data fills gaps that real data can't: rare classes, privacy-sensitive domains, edge cases, and cost-prohibitive collection. But synthetic data also introduces artifacts, hallucinations, and distribution collapse. The rule: generate judiciously, validate ruthlessly, and always flag as synthetic.

## Generation Methods by Modality

| Modality | Method | Tool | Best For |
|------|-------|-------|-------|
| **Tabular** | GAN / Copula | `sdv` skill | Class imbalance, privacy |
| **Text** | LLM generation | `llm-assisted-curation` | Few-shot expansion, instruction diversity |
| **Text** | Back-translation | Marian / NLLB | Paraphrase augmentation |
| **Images** | Diffusion (SD, Flux) | `diffusers` skill | Rare objects, privacy-sensitive |
| **Images** | Augmentation chain | `albumentations` | Robustness, invariance |
| **Speech** | TTS | `whisper` + TTS models | Accent diversity, low-resource languages |
| **3D / Molecules** | Conformer generation | `rdkit`, `diffdock` | Drug discovery, materials |

## Quality Gates (Universal)

### 1. Realism Check

- Train a classifier on real vs. synthetic. If accuracy > 70%, your synthetic data is distinguishable — and may teach the wrong signal.
- Human evaluation on a stratified sample: "Is this real or synthetic?" Target < 60% human accuracy (i.e., near-chance).

### 2. Diversity Check

```python
def synthetic_diversity(real_emb, synthetic_emb):
    """Compare embedding diversity."""
    real_diversity = np.mean(1 - cosine_similarity(real_emb))  # avg pairwise dissimilarity
    synth_diversity = np.mean(1 - cosine_similarity(synthetic_emb))
    # synth_diversity < real_diversity * 0.7 → mode collapse
    return {
        "real_diversity": real_diversity,
        "synthetic_diversity": synth_diversity,
        "mode_collapse": synth_diversity < real_diversity * 0.7,
    }
```

### 3. Hallucination Audit (Text)

- Factuality check: Does the synthetic text contain verifiable false claims?
- For instruction data: Can the response actually be derived from the prompt?
- Run a strong LLM judge on a sample. Flag factuality issues.

### 4. Contamination Check

- Synthetic data must NEVER be generated from or about benchmark content.
- Scan synthetic outputs against the benchmark exclusion list.

## Synthetic Flagging

**Mandatory**: Every synthetic example must carry `synthetic: true` in metadata.

```python
example = {
    "text": generated_text,
    "synthetic": True,
    "generator_model": "gpt-4.1",
    "generation_temperature": 0.8,
    "seed_example_id": "real_0042",  # if derived from a real example
}
```

Without this flag, downstream debugging is impossible — you can't tell if a failure is from bad synthetic data or real data drift.

## Mixing Ratios

| Scenario | Max Synthetic % | Rationale |
|------|-------|-------|
| Pre-training | 5-10% | Synthetic text degrades with scale |
| Instruction tuning | 10-30% | Higher quality tolerance if validated |
| Class balancing | Up to 50% of minority | Better than severe imbalance |
| Privacy replacement | 100% | Synthetic-only for privacy compliance |

## Quality Gate

- Real vs. synthetic classifier accuracy < 70%.
- Diversity not collapsed (synthetic_diversity ≥ real_diversity * 0.7).
- Hallucination rate < 5% on factuality audit.
- All synthetic examples flagged with `synthetic: true`.
- Generation metadata recorded (model, temperature, seed example).
- Synthetic data versioned separately from real data.
