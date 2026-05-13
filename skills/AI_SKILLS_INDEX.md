# AI Skills Index

Curated index of local skills in `skills/` that are relevant to AI model training, architecture design, evaluation, RL, and adjacent MLOps workflows.

This index is organized by **workflow need**, not alphabetically, so an agent can choose the right skill quickly.

---

## 1. Resource Planning / Systems Constraints

Use these **before** expensive training or evaluation jobs.

| Skill | License | Best for | Path |
|---|---|---|---|
| `get-available-resources` | MIT | Inspect CPU/GPU/RAM/disk before training or large eval runs | `scientific-skills/get-available-resources/` |
| `optimize-for-gpu` | not specified in frontmatter | Speeding up Python/ML/data pipelines on NVIDIA GPUs | `scientific-skills/optimize-for-gpu/` |

### Recommended choice
- Start with `get-available-resources` when the workload size or machine fit is unclear.
- Use `optimize-for-gpu` when the bottleneck is clearly compute-heavy and GPU acceleration is feasible.

---

## 2. Experiment Configuration / Reproducibility / Sweeps

| Skill | License | Best for | Path |
|---|---|---|---|
| `hydra` | MIT | hierarchical experiment config, CLI overrides, multirun sweeps, reproducible outputs | `scientific-skills/hydra/` |
| `hydra-zen` | MIT | Python-first structured config generation, less Hydra boilerplate | `scientific-skills/hydra-zen/` |
| `optuna` | MIT | hyperparameter optimization, pruning, multi-objective search | `scientific-skills/optuna/` |

### Recommended choice
- Use `hydra` for config composition and reproducible run management.
- Use `hydra-zen` when the project is Python-heavy and YAML should be minimized.
- Use `optuna` for actual search/tuning once the base training pipeline is stable.

---

## 3. General Deep Learning Training

| Skill | License | Best for | Path |
|---|---|---|---|
| `pytorch-lightning` | Apache-2.0 | scalable training loops, callbacks, logging, multi-GPU strategies | `scientific-skills/pytorch-lightning/` |
| `transformers` | Apache-2.0 | pretrained/fine-tuned transformer models for NLP/CV/audio/multimodal | `scientific-skills/transformers/` |
| `nanogpt` | MIT | minimal GPT pretraining / finetuning reference implementation | `scientific-skills/nanogpt/` |

### Recommended choice
- Use `pytorch-lightning` for production-ish or scalable PyTorch training loops.
- Use `transformers` when the task is model-hub / pretrained-model centric.
- Use `nanogpt` when the need is **understanding or hacking raw GPT pretraining** with minimal abstraction.

---

## 4. LLM Evaluation / Benchmarking / Regression Testing

| Skill | License | Best for | Path |
|---|---|---|---|
| `lm-evaluation-harness` | MIT | standard benchmark suites, leaderboard-style eval, few-shot benchmarking | `scientific-skills/lm-evaluation-harness/` |
| `lighteval` | MIT | multi-backend eval, multilingual benchmarks, sample-level analysis | `scientific-skills/lighteval/` |
| `openai-evals` | MIT | custom eval registries, model-graded evals, prompt/system regression testing | `scientific-skills/openai-evals/` |

### Recommended choice
- Use `lm-evaluation-harness` for classic academic benchmark runs and open leaderboard-style comparison.
- Use `lighteval` when backend flexibility or multilingual coverage matters.
- Use `openai-evals` when evaluating **product behavior**, prompts, or model-graded quality on custom datasets.

---

## 5. Reinforcement Learning Algorithms

| Skill | License | Best for | Path |
|---|---|---|---|
| `stable-baselines3` | MIT | standard RL baselines with strong docs and familiar API | `scientific-skills/stable-baselines3/` |
| `cleanrl` | MIT | readable single-file RL implementations for modification/research | `scientific-skills/cleanrl/` |
| `pufferlib` | MIT | high-throughput, parallel, multi-agent RL at scale | `scientific-skills/pufferlib/` |

### Recommended choice
- Use `stable-baselines3` for quick baseline experiments and standard control tasks.
- Use `cleanrl` when you need to inspect or modify the algorithm internals directly.
- Use `pufferlib` when throughput and scale matter more than API simplicity.

---

## 6. Reinforcement Learning Environments

### 6.1 Single-agent / control / classic benchmarks

| Skill | License | Best for | Path |
|---|---|---|---|
| `gymnasium` | MIT | canonical single-agent RL environment API and wrappers | `scientific-skills/gymnasium/` |

### 6.2 Multi-agent RL environments

| Skill | License | Best for | Path |
|---|---|---|---|
| `pettingzoo` | MIT | multi-agent RL environments, AEC + parallel APIs | `scientific-skills/pettingzoo/` |

### 6.3 Robotics / multi-task / meta-RL benchmarks

| Skill | License | Best for | Path |
|---|---|---|---|
| `metaworld` | MIT | robotic manipulation, multi-task RL, meta-RL adaptation benchmarks | `scientific-skills/metaworld/` |

### 6.4 Text-native / LLM RL environments

| Skill | License | Best for | Path |
|---|---|---|---|
| `textarena` | MIT | self-play, strategic reasoning, text-game RL for LLMs | `scientific-skills/textarena/` |

### Recommended choice
- Use `gymnasium` for standard single-agent pipelines.
- Use `pettingzoo` for multi-agent games and MARL.
- Use `metaworld` for robotics-style manipulation benchmarks.
- Use `textarena` for text-native self-play and LLM reasoning environments.

---

## 7. Vision Data Augmentation

| Skill | License | Best for | Path |
|---|---|---|---|
| `albumentations` | MIT | image augmentation across classification, segmentation, detection, keypoints | `scientific-skills/albumentations/` |

### Note
`albumentations` is MIT but the classic repo is no longer actively maintained. Still useful; just treat it as stable/legacy rather than actively evolving.

---

## 8. Common Workflow Compositions

### A. LLM pretraining / minimal GPT experimentation
1. `get-available-resources`
2. `hydra` or `hydra-zen`
3. `nanogpt`
4. `optuna` (only after baseline is stable)
5. `lm-evaluation-harness` or `lighteval`

### B. Transformer finetuning pipeline
1. `get-available-resources`
2. `transformers`
3. `pytorch-lightning` (if custom loop orchestration is needed)
4. `hydra`
5. `optuna`
6. `lm-evaluation-harness` / `openai-evals`

### C. Standard single-agent RL experiment
1. `gymnasium`
2. `stable-baselines3` or `cleanrl`
3. `hydra`
4. `optuna`

### D. Multi-agent RL or self-play
1. `pettingzoo` or `textarena`
2. `cleanrl` or `pufferlib`
3. `hydra`
4. `optuna`

### E. Robotics / transfer / meta-RL
1. `metaworld`
2. `cleanrl` / custom PyTorch loop
3. `hydra`
4. `optuna`
5. `lighteval` / custom analysis as needed

### F. Vision training pipeline
1. `albumentations`
2. `pytorch-lightning` or `transformers`
3. `hydra`
4. `optuna`

---

## 9. MIT Skills Added in This Curation Pass

### Tier 1
- `optuna`
- `lm-evaluation-harness`
- `gymnasium`
- `pettingzoo`
- `cleanrl`
- `hydra`
- `albumentations`

### Tier 2
- `nanogpt`
- `openai-evals`
- `lighteval`
- `metaworld`
- `textarena`
- `hydra-zen`

---

## 10. Important Gaps Still Present Under Strict MIT-Only Policy

These domains are still not fully covered by MIT-only additions because many best-in-class repos are Apache-2.0 or other licenses:

### RLHF / post-training / alignment
Strong non-MIT candidates:
- TRL
- OpenRLHF
- veRL
- alignment-handbook

### LLM serving / inference
Strong non-MIT candidates:
- vLLM
- SGLang
- TGI

### Distributed training systems
Strong non-MIT candidates:
- DeepSpeed
- Ray
- LitGPT

### Data/versioning/MLOps infra
Strong non-MIT candidates:
- DVC
- MLflow ecosystem pieces
- ZenML

If permissive-but-not-MIT licenses are acceptable, the practical skill set can be materially improved by adding these.

---

## 11. Skill Selection Heuristics for Agents

- If the user says **benchmark / leaderboard / MMLU / GSM8K**, start with `lm-evaluation-harness` or `lighteval`.
- If the user says **prompt regressions / model-graded QA / private eval dataset**, start with `openai-evals`.
- If the user says **hyperparameters / pruning / search / study**, start with `optuna`.
- If the user says **configs / sweeps / reproducibility / experiment folders**, start with `hydra`.
- If the user says **Python-first Hydra / no YAML / dataclass configs**, start with `hydra-zen`.
- If the user says **train GPT from scratch / understand GPT internals**, start with `nanogpt`.
- If the user says **single-agent RL**, start with `gymnasium` + `stable-baselines3`.
- If the user says **modify RL algorithm internals**, prefer `cleanrl`.
- If the user says **multi-agent RL**, prefer `pettingzoo`.
- If the user says **robotic manipulation / meta-RL**, prefer `metaworld`.
- If the user says **text self-play / LLM game environments**, prefer `textarena`.
- If the user says **image augmentation / bboxes / masks / keypoints**, prefer `albumentations`.

---

## 12. Maintenance Notes

- New skills added in this pass were written with **agentskills.io-compatible YAML frontmatter** and explicit semantic `tags: [...]` intended to improve skill discovery.
- Several older pre-existing skills in the repo still use broader or noisier tags; they may benefit from a later normalization pass.


## Data Lattice — Dataset Curation Framework

Complete reference for the Data Lattice dataset curation guidelines and skills. Every guideline contains working Python code, measurable quality gates, and cross-references.

### Meta / Entry Points

| Guideline | Best for | Path |
|-----------|----------|------|
| `training-data-design-principles` | 8 universal principles — always start here | `guidelines/training-data-design-principles.md` |
| `dataset-curation-manifest` | Full index of all guidelines and skills | `guidelines/dataset-curation-manifest.md` |
| `dataset-release-checklist` | Aggregated launch checklist with sign-offs | `guidelines/dataset-release-checklist.md` |

### Core Curation Skills (11)

| Skill | Best for | Path |
|-------|----------|------|
| `dataset-cleaning` | Missing values, dedup, outlier handling, normalization | `skills/data-lattice/dataset-cleaning/` |
| `dataset-splitting` | Train/val/test, stratified, group, time-series splits | `skills/data-lattice/dataset-splitting/` |
| `dataset-versioning` | DVC, manifest.json, semantic versioning, checksums | `skills/data-lattice/dataset-versioning/` |
| `hf-datasets` | HuggingFace streaming, map/filter, push_to_hub | `skills/data-lattice/hf-datasets/` |
| `embedding-analysis` | Semantic dedup (SemDedup, LSHBloom), GRAPE perplexity, DataRater, JS/Wasserstein | `skills/data-lattice/embedding-analysis/` |
| `llm-assisted-curation` | vLLM/SGLang judge, batch rewrite, synthetic generation | `skills/data-lattice/llm-assisted-curation/` |
| `data-card-writer` | Structured datasheets (Gebru et al. format) | `skills/data-lattice/data-card-writer/` |
| `label-quality-audit` | Confident learning noise detection, per-class error rates | `skills/data-lattice/label-quality-audit/` |
| `bias-audit` | Demographic parity, representation gaps, outcome disparity | `skills/data-lattice/bias-audit/` |
| `benchmark-contamination-scan` | N-gram + embedding overlap scan against 60+ benchmarks | `skills/data-lattice/benchmark-contamination-scan/` |
| `data-diff` | Structured diff between dataset versions | `skills/data-lattice/data-diff/` |

### ML Training Data Guidelines

| Guideline | Covers |
|-----------|--------|
| `llm-training-data-task` | Pre-training mixing ratios, CPT, SFT instruction quality |
| `rl-alignment-data-task` | RLHF/DPO/GRPO preference pairs, reward model data |
| `cv-dataset-task` | Image QA, annotation QC, augmentation strategy |
| `specialized-modality-data-task` | Embeddings, NER, audio TTS/STT |
| `specialized-training-data-task` | Contrastive, KD, continual, federated, anomaly |
| `rag-retrieval-data-task` | Query-document pairs, hard negatives, chunk strategies |
| `synthetic-data-generation-task` | LLM/diffusion/SDV, realism checks, hallucination audit |
| `agentic-training-data-task` | Trajectory QC, reward extraction, environment diversity |
| `time-series-data-task` | Stationarity, seasonality, walk-forward validation |
| `graph-data-task` | Node/edge dedup, degree distribution, edge-level splits |
| `curriculum-learning-data-task` | Difficulty scoring, anti-curriculum, prerequisites |
| `multitask-transfer-data-task` | Task interference, negative transfer, fine-tuning stability |
| `fewshot-retrieval-data-task` | Few-shot episodes, prompt sensitivity, retrieval quality |
| `meta-learning-data-task` | Task distribution, support/query splits, negative results |
| `architecture-specific-data-task` | Transformer, CNN, sparse attention, memory networks |

### Medical / Biological Guidelines

| Guideline | Covers |
|-----------|--------|
| `medical-bio-data-task` | HIPAA/GDPR/IRB, batch effects, reference genomes |
| `genomics-sequencing-data-task` | FASTQ QC, alignment, VCF variant QC |
| `single-cell-data-task` | Adaptive QC, ambient RNA, doublet detection, integration |
| `immunology-data-task` | TCR/BCR clonotypes, flow cytometry, cytokines |
| `clinical-drug-discovery-data-task` | Compound standardization, HTS, ADMET, clinical trials |
| `proteomics-metabolomics-data-task` | 3-level FDR, PTM localization, metabolomics QC |
| `epigenomics-data-task` | ChIP-seq, ATAC-seq, methylation, Hi-C |
| `pathology-data-task` | WSI integrity, stain normalization, annotation QC |
| `clinical-longitudinal-data-task` | Lab harmonization, survival censoring, EHR phenotypes |

### Evaluation & Validation

| Guideline | Covers |
|-----------|--------|
| `data-contamination-task` | 9 types: benchmark, temporal, group, label, cross-dataset, synthetic, canary, model-based, multimodal |
| `evaluation-dataset-design-task` | 4-level pyramid, per-class metrics, calibration, MDE |
| `cross-validation-strategy-task` | 8-strategy matrix, compatibility checker, nested CV |
| `robustness-engineering-task` | 3-tier stress tests, envelope mapping, failure genealogy |
| `intersectional-evaluation-task` | Multi-axis fairness, compound effects, mitigation audit |
| `reproducibility-science-task` | Exact reproduction capture, environment influence |

### Operations & Governance

| Guideline | Covers |
|-----------|--------|
| `annotation-management-task` | Team structure, IAA targets, active learning |
| `annotation-economics-task` | Fatigue modeling, task matching, cost-quality curves |
| `data-pipeline-monitoring-task` | Schema drift, distribution drift, backfill protocols |
| `multilingual-data-task` | Language coverage, tokenizer fertility, translation quality |
| `cultural-linguistic-data-task` | Cultural context, language-specific bias, concept alignment |
| `data-visualization-task` | 6-stage visualization protocol |
| `dataset-governance-task` | Licensing, GDPR, EU AI Act, DPAs |
| `data-lifecycle-governance-task` | Birth→Adolescence→Adulthood→Retirement→Death |
| `cost-model-task` | Build vs buy vs license, ROI framework |
| `data-strategy-foundation-models-task` | $100M data plan, mixing ratios, scaling laws |
| `privacy-preserving-data-task` | DP-SGD, k-anonymity, membership inference |
| `production-deployment-data-task` | A/B tests, canary validation, rollback triggers |
| `team-operations-data-task` | Skill matching, handoff gates, review efficiency |

### Advanced / Frontier

| Guideline | Covers |
|-----------|--------|
| `data-attribution-task` | TRAK, influence functions, datamodels |
| `data-mixture-optimization-task` | DoReMi, DoGE, auto-curricula |
| `data-feedback-loop-task` | Self-training drift, confidence decay, optimal stopping |
| `mechanistic-interpretability-data-task` | SAE data, circuit discovery, activation patching |
| `sim-to-real-bridge-task` | Multi-axis gap, domain randomization, failure detection |
| `data-model-coevolution-task` | Capability inheritance, cross-gen contamination |
| `data-archaeology-task` | Schema reconstruction, corruption recovery, legacy bias |
| `adversarial-data-design-task` | Targeted failures, universal patterns, red-teaming |
| `model-editing-energy-data-task` | Behavior editing, carbon estimation, energy efficiency |
| `embodied-compositional-data-task` | Robot trajectories, physics, recursive composition |
| `data-portfolio-theory-task` | Marginal value, diversification, data asset valuation |
| `scaling-law-data-task` | Calibration, compute-optimal selection, anomaly detection |
| `knowledge-engineering-data-task` | Expert knowledge capture, conflict resolution |
| `data-compression-learning-task` | Redundancy taxonomy, lossy tolerance, decompression |
| `data-economics-task` | Marginal value curves, portfolio diversification, market valuation |
| `industry-verticals-data-task` | Security, finance, supply chain, energy, insurance |
| `data-lattice-ecosystem-task` | Certification, academic integration, vertical packs, training |
| `dataset-certification-task` | Bronze/Silver/Gold/Platinum automated certification |

### Source Catalogs

| Catalog | Covers |
|---------|--------|
| `medical-dataset-sources-task` | 70+ datasets: EHR, imaging, genomics, single-cell, drug |
| `protein-dataset-sources-task` | PDB, AlphaFold DB, ESM Atlas, STRING, PDBbind |
| `chemistry-materials-sources-task` | COD, Materials Project, QM9, OC20 |
| `neuroscience-sources-task` | Neuropixels, Allen, HCP, MICrONS, FlyWire |
| `satellite-geospatial-sources-task` | Sentinel, Landsat, SpaceNet, BigEarthNet |
