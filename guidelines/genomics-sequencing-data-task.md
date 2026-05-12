---
name: genomics-sequencing-data-task
description: Curate DNA/RNA sequencing datasets — FASTQ quality control, read alignment (BAM), variant calling (VCF), coverage metrics, contamination detection, and reference genome management.
recommended_skills:
  - polars-bio
  - pysam
  - biopython
  - tiledbvcf
  - gget
  - deeptools
  - bioservices
  - dataset-versioning
recommended_guidelines:
  - medical-bio-data-task
  - bioinformatics-analysis-task
  - training-data-design-principles
---

## Overview

Sequencing data is the foundation of modern genomics. Errors at this stage propagate through every downstream analysis. FASTQ quality, alignment accuracy, and variant calling confidence are not post-hoc checks — they are the dataset.

## Phase 1: FASTQ Quality Control

### 1a. Per-Base Quality Scores

Every FASTQ file must pass these checks BEFORE alignment:

| Metric | Tool | Threshold |
|-------|-------|-------|
| Per-base quality (Phred) | FastQC | Median > 30 (Q30), no position < 20 |
| Per-sequence quality | FastQC | > 80% of reads have mean Q > 28 |
| GC content | FastQC | Within ±5% of expected for organism |
| Adapter contamination | FastQC / cutadapt | < 1% of reads |
| Overrepresented sequences | FastQC | No single sequence > 1% (unless expected) |
| Read length distribution | FastQC | Consistent length, no truncation artifacts |
| N content | FastQC | < 5% Ns per read |

```bash
# FastQC batch run
fastqc -t 8 -o qc_reports/ *.fastq.gz
multiqc qc_reports/ -o multiqc_report/

# Adapter trimming
cutadapt -a ADAPTER_SEQ -A ADAPTER_SEQ_R2 \
  -o trimmed_R1.fastq.gz -p trimmed_R2.fastq.gz \
  --minimum-length 50 -q 30 \
  input_R1.fastq.gz input_R2.fastq.gz
```

### 1b. Contamination Detection

- **Cross-species**: Align a sample of reads against common contaminants (mycoplasma, E. coli, human for non-human samples) using Kraken2 or Centrifuge.
- **Sample swap**: Verify that SNP fingerprints match expected donor genotype.
- **Index hopping**: For multiplexed runs, check for unexpected barcode combinations.

### 1c. Read-Level QC Decisions

| Issue | Action |
|-------|-------|
| Low-quality bases at read ends | Trim (not filter — trim restores alignment rate) |
| Adapter dimer (no insert) | Remove |
| PCR duplicates (identical start/end) | Mark, don't remove (they carry information in RNA-seq) |
| Short reads after trimming (< 50bp) | Remove |
| Ambiguous base calls (high N content) | Remove |

## Phase 2: Alignment and Post-Alignment QC

### 2a. Alignment Metrics

After aligning reads to a LOCKED reference genome:

| Metric | What It Catches | Threshold |
|-------|-------|-------|
| Overall alignment rate | Failed library prep | > 70% for DNA, > 60% for RNA |
| Uniquely mapped rate | Repetitive regions, poor library | > 80% of aligned reads |
| Proper pair rate (PE) | Library insert size issues | > 80% of paired reads |
| Duplicate rate | PCR over-amplification | DNA: < 30%, RNA: expected higher |
| Insert size distribution | Library prep quality | Single peak, expected size |
| Coverage uniformity | GC bias, capture bias | < 2-fold variation across target |
| % MT reads (scRNA-seq) | Cell viability | < 20% typically |
| % rRNA (RNA-seq) | rRNA depletion efficiency | < 10% |

```bash
# Alignment with BWA-MEM
bwa mem -t 8 -R "@RG\tID:sample\tSM:sample" ref.fa R1.fq R2.fq | \
  samtools sort -@ 4 -o sample.bam -
samtools index sample.bam

# Collect metrics
picard CollectAlignmentSummaryMetrics I=sample.bam O=align_metrics.txt R=ref.fa
picard CollectInsertSizeMetrics I=sample.bam O=insert_metrics.txt H=insert_hist.pdf
samtools flagstat sample.bam > flagstat.txt
```

### 2b. Coverage Analysis

```bash
# Genome-wide coverage
mosdepth -n -t 4 sample sample.bam

# Target region coverage (exome/panel)
samtools depth -b targets.bed sample.bam | \
  awk '{sum+=$3; if($3<20) low++} END {
    print "Mean:", sum/NR, "Bases <20x:", low
  }'
```

Coverage requirements by application:
- **Germline WGS**: ≥ 30x mean
- **Germline WES**: ≥ 100x on target
- **Somatic tumor**: ≥ 80x tumor, ≥30x normal
- **RNA-seq**: ≥ 30M paired reads per sample
- **scRNA-seq**: ≥ 20K reads per cell (10x)

## Phase 3: Variant Calling Data

### 3a. VCF Quality Metrics

| Metric | Tool | Threshold |
|-------|-------|-------|
| Transition/Transversion ratio (Ti/Tv) | bcftools stats | WGS: 2.0-2.2, WES: 2.8-3.2 |
| Het/Hom ratio | bcftools stats | ~1.5 for human WGS |
| dbSNP concordance | bcftools + vcfanno | > 95% for known sites |
| Depth distribution | bcftools stats | No systematic drop in variant-supporting reads |
| Strand bias (FS/SOR) | GATK VQSR | FS < 60, SOR < 3 |
| QUAL score distribution | bcftools stats | Expected peak at high QUAL |

### 3b. Variant Filtering

```bash
# GATK hard filtering (germline)
gatk VariantFiltration \
  -R ref.fa -V raw.vcf -O filtered.vcf \
  --filter-name "QD2" --filter-expression "QD < 2.0" \
  --filter-name "FS60" --filter-expression "FS > 60.0" \
  --filter-name "MQ40" --filter-expression "MQ < 40.0" \
  --filter-name "SOR3" --filter-expression "SOR > 3.0"

# VQSR (when enough variants exist)
gatk VariantRecalibrator -R ref.fa -V raw.vcf \
  --resource:dbsnp dbsnp.vcf --resource:mills mills.vcf \
  -an QD -an MQ -an MQRankSum -an ReadPosRankSum -an FS -an SOR \
  -mode SNP -O recal.vcf --tranches-file tranches
```

### 3c. Population-Level QC

When curating datasets across multiple samples:
- **Relatedness**: KING or PLINK --genome. Remove one from each related pair unless family structure is intended.
- **Population stratification**: PCA. Flag outliers > 6 SD from population mean on any of top 10 PCs.
- **Missingness**: Remove variants with > 5% missing calls, samples with > 10% missing.
- **Allele frequency spectrum**: Check against expected distribution (no excess of rare variants = contamination signal).

## Phase 4: RNA-seq Specific

### 4a. Expression Quantification

- Use transcript-level quantification (Salmon/kallisto) rather than gene-level counting for more accurate estimates.
- TPM, not raw counts, for cross-sample comparison within a dataset.
- Filter lowly expressed genes: keep genes with CPM > 1 in at least N samples (N = smallest group size).

### 4b. RNA-seq QC Additions

| Metric | Threshold |
|-------|-------|
| % uniquely mapped | > 70% |
| % rRNA | < 10% |
| % exonic (human) | > 60% |
| 3' bias (human) | < 2 (median 3'/5' coverage ratio) |
| Number of detected genes (CPM > 1) | > 10,000 for human |

## Quality Gate

Sequencing data is ready when:
- All FASTQ files pass FastQC with no critical failures.
- Alignment metrics are within thresholds for the assay type.
- Coverage meets application-specific requirements.
- VCF passes filtering (hard filters or VQSR).
- Batch metadata (date, instrument, reagent lot, operator) is recorded.
- Reference genome version is locked and documented.
- Contamination and sample-swap checks are clean.
