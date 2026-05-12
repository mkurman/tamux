---
name: proteomics-metabolomics-data-task
description: Curate mass spectrometry proteomics and metabolomics datasets — peptide identification, protein quantification, PTM analysis, metabolite annotation, and spectral library QC.
recommended_skills:
  - pyopenms
  - matchms
  - bioservices
  - biopython
  - lamindb
  - dataset-versioning
recommended_guidelines:
  - medical-bio-data-task
  - bioinformatics-analysis-task
  - training-data-design-principles
---

## Overview

Mass spectrometry proteomics and metabolomics produce some of the most complex data in biology. A single run generates millions of spectra, each a vector of m/z and intensity pairs. Curation here demands spectral-level QC, peptide/protein FDR control, and quantitative normalization that accounts for run-to-run instrument drift.

## Phase 1: Raw MS Data and Spectral QC

### 1a. File Format and Conversion

| Format | Type | Tool |
|-------|-------|-------|
| `.raw` (Thermo) | Vendor binary | ProteoWizard `msconvert` |
| `.d` (Bruker) | Vendor directory | ProteoWizard |
| `.wiff` (SCIEX) | Vendor binary | ProteoWizard |
| `.mzML` | Open standard | Native in `pyopenms` |
| `.mzXML` | Legacy open | Convert to mzML |
| `.mgf` | Peak lists | Read via `pyopenms` |

**Rule**: Always convert to `.mzML` before archiving. Vendor formats are not guaranteed readable in 5 years.

### 1b. MS Run-Level QC

| Metric | Detection | Threshold |
|-------|-------|-------|
| Total ion current (TIC) | `pyopenms` | Within 2 SD of cohort mean |
| MS1 peak count | `pyopenms` | > 50K for DDA, > 200K for DIA |
| MS/MS acquisition rate | `pyopenms` | Consistent across run (no drop-off) |
| Precursor m/z error | Search engine output | < 10 ppm (Orbitrap), < 20 ppm (TOF) |
| Chromatographic peak width | `pyopenms` | 20-60s FWHM typical; < 10s = spray instability |
| Retention time drift | `pyopenms` | < 2 min across runs |

```python
from pyopenms import MSExperiment, MzMLFile

exp = MSExperiment()
MzMLFile().load("sample.mzML", exp)

ms1_spectra = [s for s in exp if s.getMSLevel() == 1]
ms2_spectra = [s for s in exp if s.getMSLevel() == 2]

tic = sum(sum(s.get_peaks()[1]) for s in ms1_spectra)
ms1_count = len(ms1_spectra)
ms2_count = len(ms2_spectra)
print(f"MS1: {ms1_count}, MS2: {ms2_count}, TIC: {tic:.2e}")
```

## Phase 2: Peptide and Protein Identification

### 2a. Search Engine Configuration

| Parameter | Recommendation | Why |
|-------|-------|-------|
| Precursor mass tolerance | 10 ppm (Orbitrap), 20 ppm (TOF) | Instrument-dependent |
| Fragment mass tolerance | 0.02 Da (Orbitrap), 0.05 Da (TOF) | Higher resolution = tighter tolerance |
| Enzyme | Trypsin/P (semi-specific for discovery) | Full specificity if labeled |
| Max missed cleavages | 2 | Up to 3 for label-free |
| Fixed modifications | Carbamidomethyl (C) | Always for reduced/alkylated |
| Variable modifications | Oxidation (M), Acetyl (Protein N-term) | Keep limited — combinatorial explosion |
| FDR threshold | 1% at PSM, peptide, AND protein level | Three-level FDR control |

### 2b. FDR Control

```python
# Percolator-style target-decoy FDR estimation
def compute_qvalue(target_scores, decoy_scores):
    """Simple target-decoy FDR estimation."""
    all_scores = [(s, True) for s in target_scores] + [(s, False) for s in decoy_scores]
    all_scores.sort(key=lambda x: x[0], reverse=True)
    
    qvalues = []
    n_decoy, n_target = 0, 0
    for score, is_target in reversed(all_scores):
        if is_target:
            n_target += 1
        else:
            n_decoy += 1
        fdr = (n_decoy + 1) / (n_target + n_decoy) if (n_target + n_decoy) > 0 else 0
        qvalues.insert(0, fdr)
    return qvalues
```

Three levels of FDR control required:
1. **PSM-level**: 1% FDR on peptide-spectrum matches
2. **Peptide-level**: 1% FDR on unique peptide sequences
3. **Protein-level**: 1% FDR on protein groups (use leading razor protein)

### 2c. Protein Inference

- **Protein grouping**: Shared peptides → protein group. Report group, not individual accessions.
- **Parsimony rule**: Report the minimal set of proteins that explain all peptides.
- **Contaminant removal**: Filter out common contaminants (keratin, trypsin, BSA, casein). Use cRAP database.

## Phase 3: Quantitative Proteomics

### 3a. Label-Free Quantification (LFQ)

| Issue | Detection | Fix |
|-------|-------|-------|
| Missing values between runs | > 30% missing in a condition | Imputation (not for differential testing without caution) |
| Run-to-run intensity drift | Boxplot of log2 intensities | Median normalization or quantile normalization |
| Batch effects | PCA colored by run date | ComBat or limma::removeBatchEffect |
| Saturation | Max intensity plateau | Dilute and re-run |

```python
# Quantile normalization
import numpy as np

def quantile_normalize(matrix):
    """Normalize columns to the same distribution."""
    rank_mean = np.sort(matrix, axis=0).mean(axis=1)
    ranks = np.argsort(np.argsort(matrix, axis=0), axis=0)
    return rank_mean[ranks]
```

### 3b. TMT / Isobaric Labeling

- **Reporter ion purity**: Check for isolation interference (co-isolated peptides inflate ratios).
- **Ratio compression**: TMT ratios are compressed toward 1.0. Use MS3-based methods (SPS-MS3) for accurate ratios.
- **Reference channel**: Normalize to pooled reference channel in each run.

### 3c. PTM (Post-Translational Modification) Analysis

| PTM | Mass Shift (Da) | Localization Score |
|-------|-------|-------|
| Phosphorylation (STY) | +79.966 | PhosphoRS or Ascore > 13 |
| Acetylation (K) | +42.011 | Site localization probability > 0.75 |
| Ubiquitination (K) | +114.043 (GlyGly) | Requires GG-K enrichment |
| Oxidation (M) | +15.995 | Common artifact; flag if > 10% of peptides |

## Phase 4: Metabolomics-Specific

### 4a. Metabolite Identification

- **Level 1** (gold standard): Match to authentic standard (RT + MS/MS).
- **Level 2**: Spectral library match (MS/MS similarity > 0.8).
- **Level 3**: In-silico fragmentation match (e.g., SIRIUS, CSI:FingerID).
- **Level 4**: Molecular formula prediction only.
- **Level 5**: Exact mass only (NOT sufficient for publication).

### 4b. Metabolomics QC

- **Pooled QC samples**: Inject every 5-10 runs. CV < 30% in pooled QCs for > 80% of features.
- **Internal standards**: Spiked-in standards at known concentration. CV < 15%.
- **Blanks**: Process blanks to subtract background. Feature in blanks < 3x blank intensity.
- **Ion suppression**: Post-column infusion to detect matrix effects in chromatographic regions.

## Phase 5: Spectral Libraries

- **Spectral library QC**: Remove spectra with < 5 fragment ions. Check for precursor contamination.
- **Library format**: MSP or MGF for sharing.
- **Match score**: Use spectral entropy or dot-product. > 0.7 for confident match.

## Quality Gate

Proteomics/metabolomics data is ready when:
- All raw files converted to mzML; vendor formats archived separately.
- PSM, peptide, and protein FDR all < 1%.
- Contaminants removed (cRAP database).
- Quantitative data normalized with documented method.
- Batch metadata (run date, column batch, reagent lot) recorded.
- PTM sites have localization scores above threshold.
- Metabolite identifications annotated with confidence level (1-5).
- Pooled QC CV < 30% for > 80% of features.
