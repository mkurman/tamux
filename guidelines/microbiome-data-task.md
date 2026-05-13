---
name: microbiome-data-task
description: Curate microbiome and metagenomics datasets — 16S rRNA amplicon, shotgun metagenomics, taxonomic classification, functional profiling. Covers rarefaction curves, compositional data awareness, batch effects across extraction kits, and phylogenetic tree integration.
recommended_skills:
  - biopython
  - bioservices
  - polars-bio
  - lamindb
  - dataset-versioning
recommended_guidelines:
  - medical-bio-data-task
  - genomics-sequencing-data-task
  - training-data-design-principles
---

## Overview

Microbiome data is compositional, sparse, and dominated by batch effects from DNA extraction kits. A different extraction kit can change your "discovery" more than a drug treatment. Curation here requires compositional statistics awareness, phylogenetic-aware QC, and kit-level batch documentation.

## Phase 1: 16S rRNA Amplicon Data

### 1a. Read Quality

| Metric | Threshold |
|-------|-------|
| Q-score | > Q30 across amplicon |
| Read length | Consistent with amplicon (e.g., V3-V4 ≈ 460bp) |
| Chimera detection | UCHIME or vsearch; remove chimeras |
| Primer/adapter removal | cutadapt; trim primers completely |

### 1b. ASV vs OTU

- **ASV** (Amplicon Sequence Variants, DADA2/deblur): single-nucleotide resolution, reproducible across studies.
- **OTU** (97% identity clusters): legacy, less resolvable. Only use when integrating legacy data.
- **Prefer ASVs** for all new datasets.

### 1c. Rarefaction

Always rarefy to the same depth before diversity analysis. Report the rarefaction depth and how many samples were excluded.

```python
import numpy as np

def rarefy(counts, depth):
    """Rarefy a count vector to a given depth."""
    if counts.sum() < depth:
        return None  # insufficient depth
    probs = counts / counts.sum()
    return np.random.multinomial(depth, probs)

# Minimum depth: choose depth to retain > 90% samples
depths = count_table.sum(axis=1)
rarefaction_depth = int(np.percentile(depths, 10))  # keep 90% samples
```

### 1d. Contamination

- **Negative controls** (extraction blanks, PCR water) are MANDATORY.
- **decontam** (R package): frequency-based (inversely correlates with concentration) or prevalence-based (appears in controls).
- Remove ASVs present in controls at > 10% relative abundance unless biologically expected.

---

## Phase 2: Shotgun Metagenomics

### 2a. Host Decontamination

| Step | Tool |
|-------|-------|
| Align to host genome | Bowtie2 / BWA against hg38 |
| Remove host reads | `samtools view -f 12` (unmapped pairs) |
| Verify removal | > 95% non-host reads retained |

### 2b. Taxonomic Profiling

| Tool | Approach | Speed | Accuracy |
|-------|-------|-------|-------|
| **Kraken2 + Bracken** | K-mer based, fast | Fastest | Good |
| **MetaPhlAn4** | Marker gene based | Fast | Very good for known species |
| **mOTUs3** | Universal single-copy marker genes | Fast | Quantified relative abundance |

### 2c. Functional Profiling

- **HUMAnN3**: pathway-level functional profiling from metagenomes.
- **eggNOG-mapper**: functional annotation of predicted genes.
- **CheckM**: genome completeness and contamination for MAGs (Metagenome-Assembled Genomes).

---

## Phase 3: Compositional Data Awareness

Microbiome data is compositional — a change in one taxon forces changes in others. ABSOLUTE abundance changes cannot be inferred from relative abundance data alone.

| Rule | Consequence |
|-------|-------|
| **Never use standard correlation** | Pearson/Spearman on relative abundances is misleading |
| **Use Aitchison distance** (CLR-transformed) | Accounts for compositionality |
| **Never rarefy + CLR** | Rarefy for alpha diversity; CLR for beta diversity — not both |
| **ALR/CLR transform** | ALR = pick reference; CLR = geometric mean reference |
| **Report total counts** (qPCR/spike-in) | Only way to know if a taxon's RELATIVE increase is an ABSOLUTE increase |

---

## Phase 4: Batch Effects

The #1 source of variation in microbiome studies is the DNA extraction kit, not biology.

| Batch Variable | Detection | Fix |
|-------|-------|-------|
| **Extraction kit** | PCA colored by kit lot | Document; include as covariate |
| **Sequencing run** | PCA colored by run | Randomize samples across runs |
| **PCR cycle number** | Adequate cycles for low biomass | Record; include as covariate |
| **Collection method** | Swab vs. biopsy vs. stool card | Document; don't mix methods |
| **Storage / freeze-thaw** | Freeze-thaw cycle count | Flag high cycle samples |

---

## Phase 5: Databases and Taxonomy

| Database | Version | Use |
|-------|-------|-------|
| **SILVA** | v138.1+ | 16S/18S rRNA taxonomy |
| **Greengenes2** | 2022.10 | 16S taxonomy, whole-genome backbone |
| **GTDB** | R220 | Genome Taxonomy Database, species-level |
| **UniRef** | Latest | Functional annotation |
| **KEGG** | Latest | Pathway mapping |

**Lock database versions** — taxonomy changes between versions change your results.

---

## Quality Gate

Microbiome data is ready when:
- Negative controls included and decontamination applied.
- Rarefaction depth documented and > 90% samples retained.
- Extraction kit, PCR cycles, and sequencer run recorded as batch metadata.
- Compositional methods used for beta diversity and differential abundance.
- Taxonomy database version locked and reported.
- Total abundance method documented (qPCR, spike-in, or acknowledged as relative-only).
