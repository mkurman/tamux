---
name: single-cell-data-task
description: Curate single-cell datasets — scRNA-seq, scATAC-seq, spatial transcriptomics, multiome. Covers doublet detection, ambient RNA removal, mitochondrial filtering, batch correction, cell-type annotation validation, and integration quality metrics.
recommended_skills:
  - scanpy
  - anndata
  - scvi-tools
  - scvelo
  - cellxgene-census
  - lamindb
  - embedding-analysis
  - dataset-versioning
recommended_guidelines:
  - medical-bio-data-task
  - genomics-sequencing-data-task
  - bioinformatics-analysis-task
  - training-data-design-principles
---

## Overview

Single-cell data is messy by nature: each cell is a noisy observation, ambient RNA contaminates the signal, and every experiment has batch effects. Curation is not optional — it's the difference between finding biology and finding artifacts.

## Phase 1: Raw Count Matrix QC

### 1a. Cell-Level Quality Filters

Apply these filters BEFORE normalization. Use per-sample cutoffs, not dataset-wide:

| Metric | Detection | Typical Threshold | Rationale |
|-------|-------|-------|-------|
| **UMI count** | `sc.pp.calculate_qc_metrics` | > 1000 (10x 3'), > 500 (Smart-seq) | Dead/empty droplets |
| **Number of genes** | `sc.pp.calculate_qc_metrics` | > 500 (10x 3'), > 2000 (Smart-seq) | Degraded cells |
| **% Mitochondrial reads** | `sc.pp.calculate_qc_metrics` | < 20% (most tissues), < 10% (brain/heart) | Dying/damaged cells |
| **% Ribosomal reads** | `sc.pp.calculate_qc_metrics` | < 40% | Technical artifact |
| **Doublet score** | Scrublet / DoubletDetection / scDblFinder | Remove predicted doublets | Two cells in one droplet |

```python
import scanpy as sc
import scrublet as scr
import numpy as np

adata = sc.read_10x_h5("filtered_feature_bc_matrix.h5")

# Compute QC metrics
adata.var["mt"] = adata.var_names.str.startswith("MT-")
adata.var["ribo"] = adata.var_names.str.startswith(("RPS", "RPL"))
sc.pp.calculate_qc_metrics(adata, qc_vars=["mt", "ribo"], percent_top=None, inplace=True)

# Adaptive QC — use MAD (Median Absolute Deviation), not hard cutoffs
def is_outlier(adata, metric, nmads=5):
    from scipy.stats import median_abs_deviation
    M = adata.obs[metric]
    return (M > np.median(M) + nmads * median_abs_deviation(M))

# Flag, don't blindly remove — inspect the distribution first
adata.obs["outlier"] = (
    is_outlier(adata, "n_genes_by_counts", nmads=5) |
    is_outlier(adata, "total_counts", nmads=5) |
    is_outlier(adata, "pct_counts_mt", nmads=5)
)

# Doublet detection
scrub = scr.Scrublet(adata.X)
doublet_scores, predicted_doublets = scrub.scrub_doublets()
adata.obs["doublet_score"] = doublet_scores
adata.obs["predicted_doublet"] = predicted_doublets
```

### 1b. Empty Droplet vs. Cell

For droplet-based data (10x), distinguish real cells from empty droplets:

```python
# Use DropletUtils-like approach via knee plot inflection
# Keep cells above the "knee" in the UMI count distribution
# This is often done by the pipeline (Cell Ranger), but verify
sc.pp.filter_cells(adata, min_counts=1000)
```

### 1c. Ambient RNA

Ambient RNA is the biggest undiagnosed problem in single-cell data. It mimics low-level expression and creates false-positive "contamination" patterns.

- **SoupX** or **CellBender** for ambient RNA removal.
- Run on each sample separately — ambient profiles differ per sample.
- After correction, check that known mutually exclusive markers no longer co-express.

## Phase 2: Normalization and Feature Selection

### 2a. Normalization

```python
# Standard pipeline:
sc.pp.normalize_total(adata, target_sum=1e4)  # library-size normalization
sc.pp.log1p(adata)  # log-transform

# For integration / batch correction:
# Use scVI or similar model-based normalization instead
# Do NOT use simple log-normalization before integration
```

### 2b. Highly Variable Gene Selection

```python
# Select HVGs BEFORE batch correction
# Use sample-aware selection, not dataset-wide
sc.pp.highly_variable_genes(adata, n_top_genes=2000, batch_key="sample_id")
adata = adata[:, adata.var.highly_variable]
```

## Phase 3: Batch Correction and Integration

### 3a. Batch Effect Detection

```python
# Check batch effects BEFORE correction
sc.tl.pca(adata)
sc.pl.pca(adata, color=["sample_id", "cell_type"], ncols=1)
# If sample_id dominates the PCA → batch effects are strong
```

### 3b. Integration Methods (Choose Wisely)

| Method | When to Use | Caveats |
|-------|-------|-------|
| **scVI** | General purpose, large datasets | Requires GPU for speed; good at preserving biology |
| **Harmony** | Fast, good for atlas-scale | Can over-correct weak biological signals |
| **Scanorama** | Heterogeneous data | Conservative — may leave some batch effects |
| **BBKNN** | Graph-based, fast | Only produces neighborhood graph, not corrected expression |
| **ComBat** | Simple, well-understood | Parametric; struggles with strong non-linear effects |

```python
import scvi

# scVI integration
scvi.model.SCVI.setup_anndata(adata, batch_key="sample_id")
model = scvi.model.SCVI(adata)
model.train()
adata.obsm["X_scVI"] = model.get_latent_representation()

# Verify integration quality
sc.pp.neighbors(adata, use_rep="X_scVI")
sc.tl.umap(adata)
sc.pl.umap(adata, color=["sample_id", "cell_type"])
```

### 3c. Integration Validation

| Metric | How to Check | Good Sign |
|-------|-------|-------|
| **Batch mixing** | kBET, iLISI | Cells from different batches mixed in UMAP |
| **Biology preservation** | cLISI, ASW_celltype | Same cell types cluster together |
| **No over-correction** | UMAP colored by known marker genes | Marker patterns preserved after integration |
| **No new clusters** | Compare pre/post clusters | Same number of meaningful clusters |

## Phase 4: Cell-Type Annotation

### 4a. Annotation Approaches

| Approach | Accuracy | Effort | Best For |
|-------|-------|-------|-------|
| **Automated reference mapping** | Medium-High | Low | Well-studied tissues, healthy samples |
| **Marker-based manual** | High | High | Novel biology, disease states |
| **LLM-assisted** | Medium | Medium | Exploratory, hypothesis generation |
| **Supervised classifier** | High | High (training data) | Production, reproducible pipelines |

### 4b. Automated Annotation with Reference

```python
# CellTypist for immune cells
import celltypist
predictions = celltypist.annotate(adata, model="Immune_All_High.pkl", majority_voting=True)
adata.obs["celltypist_label"] = predictions.predicted_labels

# scArches / scVI reference mapping
scvi.model.SCVI.prepare_query_anndata(query_adata, reference_model)
query_model = scvi.model.SCVI.load_query_data(query_adata, reference_model)
query_model.train(max_epochs=200, plan_kwargs=dict(weight_decay=0.0))
```

### 4c. Annotation Validation

- **Marker gene expression**: Check that annotated cell types express canonical markers.
- **Confusion with reference**: CellTypist reports probability — flag low-confidence (< 0.7) assignments.
- **Cluster purity**: Within each annotated cluster, > 70% of cells should have the same label.
- **Expert review**: Spot-check 50-100 cells per cell type with a biologist.

## Phase 5: Modality-Specific Considerations

### 5a. scATAC-seq

- **TSS enrichment**: > 7 expected. Low enrichment = poor library quality.
- **Nucleosome banding**: Clear periodicity at 147bp multiples.
- **FRiP (Fraction of Reads in Peaks)**: > 20% for good libraries.
- **Peak calling**: Use MACS2 per sample, then merge peaks across samples.

### 5b. Spatial Transcriptomics (Visium, MERFISH, Xenium)

- **Tissue morphology**: Check that clusters correspond to anatomical structures.
- **Spot deconvolution**: For Visium, deconvolve spots into cell-type proportions (RCTD, cell2location).
- **Resolution mismatch**: Visium spots contain 1-10 cells; single-cell resolution methods are fundamentally different datasets.

### 5c. Multiome (RNA + ATAC)

- Both modalities must pass their individual QC before joint analysis.
- Check correlation between RNA and ATAC clusters — they should agree.
- Low ATAC quality is NOT rescued by good RNA quality. Filter independently.

## Quality Gate

Single-cell data is ready when:
- Per-sample QC thresholds are justified and applied.
- Ambient RNA removal is performed and verified.
- Doublet detection is run and doublets are flagged.
- Batch effects are visualized and documented (PCA before correction).
- Integration method is chosen with documented rationale.
- Batch mixing (kBET/iLISI) and biology preservation (cLISI/ASW) metrics are reported.
- Cell-type annotations are validated with marker gene expression.
- Cell-level metadata includes: sample_id, batch, doublet_score, QC pass/fail flags.
