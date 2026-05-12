---
name: medical-bio-data-task
description: Meta-guideline for curating biomedical and clinical datasets — genomics, single-cell, immunology, drug discovery, proteomics, epigenomics, pathology, clinical longitudinal, medical imaging, and EHR. Covers domain-specific quality control, regulatory compliance (HIPAA/GDPR/IRB), and reproducibility standards.
recommended_guidelines:
  - medical-dataset-sources-task
  - genomics-sequencing-data-task
  - single-cell-data-task
  - immunology-data-task
  - clinical-drug-discovery-data-task
  - epigenomics-data-task
  - proteomics-metabolomics-data-task
  - pathology-data-task
  - clinical-longitudinal-data-task
  - medical-imaging-task
  - ehr-integration-task
  - clinical-nlp-task
  - training-data-design-principles
  - dataset-creation-curation-task
---

## Overview

Biomedical data curation has stakes that general ML datasets don't: patient safety, regulatory compliance, and clinical validity. Every decision must be auditable, every exclusion must be justified, and every dataset must survive adversarial regulatory review. This guideline layers medical-domain requirements on top of the universal `training-data-design-principles`.

## Universal Biomedical Principles

### 1. Regulatory Alignment From Day One

Before collecting or processing a single sample, identify which regulations apply:

| Regulation | Scope | Key Requirement |
|-------|-------|-------|
| **HIPAA** (US) | Protected Health Information (PHI) | De-identification, BAAs, minimum necessary |
| **GDPR** (EU) | Personal data of EU residents | Consent, right to erasure, data minimization |
| **IRB** | Human subjects research | Protocol approval, informed consent, continuing review |
| **FDA 21 CFR Part 11** | Electronic records in clinical trials | Audit trails, electronic signatures, validation |
| **GxP** (GLP/GCP/GMP) | Lab/clinical/manufacturing practice | SOPs, training records, validated systems |
| **ICH E6 (GCP)** | Clinical trial conduct | Data integrity, source document verification |

### 2. Consent and Re-Identification Risk

- Every dataset must trace back to a specific consent form that explicitly allows the intended use.
- De-identification is not a binary state — it's a risk spectrum. Run re-identification attacks on your "de-identified" data before assuming it's safe.
- For genomics data, be aware that DNA sequence is inherently identifying. "De-identified genomic data" is an oxymoron.
- Document the consent scope: "This data may be used for X but not for Y."

### 3. Batch Effects Are the #1 Silent Killer

Biomedical data is produced by instruments, not extracted from the web. Instruments drift. Reagents change. Protocols evolve.

- Record batch metadata: date, instrument ID, reagent lot, operator, protocol version.
- Visualize batch effects before accepting any dataset (PCA colored by batch, not by biology).
- Apply batch correction only when necessary and document the method (Harmony, ComBat, scVI, etc.).
- Never apply batch correction to test/holdout data using parameters learned from training data. Split first, correct after.

### 4. Reference Genome and Annotation Versioning

For any sequencing-based data:
- Lock the reference genome assembly (e.g., GRCh38.p14, not "latest").
- Lock the gene annotation version (e.g., GENCODE v44, Ensembl 110).
- Changing references changes coordinates, gene models, and variant interpretations.
- Document the exact reference used and why.

### 5. Clinical Metadata Standards

| Data Type | Metadata Standard |
|-------|-------|
| Sequencing | MIxS (Minimum Information about any (x) Sequence) |
| Microarray | MIAME (Minimum Information About a Microarray Experiment) |
| Single-cell | Minimum Information About a Single-Cell Experiment (in development) |
| Clinical trials | CDISC (SDTM, ADaM) |
| Imaging | DICOM headers |
| EHR/OMOP | OMOP CDM (Observational Medical Outcomes Partnership Common Data Model) |

### 6. Phenotype and Label Quality

Clinical labels are not ground truth — they're approximations with error rates:
- ICD codes are billing codes, not clinical diagnoses. They reflect what was billed, not what the patient had.
- Lab values have reference ranges that vary by instrument, population, and time.
- Expert annotation has inter-rater variability. Measure it (Cohen's kappa, Fleiss' kappa) and report it.
- When using LLM-assisted label extraction, validate against a human-annotated gold standard.

---

## Domain-Specific Guidelines

| Domain | Guideline | Key Data Types |
|-------|-----------|-------|
| **Genomics / Sequencing** | `genomics-sequencing-data-task` | FASTQ, BAM, VCF, coverage metrics, variant QC |
| **Single-Cell** | `single-cell-data-task` | scRNA-seq, ATAC-seq, spatial, doublet detection |
| **Immunology** | `immunology-data-task` | TCR/BCR repertoires, flow cytometry, cytokine panels |
| **Drug Discovery** | `clinical-drug-discovery-data-task` | Compound libraries, HTS assays, ADMET, clinical trials |
| **Proteomics / Metabolomics** | `proteomics-metabolomics-data-task` | MS spectra, peptide ID, protein quant, PTM analysis |
| **Protein Datasets** | `protein-dataset-sources-task` | PDB, AlphaFold DB, ESM Atlas, STRING, PDBbind, ProteinGym |
| **Epigenomics** | `epigenomics-data-task` | ChIP-seq, ATAC-seq, methylation, Hi-C |
| **Pathology (WSI)** | `pathology-data-task` | Whole slide images, IHC, stain normalization |
| **Clinical Longitudinal** | `clinical-longitudinal-data-task` | Lab values, survival data, EHR phenotypes |
| **Medical Imaging** | `medical-imaging-task` | DICOM, NIfTI, annotation QC, augmentation |
| **EHR / Clinical Data** | `ehr-integration-task` | OMOP CDM, FHIR, phenotyping, temporal alignment |
| **Clinical NLP** | `clinical-nlp-task` | De-identification, entity extraction, negation detection |

---

## Quality Gate (Biomedical)

A biomedical dataset is ready when:
- Regulatory requirements (HIPAA/GDPR/IRB) are documented and satisfied.
- Consent scope matches intended use.
- Batch metadata is recorded and batch effects are visualized.
- Reference genome and annotation versions are locked and documented.
- Clinical metadata follows domain standards (MIxS, CDISC, OMOP).
- Label quality is measured (inter-rater agreement or validated against gold standard).
- Re-identification risk is assessed and documented.
- Data provenance traces from instrument/sample to final file.
