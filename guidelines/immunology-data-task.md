---
name: immunology-data-task
description: Curate immunology datasets — TCR/BCR immune repertoires, flow/mass cytometry, cytokine profiling, and immunopeptidomics. Covers clonotype analysis, panel design validation, compensation, and immune-specific batch correction.
recommended_skills:
  - scanpy
  - anndata
  - scvi-tools
  - biopython
  - lamindb
  - embedding-analysis
  - dataset-versioning
recommended_guidelines:
  - medical-bio-data-task
  - single-cell-data-task
  - genomics-sequencing-data-task
  - training-data-design-principles
---

## Overview

Immunology data lives at the intersection of single-cell biology and protein biochemistry. It has unique challenges: adaptive immune receptors are combinatorially diverse, flow cytometry requires compensation matrices, and immune states are inherently dynamic. Curation here requires domain-specific QC that general single-cell or genomics pipelines miss.

## Phase 1: TCR/BCR Immune Repertoire Data

### 1a. Repertoire Sequencing Data Types

| Data Type | What It Captures | Format | Key QC Metrics |
|-------|-------|-------|-------|
| **Bulk TCR-seq** | Aggregate repertoire of T-cells | FASTQ → clonotype table | Clonotype count, diversity |
| **Single-cell TCR-seq** | Paired α/β chains per cell | 10x V(D)J output | Pairing rate, cell barcode match |
| **BCR-seq (heavy + light)** | Paired IgH/IgL per B-cell | 10x V(D)J output | Somatic hypermutation rate, isotype |
| **AIRR-seq** | Adaptive immune repertoire | AIRR-compliant TSV | Community standards |

### 1b. Clonotype Quality Filtering

```python
# Essential clonotype-level QC
def filter_clonotypes(clonotype_df):
    """Filter clonotype table to high-confidence entries."""
    
    # Remove non-productive rearrangements
    df = clonotype_df[clonotype_df["productive"] == True]
    
    # Remove incomplete V(D)J calls
    df = df[df["v_call"] != "None"]
    df = df[df["j_call"] != "None"]
    
    # Minimum read support
    df = df[df["reads"] >= 3]  # remove singletons/doubletons
    
    # CDR3 must be in-frame (no stop codons)
    df = df[~df["cdr3_aa"].str.contains(r"\*")]
    
    # CDR3 length filter (typical range)
    df = df[df["cdr3_aa"].str.len().between(5, 30)]
    
    return df
```

### 1c. Paired Chain Validation (scTCR/scBCR)

| Check | Action |
|-------|-------|
| Cell barcode not in GEX matrix | Remove — likely ambient or barcode swap |
| Only α or only β chain (not both) | Flag as "single-chain". Most analyses need paired |
| Multiple α or β chains per barcode | Keep dominant chains (highest UMI) |
| V-gene usage outlier | Check for alignment artifact |
| Extreme clonal expansion (> 50% of repertoire) | May indicate a single expanded clone — flag |

### 1d. Diversity Metrics

```python
from scipy.stats import entropy
import numpy as np

# Shannon entropy (normalized)
clonotype_counts = df.groupby("clonotype_id").size()
shannon = entropy(clonotype_counts)
n_unique = len(clonotype_counts)
normalized_shannon = shannon / np.log(n_unique)  # 0-1 scale

# Clonality (1 - normalized_shannon)
# Near 0 = polyclonal, near 1 = monoclonal

# Gini-Simpson diversity
simpson = 1 - np.sum((clonotype_counts / clonotype_counts.sum()) ** 2)
```

| Metric | Healthy Donor Range | Warning Flag |
|-------|-------|-------|
| Unique clonotypes (CDR3) | 10K-100K (bulk TCR) | < 1K = low diversity, possible contamination |
| Normalized Shannon | 0.7-0.95 | < 0.5 = oligoclonal |
| Top clone frequency | < 5% | > 20% = clonal expansion (check clinical context) |
| Paired chain recovery | > 80% (10x) | < 50% = poor library |

### 1e. AIRR Standards Compliance

The Adaptive Immune Receptor Repertoire (AIRR) community maintains strict data standards:
- **AIRR-C v1.4** format for repertoire data (MiAIRR compliant).
- Required fields: `sequence_id`, `sequence`, `v_call`, `d_call`, `j_call`, `cdr3_aa`, `productive`.
- Use `airr` Python package for validation.

---

## Phase 2: Flow and Mass Cytometry

### 2a. Panel Design Validation

| Issue | Detection | Fix |
|-------|-------|-------|
| **Compensation errors** | Spreading error, negative populations | Recalculate compensation matrix |
| **Fluorophore spillover** | Unexpected double-positive populations | Redesign panel (separate tandem dyes) |
| **Dead cell signal** | High background in all channels | Use live/dead stain; gate out dead cells |
| **Doublets** | FSC-A vs FSC-H linear relationship | Gate singlets |
| **Batch drift** | MFI shift over acquisition time | Normalize to beads, record acquisition time |

### 2b. Gating Quality

- **Gates should be documented** with rationale, not just coordinates.
- **Inter-operator gating variability** must be measured: have two analysts gate the same data independently.
- **Back-gating**: after gating a population, project it back onto parent gates to verify the gate captures what you think it does.
- **Manual gates are not reproducible**: prefer automated clustering (FlowSOM, PhenoGraph) with manual annotation of clusters.

### 2c. Flow Cytometry Standard (FCS) Validation

```python
import flowio
import numpy as np

def validate_fcs(filepath):
    """Validate an FCS file for common issues."""
    fcs = flowio.FlowData(filepath)
    
    checks = {}
    
    # Event count
    checks["n_events"] = fcs.event_count
    
    # Compensation
    checks["has_spill"] = "$SPILL" in fcs.text or "SPILL" in fcs.text
    
    # Parameter count consistency
    checks["param_match"] = (
        int(fcs.text.get("$PAR", 0)) == len(fcs.channels)
    )
    
    # Check for negative values in fluorescence channels
    data = np.array(fcs.events, dtype=np.float32)
    neg_fraction = (data < 0).mean(axis=0)
    checks["channels_with_negatives"] = int(np.sum(neg_fraction > 0.05))
    
    return checks
```

### 2d. Mass Cytometry (CyTOF) Specific

- **Metal isotope purity**: check for oxidation (+16 Da) and impurity signals.
- **Normalization**: EQ bead-based normalization per Helios protocol.
- **Debarcoding**: verify sample assignment accuracy > 99%.
- **Cell number**: minimum 10K events per sample for clustering; 50K+ for rare population detection.

---

## Phase 3: Cytokine and Protein Profiling

### 3a. Multiplexed Immunoassays (Luminex, MSD, Olink)

| Issue | Detection | Threshold |
|-------|-------|-------|
| Below detection limit | Concentration below LLOQ | Flag, do not impute to 0 |
| Above detection limit | Concentration above ULOQ | Flag as censored |
| CV between replicates | > 20% CV | Flag for review |
| Batch effect (plate-to-plate) | PCA colored by plate | Normalize using bridge samples |
| Matrix effect | Serum vs plasma mismatch | Document matrix; don't mix |

### 3b. ELISpot / Flow-based Cytokine

- **Spot counting variability**: automated counters have ±15% CV. Report CV between replicate wells.
- **Background subtraction**: unstimulated control wells. Flag if background > 20% of stimulated.
- **Stimulation consistency**: PMA/ionomycin response varies by donor and time. Include a positive control well to verify stimulation worked.

---

## Phase 4: Immunopeptidomics

- **FDR control**: Percolator or similar for peptide-spectrum match scoring. Target 1% FDR at peptide level.
- **HLA allele typing**: Must be 4-digit resolution (e.g., HLA-A*02:01). Infer from MS data or genotype separately.
- **Binding affinity**: Use NetMHCpan. Filter for predicted binders (< 500 nM or rank < 2%).
- **Contaminant removal**: Filter out contaminants (keratins, trypsin, BSA).

## Quality Gate

Immunology data is ready when:
- TCR/BCR clonotypes pass productive, CDR3 length, and read-support filters.
- AIRR compliance is verified for repertoire data.
- Flow cytometry files pass FCS validation and compensation checks.
- Gating strategy is documented with inter-operator agreement measured.
- Multiplexed assay data has CV < 20% on replicate measurements.
- Batch metadata includes: donor ID, timepoint, stimulation condition, acquisition date, instrument.
- All immune-specific flags (single-chain, clonal expansion, below LLOQ) are in metadata.
