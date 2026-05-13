---
name: epigenomics-data-task
description: Curate epigenomics datasets — ChIP-seq, ATAC-seq (bulk), DNA methylation (WGBS, array), and 3D chromatin (Hi-C). Covers peak calling QC, FRiP/IDR, bisulfite conversion efficiency, chromatin contact matrices, and batch-aware experimental design.
recommended_skills:
  - deeptools
  - polars-bio
  - pysam
  - biopython
  - bioservices
  - dataset-versioning
recommended_guidelines:
  - medical-bio-data-task
  - genomics-sequencing-data-task
  - bioinformatics-analysis-task
  - training-data-design-principles
---

## Overview

Epigenomics measures the regulatory layer on top of the genome — where proteins bind, which chromatin is open, what DNA is methylated. Every epigenomic assay has its own failure modes: ChIP-seq is antibody-dependent, bisulfite conversion is never 100% efficient, and Hi-C contact matrices are sparse and distance-dependent. Curation here is assay-specific QC plus cross-assay normalization.

---

## Phase 1: ChIP-seq Data

### 1a. ChIP-seq Library QC

| Metric | Tool | Threshold |
|-------|-------|-------|
| **FRiP** (Fraction of Reads in Peaks) | `deeptools` / ENCODE | > 1% (sharp marks like H3K4me3 > 5%) |
| **NSC / RSC** (phantompeakqualtools) | SPP / `deeptools` | NSC > 1.05, RSC < 1.0 |
| **Library complexity (NRF)** | `deeptools` / PRESEQ | NRF > 0.8 for high-complexity |
| **Strand cross-correlation** | `deeptools` | Clear peak at read length, not fragment length |
| **Reads in blacklist regions** | `deeptools` | < 10% |

```bash
# ChIP-seq QC with deeptools
plotFingerprint -b treatment.bam \
  --JSDsample control.bam \
  --labels treatment control \
  --plotFile fingerprint.png
```

### 1b. Peak Calling QC

| Issue | Detection | Fix |
|-------|-------|-------|
| Too few peaks (< 5K for TF, < 20K for broad mark) | `samtools flagstat` | Poor antibody, check IP efficiency |
| Too many peaks (> 200K for TF) | `samtools flagstat` | High background, increase input control |
| Peak width distribution unexpected | `deeptools plotFingerprint` | Broad marks (H3K27me3) should have wide peaks |
| Peaks in blacklist regions | BEDTools intersect | Remove from downstream |

### 1c. Replicate Concordance

- **IDR** (Irreproducible Discovery Rate): > 2 replicates → IDR < 0.05 for reproducible peaks.
- **Peak overlap**: Jaccard > 0.5 between replicates.
- Never combine replicates before peak calling — call peaks per-replicate and intersect.

### 1d. Antibody-Specific Controls

| Control | When Required | What It Guards Against |
|-------|-------|-------|
| **Input DNA** | Always | Background DNA shearing bias |
| **IgG control** | When available | Non-specific antibody binding |
| **Knockout validation** | New antibodies | Antibody specificity |
| **Spike-in (Drosophila/yeast)** | Quantitative comparison | Between-sample normalization |

---

## Phase 2: ATAC-seq Data (Bulk)

### 2a. ATAC-seq-Specific QC

| Metric | Tool | Threshold |
|-------|-------|-------|
| **TSS enrichment** | `deeptools` / ATACseqQC | > 7 (human/mouse) |
| **Nucleosome banding** | Fragment size distribution | Clear 147bp periodicity |
| **% Mitochondrial reads** | `samtools idxstats` | < 20% |
| **% Duplicate reads** | `picard MarkDuplicates` | < 40% |
| **Fragment size distribution** | `deeptools` | Peak at < 100bp (nucleosome-free) and ~200bp (mono-nucleosome) |

### 2b. Transposase Bias

Tn5 transposase has sequence bias creating false "open chromatin" signals:
- **HMMRATAC** or **Genrich** for bias-aware peak calling.
- Or apply bias correction post-calling using expected cleavage patterns.

---

## Phase 3: DNA Methylation Data

### 3a. Bisulfite Sequencing (WGBS/RRBS)

| Metric | Tool | Threshold |
|-------|-------|-------|
| **Bisulfite conversion rate** | spike-in lambda DNA or chrM | > 99% |
| **Coverage per CpG** | Bismark / `methylKit` | > 5x (WGBS), > 10x (RRBS) |
| **M-bias plot** | Bismark | No methylation bias at read ends |
| **CpG vs CHG/CHH methylation** | Bismark | CHG/CHH near zero (animals) |

```bash
bismark --genome ref/ -1 R1.fq -2 R2.fq
bismark_methylation_extractor --bedGraph --comprehensive --merge_non_CpG *.bam
```

### 3b. Array-Based Methylation (450K / EPIC)

| Issue | Detection | Fix |
|-------|-------|-------|
| **Probe cross-reactivity** | Known list of cross-reactive probes | Remove |
| **SNP at probe site** | `minfi::dropLociWithSnps` | Remove |
| **Detection P-value** | `minfi::detectionP` | Remove probes with p > 0.01 in > 5% samples |
| **Sex mismatch** | X-chromosome methylation vs reported sex | Flag sample |
| **Batch effects** | PCA by array/slide/plate | Normalize (BMIQ, SWAN, Noob, quantile) |

---

## Phase 4: Hi-C and 3D Genome

### 4a. Hi-C Library QC

| Metric | Tool | Threshold |
|-------|-------|-------|
| **Cis/trans ratio** | HiCPro / Juicer | > 70% cis |
| **Valid interaction pairs** | HiCPro | > 50% of reads |
| **Long-range cis (> 20kb)** | HiCPro | > 30% |
| **Duplicate rate** | HiCPro | < 30% |
| **Library complexity** | HiCPro / PRESEQ | Report |

```bash
HiC-Pro -c config.txt -i raw/ -o output/
```

### 4b. Contact Matrix QC

- **Coverage**: Total contacts > 100M valid pairs. Below 50M, TAD and loop calling unreliable.
- **Distance decay**: log-log plot of contact frequency vs. genomic distance. Should be approximately linear (P(s) ∼ s⁻¹).
- **Compartment analysis**: First eigenvector of contact matrix should recapitulate known A/B compartments.
- **Replicate reproducibility**: HiCRep SCC > 0.9; TAD boundary Jaccard > 0.7.

---

## Phase 5: Cross-Assay Integration

- ALL epigenomic coordinates must use the SAME reference genome assembly.
- Chromosome naming convention: pick "chr1" or "1" and enforce it.
- Batch metadata is mandatory: sequencer model, flowcell, lane, library prep date, antibody lot.
- For differential analysis (methylation, accessibility), include batch as a covariate.

---

## Quality Gate

Epigenomics data is ready when:
- ChIP-seq passes FRiP, NSC/RSC, and IDR thresholds.
- ATAC-seq TSS enrichment > 7 with clear nucleosome periodicity.
- Bisulfite conversion rate > 99%; CpG coverage meets minimum.
- Array probes filtered for cross-reactivity, SNPs, and detection P-value.
- Hi-C libraries have > 50% valid pairs; replicate SCC > 0.9.
- All coordinates use the same reference genome assembly.
- Batch metadata (sequencer, flowcell, lane, antibody lot) recorded.
