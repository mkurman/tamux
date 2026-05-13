---
name: dataset-curation-manifest
description: Complete manifest of the zorai Dataset Curation Framework — 32 guidelines, 10 skills, 5 source catalogs. Organized by workflow need. Start here.
recommended_guidelines:
  - training-data-design-principles
  - dataset-release-checklist
---

# Dataset Curation Framework — Complete Manifest

This is the complete index of the zorai dataset curation framework. Every guideline and skill listed below exists as a file under `~/.zorai/guidelines/` or `~/.zorai/skills/`.

---

## Entry Points: Start Here

| # | Guideline | What It Covers | When to Read |
|---|-----------|----------------|--------------|
| 1 | **`training-data-design-principles`** | 8 universal principles (provenance, dedup, quality, diversity, bias, versioning, contamination) | First. Always first. |
| 2 | **`dataset-creation-curation-task`** | 10-step general-purpose pipeline: spec → source → clean → dedup → split → validate → version → document | Every new dataset |
| 3 | **`dataset-release-checklist`** | Aggregated launch checklist combining all quality gates | Before shipping any dataset |

---

## By Modality / Task

### LLM and Language
| Guideline | Covers |
|-----------|--------|
| `llm-training-data-task` | Pre-training mixing ratios, CPT domain adaptation, SFT instruction quality scoring, contamination prevention |
| `rag-retrieval-data-task` | Query-document pairs, hard negative mining, chunk strategies, BEIR/MTEB benchmarks |
| `specialized-modality-data-task` | Embeddings (triplet mining), NER (BIO tagging), audio TTS/STT (alignment, diarization) |
| `llm-assisted-curation` (skill) | vLLM/SGLang for LLM-as-judge scoring, batch rewriting, synthetic generation, curriculum scoring |
| `agentic-training-data-task` | Trajectory QC (stuck loops, tool hallucination), reward signal extraction, environment diversity |

### Computer Vision
| Guideline | Covers |
|-----------|--------|
| `cv-dataset-task` | Image technical QA (pHash, EXIF, color space), annotation QC, augmentation-by-task table, multimodal pairs |

### RL / Alignment
| Guideline | Covers |
|-----------|--------|
| `rl-alignment-data-task` | Preference pair construction (6 axes), DPO/GRPO data requirements, CoT preferences, reward model training data |

### Specialized Training Paradigms
| Guideline | Covers |
|-----------|--------|
| `specialized-training-data-task` | Contrastive learning, knowledge distillation, continual learning, federated learning, anomaly detection |
| `model-compression-data-task` | Pruning calibration, distillation teacher data (SlimQwen 2026, DistilQwen2.5 2025), quantization calibration, MoE expert preservation |
| `continual-pretraining-data-task` | CPT domain mixing, replay buffer design, forgetting detection, progressive domain adaptation |
| `model-merging-data-task` | SLERP/TIES/DARES/soup merging, merge candidate selection, post-merge validation, conflict detection |
| `efficient-inference-data-task` | Speculative decoding calibration, KV cache optimization, dynamic batching, inference-aware training data |
| `synthetic-data-generation-task` | LLM/diffusion/SDV generation, realism checks, hallucination audit, synthetic flagging |
| `time-series-data-task` | Stationarity tests, seasonality detection, walk-forward validation, lag leakage |
| `graph-data-task` | Node/edge dedup, degree distribution QC, edge-level splitting, negative sampling |

### Medical / Biological
| Guideline | Covers |
|-----------|--------|
| `medical-bio-data-task` | Meta: HIPAA/GDPR/IRB, batch effects, reference genome versioning, clinical metadata standards |
| `genomics-sequencing-data-task` | FASTQ QC, alignment metrics (BWA-MEM), VCF variant QC, coverage analysis |
| `single-cell-data-task` | Adaptive QC (MAD-based), ambient RNA, doublet detection, scVI/Harmony integration |
| `immunology-data-task` | TCR/BCR clonotypes, AIRR compliance, flow cytometry FCS validation, cytokine multiplex |
| `clinical-drug-discovery-data-task` | Compound standardization (RDKit), HTS Z' factor, ADMET, clinical trial integrity |
| `proteomics-metabolomics-data-task` | 3-level FDR (PSM/peptide/protein), PTM localization, metabolomics QC |
| `epigenomics-data-task` | ChIP-seq FRiP/IDR, ATAC-seq TSS enrichment, bisulfite conversion, Hi-C contact matrices |
| `pathology-data-task` | WSI integrity, stain normalization, annotation QC, multi-site harmonization |
| `clinical-longitudinal-data-task` | Lab unit harmonization, temporal leakage, survival censoring audit, EHR phenotype validation |

### Evaluation and Validation
| Guideline | Covers |
|-----------|--------|
| `data-contamination-task` | 9 contamination types: benchmark, temporal, group, label, cross-dataset,, canary, model-based, multimodal |
| `evaluation-dataset-design-task` | 4-level evaluation pyramid, per-class metrics, calibration audit, minimum detectable effect |
| `cross-validation-strategy-task` | 8-strategy matrix, compatibility checker, nested CV for tuning |
| `robustness-engineering-task` | 3-tier stress test catalog, robustness envelope mapping, failure mode genealogy, recovery test sets |

### Operations and Governance
| Guideline | Covers |
|-----------|--------|
| `annotation-management-task` | Team structure, IAA targets, active learning loop, tooling comparison |
| `annotation-economics-task` | Fatigue modeling, task-specialization matching, cost-quality curves, disagreement valuation |
| `data-pipeline-monitoring-task` | Schema drift, distribution drift, freshness, volume alerts, backfill protocols |
| `multilingual-data-task` | Language coverage, script validation, tokenizer fertility, translation quality |
| `data-visualization-task` | 6-stage visualization protocol: raw → cleaned → split → embedding → labels → interactive |
| `dataset-governance-task` | Licensing, GDPR consent, EU AI Act compliance, DPAs, data subject rights |
| `cost-model-task` | Build vs buy vs license vs generate, annotation cost estimation, ROI framework |
| `data-strategy-foundation-models-task` | $100M data budget allocation, mixing ratios, scaling laws, redundancy saturation |
| `data-lifecycle-governance-task` | Birth → Adolescence → Adulthood → Retirement → Death with gate checks |
| `privacy-preserving-data-task` | DP-SGD with ε accounting, k-anonymity, membership inference attack validation |
| `sim-to-real-bridge-task` | Multi-axis gap analysis, domain randomization tuning, synthetic failure detection |
| `data-decision-integration-task` | Decision type→data req, frequency→refresh rate, stakes→certainty, reversibility→retention, accountability→provenance |
| `data-risk-integration-task` | Risk type→detection, severity→intensity, velocity→latency, correlation→discovery, mitigation→validation |
| `data-innovation-integration-task` | Innovation stage→data stage, type→experiment design, portfolio→coordination, failure→capture, scaling→sync |
| `data-performance-integration-task` | Performance dimension→data dimension, target→measurement, variance→explanation, benchmark→construction, improvement→tracking |
| `data-learning-organization-task` | Learning type→design, capture→knowledge extraction, distribution→mechanisms, retention→preservation, application→tracking |
| `data-communication-integration-task` | Audience archetypes→format, data→narrative, dashboard→decision, alerting→signal discipline, reporting→cadence |
| `data-culture-integration-task` | Culture type→diagnosis, literacy→measurement, psychological safety→engineering, incentives→alignment, rituals→institutionalization |
| `data-talent-integration-task` | Data roles→skills, hiring→data literacy signals, careers→lattices, retention→risk detection, upskilling→orchestration |
| `data-ethics-integration-task` | Stakeholder→harm taxonomy, consent→architecture, fairness→beyond metrics, redress→mechanisms, ethics gates→pipeline |
| `data-sustainability-integration-task` | Environmental→footprint, social→supply chain, economic→stress test, intergenerational→stewardship, circular→data economy |
| `data-ecosystem-integration-task` | Partners→sharing agreements, open-source→dependency health, regulatory→readiness, standards→participation, academia→collaboration |
| `data-resilience-integration-task` | M&A→integration, leadership→continuity, bankruptcy→preservation, obsolescence→migration, chaos→pipeline engineering |
| `data-governance-federation-integration-task` | Holding co→minimum standards, JV→partition, franchise→certification, matrix→dual ownership |
| `data-geopolitics-integration-task` | Sovereignty→regime mapping, cross-border→architectures, trade→agreements, sanctions→routing |
| `data-trust-integration-task` | Provenance→cryptographic chains, reputation→source scores, attestation→verification, ZK→readiness, decay→models |
| `data-interoperability-integration-task` | Semantic→ontologies, syntactic→format layers, organizational→governance alignment, temporal→schema evolution |

### Advanced / Bleeding Edge
| Guideline | Covers |
|-----------|--------|
| `data-attribution-task` | TRAK, influence functions, datamodels — trace training examples to predictions |
| `data-mixture-optimization-task` | DoReMi, DoGE, auto-curricula — learned data composition |
| `data-feedback-loop-task` | Self-training drift detection, pseudo-label confidence decay, optimal stopping criteria |
| `mechanistic-interpretability-data-task` | SAE training data design, circuit discovery, activation patching datasets |

### Source Catalogs
| Catalog | Covers |
|---------|--------|
| `medical-dataset-sources-task` | 70+ datasets: EHR, imaging, genomics, single-cell, drug discovery, clinical trials, audio |
| `protein-dataset-sources-task` | PDB, AlphaFold DB 200M, ESM Atlas 772M, STRING, PDBbind, ProteinGym, ESM embeddings |
| `chemistry-materials-sources-task` | COD, Materials Project, QM9, ANI-1x, OC20, MatBench |
| `neuroscience-sources-task` | Neuropixels, Allen Brain Observatory, HCP, MICrONS, FlyWire |
| `satellite-geospatial-sources-task` | Sentinel, Landsat, SpaceNet, BigEarthNet, Dynamic World |

---

## Skills

| Skill | What It Does | Path |
|-------|-------------|------|
| `dataset-cleaning` | Missing value handling, dedup, outlier treatment, normalization, audit trails | `scientific-skills/dataset-cleaning/` |
| `dataset-splitting` | Train/val/test splits, stratification, group/time-series splits, leakage prevention | `scientific-skills/dataset-splitting/` |
| `dataset-versioning` | DVC integration, manifest.json, semantic versioning, checksums, registry | `scientific-skills/dataset-versioning/` |
| `hf-datasets` | HuggingFace datasets: streaming, map/filter, push_to_hub, interleave, concatenate | `scientific-skills/hf-datasets/` |
| `embedding-analysis` | Sentence-transformers, NeMo Curator SemDedup, LSHBloom, GRAPE perplexity, DataRater scoring, distribution comparison | `scientific-skills/embedding-analysis/` |
| `llm-assisted-curation` | vLLM/SGLang-backed LLM-as-judge scoring, structured output, batch rewriting, synthetic generation | `scientific-skills/llm-assisted-curation/` |
| `data-card-writer` | Structured datasheets following Gebru et al. "Datasheets for Datasets" format | `scientific-skills/data-card-writer/` |
| `label-quality-audit` | Confident learning noise detection, per-class error rates, mislabeled example identification | `scientific-skills/label-quality-audit/` |
| `bias-audit` | Demographic parity, representation gaps, outcome disparity, intersectional bias | `scientific-skills/bias-audit/` |
| `benchmark-contamination-scan` | N-gram + embedding overlap scan against 60+ evaluation datasets | `scientific-skills/benchmark-contamination-scan/` |
| `data-diff` | Structured diff between dataset versions: what was added, removed, changed | `scientific-skills/data-diff/` |

---

## 2025-2026 Literature Integrated

| Paper | Venue | Where Integrated |
|-------|-------|-----------------|
| DataRater (Calian et al.) | NeurIPS 2025 | `embedding-analysis` (neighborhood quality scoring) |
| Why Less is More (Dohmatob et al.) | 2025 | Filtering philosophy throughout |
| GRAPE Score | 2025 | `embedding-analysis` (perplexity filtering) |
| NeMo Curator SemDedup (NVIDIA) | 2024-2025 | `embedding-analysis` (clustering dedup) |
| LSHBloom (Khan et al.) | 2025 | `embedding-analysis` (connected components dedup) |
| Blu-WERP (Rupesh et al.) | 2025 | Streaming pipeline patterns |
| TBDFiltering (Busa-Fekete et al.) | 2025 | Tree-based filtering strategy |
| Ensembled Multimodal Curation (Xu et al.) | 2025 | Multi-signal quality fusion |
| SlimQwen (Tang et al.) | arXiv:2605.08738, May 2026 | `model-compression-data-task` (MoE pruning+distillation, progressive schedules, MTP distillation) |
| DistilQwen2.5 (Wang et al.) | ACL 2025 Industry | `model-compression-data-task` (multi-agent teacher data selection, model fusion) |
| REAP | OpenMOSE 2026 | `model-compression-data-task` (router-weighted expert activation pruning) |
| DoReMi (Xie et al.) | 2024 | `data-mixture-optimization-task` |
| TRAK (Park et al.) | 2023 | `data-attribution-task` |
| Confident Learning (Northcutt et al.) | 2021 | `label-quality-audit` |
| Datasheets for Datasets (Gebru et al.) | 2021 | `data-card-writer` |

---

## Quick Reference: "I need to..."

| Task | Read This First |
|------|----------------|
| Create a new dataset from scratch | `dataset-creation-curation-task` |
| Clean messy data | `dataset-cleaning` (skill) |
| Split dataset for training | `dataset-splitting` (skill) |
| Check for benchmark contamination | `data-contamination-task` + `benchmark-contamination-scan` (skill) |
| Curate data for LLM pre-training | `llm-training-data-task` |
| Build preference data for RLHF | `rl-alignment-data-task` |
| Curate medical imaging data | `cv-dataset-task` + `medical-imaging-task` |
| Process single-cell RNA-seq | `single-cell-data-task` |
| Find medical datasets to train on | `medical-dataset-sources-task` |
| Find protein structure data | `protein-dataset-sources-task` |
| Design an evaluation that actually works | `evaluation-dataset-design-task` |
| Choose the right cross-validation | `cross-validation-strategy-task` |
| Audit for bias | `bias-audit` (skill) |
| Find mislabeled examples | `label-quality-audit` (skill) |
| Version a dataset | `dataset-versioning` (skill) |
| Write a data card | `data-card-writer` (skill) |
| Estimate dataset cost | `cost-model-task` |
| Comply with GDPR / EU AI Act | `dataset-governance-task` + `privacy-preserving-data-task` |
| Monitor pipeline health | `data-pipeline-monitoring-task` |
| Design annotation workflow | `annotation-management-task` + `annotation-economics-task` |
| Trace why model made a mistake | `data-attribution-task` |
| Self-train without collapse | `data-feedback-loop-task` |
| Bridge synthetic to real | `sim-to-real-bridge-task` |
| Test model robustness | `robustness-engineering-task` |
| Deprecate an old dataset | `data-lifecycle-governance-task` |
| Visualize dataset quality | `data-visualization-task` |
| Prune or distill a model | `model-compression-data-task` |
| Continually pre-train a model | `continual-pretraining-data-task` |
| Merge multiple fine-tuned models | `model-merging-data-task` |
| Optimize inference efficiency | `efficient-inference-data-task` |
