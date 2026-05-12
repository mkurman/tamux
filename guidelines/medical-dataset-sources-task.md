---
name: medical-dataset-sources-task
description: Find and access medical/biological datasets for ML training — genomics, imaging, EHR, pathology, drug discovery, single-cell, and more. Catalogs major public repositories with access requirements, formats, and use cases.
recommended_skills:
  - database-lookup
  - cellxgene-census
  - depmap
  - imaging-data-commons
  - gget
  - bioservices
  - clinical-trials
  - mimic
  - fhir
  - omop-ohdsi
  - tiledbvcf
  - hf-datasets
recommended_guidelines:
  - medical-bio-data-task
  - scientific-database-lookup-task
  - research-task
---

## Overview

You don't need to collect raw clinical data to start training. Thousands of high-quality, de-identified, IRB-approved medical datasets are publicly available. This guideline catalogs the major repositories organized by domain, with access requirements, formats, and what they're best used for.

---

## Part 1: Electronic Health Records (EHR)

| Dataset | Description | Size | Access |
|------|-------|-------|-------|
| **MIMIC-IV** | ICU admissions, vitals, labs, meds, notes — the most-cited clinical ML dataset | 40K+ patients, 2008-2019 | Free, credentialed (PhysioNet) |
| **MIMIC-III** | Predecessor; still heavily cited for clinical NLP | 38K+ adults | Free, credentialed |
| **eICU-CRD** | Multi-center ICU data across US hospitals | 200K+ admissions | Free, credentialed |
| **EHRSHOT** (Stanford) | Longitudinal EHR for foundation model benchmarking | 6,739 patients, 41M events | Free, academic |
| **UK Biobank** | 500K participants, genotypes, imaging, EHR linkage | 500K participants | Application + fee |
| **All of Us** (NIH) | Diverse US cohort, EHR + genomics + wearables | 400K+ participants | Free, credentialed |
| **N3C** (NCATS) | COVID-19 EHR data across 70+ US sites | 8M+ patients | Free, institutional |

**Best first choice**: MIMIC-IV for clinical ML. It has the most community tooling, tutorials, and benchmark results.

---

## Part 2: Medical Imaging

### 2a. Radiology

| Dataset | Modality | Size | Annotations |
|------|-------|-------|-------|
| **MIMIC-CXR-JPG** | Chest X-ray | 377K+ images | 14 chest findings, radiology reports |
| **CheXpert** (Stanford) | Chest X-ray | 224K images | 14 findings, uncertainty labels |
| **CheXpert Plus** (Stanford, 2025) | Chest X-ray | Extended CheXpert with reports | Expanded annotations |
| **NIH Chest X-ray** | Chest X-ray | 112K images | 14 disease labels |
| **RSNA Pneumonia** | Chest X-ray | 30K images | Pneumonia opacity boxes |
| **VinDr-CXR** | Chest X-ray | 18K images | 22 local + 6 global findings, Vietnamese |
| **SIIM-ACR Pneumothorax** | Chest X-ray | 12K images | Pneumothorax segmentations |
| **CT-RATE** | Chest CT | 25K CT volumes | 18 radiology report labels |

### 2b. Oncology Imaging

| Dataset | Source | Contents |
|------|-------|-------|
| **TCIA** (The Cancer Imaging Archive) | NCI | 100+ collections: CT, MRI, PET, DICOM + segmentations |
| **LUNA16 / LUNA25** | Lung CT | Nodule detection; 888 CTs (LUNA16), expanded in 2025 |
| **NLST** | Lung CT | 75K participants, lung cancer screening, survival |
| **TCGA Imaging** | Multi-cancer | Linked to genomics for radiogenomics |
| **BraTS** | Brain MRI | Glioblastoma segmentation, multi-modal MRI |
| **LIDC-IDRI** | Lung CT | 1,018 CTs with 4-reader annotations |
| **OASIS** | Brain MRI | Alzheimer's, longitudinal, 1K+ subjects |

### 2c. Pathology (Whole Slide Images)

| Dataset | Organ | Size | Use |
|------|-------|-------|-------|
| **TCGA Pathology** | Multi-organ | 10K+ WSI, 32 cancer types | Cancer classification, survival |
| **CAMELYON16** | Lymph node | 400 WSI | Metastasis detection |
| **CAMELYON17** | Lymph node | 1K WSI, 5 centers | Domain generalization |
| **PatchCamelyon (PCam)** | Lymph node | 327K patches (96×96) | Binary classification benchmark |
| **PANDA** | Prostate | 10K+ WSI | Gleason grading |
| **BACH** | Breast | 400 images | Microscope classification |

### 2d. Other Modalities

| Dataset | Modality | Use |
|------|-------|-------|
| **STARE / DRIVE** | Retinal fundus | Vessel segmentation |
| **APTOS 2019** | Retinal fundus | Diabetic retinopathy grading |
| **ISIC Archive** | Dermoscopy | Skin lesion classification, > 50K images |
| **MedPix** | Multimodal | 59K rad/path/derm cases with metadata |

**Best first choice**: MIMIC-CXR + CheXpert for chest X-ray. TCIA for oncology imaging. PatchCamelyon for fastest pathology prototyping.

---

## Part 3: Genomics and Sequencing

### 3a. Cancer Genomics

| Resource | Contents | Access Method |
|------|-------|-------|
| **TCGA** (The Cancer Genome Atlas) | 20K+ tumors, 33 cancer types: WGS, WES, RNA-seq, methylation, clinical | GDC Portal / `gget` |
| **ICGC** (International Cancer Genome Consortium) | 25K+ tumors, 50+ cancer types, international | ICGC Data Portal |
| **DepMap** (Broad) | 1,800+ cancer cell lines: CRISPR/RNAi dependency, genomics, drug sensitivity | `depmap` skill / DepMap Portal |
| **CCLE** (Cancer Cell Line Encyclopedia) | 1,400+ cell lines: expression, mutations, drug response | DepMap / CCLE Portal |
| **MSK-IMPACT** | 40K+ clinical tumor sequences, outcomes, treatment | cBioPortal |
| **AACR GENIE** | 150K+ tumor samples across 19 centers | cBioPortal / synapse.org |
| **TARGET** | Pediatric cancers: genomics + clinical | GDC Portal |
| **PCAWG** (Pan-Cancer Analysis) | 2,800 tumors, whole genomes, integrated analysis | ICGC |

### 3b. Population Genomics

| Resource | Contents | Access |
|------|-------|-------|
| **gnomAD** (Broad) | 800K+ individuals, WGS/WES, population allele frequencies | `gget` / web |
| **1000 Genomes** | 2,500 individuals, 26 populations, WGS | IGSR / `tiledbvcf` |
| **UK Biobank** | 500K WGS + genotype array, phenotype linkage | Application-based |
| **All of Us** | Diverse US cohort, WGS + array | Free, credentialed |
| **TOPMed** | 180K+ WGS, heart/lung/blood focus | dbGaP |
| **HPRC** (Human Pangenome) | 47 phased diploid assemblies, diverse populations | HPRC website |
| **ClinVar** | Clinically annotated variants, pathogenicity | NCBI / `bioservices` |

### 3c. Functional Genomics

| Resource | Contents |
|------|-------|
| **ENCODE** | TF binding, histone marks, chromatin state across tissues |
| **Roadmap Epigenomics** | 127 reference epigenomes, histone + methylation |
| **GTEx** | 54 tissues × 948 donors, expression QTLs |
| **FANTOM5** | CAGE expression atlas across cell types |

**Best first choice**: TCGA for cancer biology. gnomAD for variant frequency. DepMap for drug sensitivity + genomics.

---

## Part 4: Single-Cell

| Resource | Contents | Access |
|------|-------|-------|
| **CZ CELLxGENE** (Chan Zuckerberg) | 90M+ cells, 1,400+ datasets, standardized AnnData | `cellxgene-census` API / web |
| **Human Cell Atlas** | Reference atlas of all human cell types | CELLxGENE + HCA Data Portal |
| **HuBMAP** | Spatial + single-cell, human tissue maps | HuBMAP Portal |
| **Single Cell Portal** (Broad) | 600+ studies, diverse tissues/species | Web / API |
| **Tabula Sapiens / Muris** | Cell atlases for human (500K cells) and mouse (100K cells) | CELLxGENE |
| **JingleBells** | Standardized scRNA-seq, immune focus | Web |
| **Tumor Immune Single-Cell Hub (TISCH)** | 2M+ cells, 190 datasets, tumor microenvironment | Web |

**Best first choice**: `cellxgene-census` for programmatic access via Python. CELLxGENE for browsing.

---

## Part 5: Drug Discovery

| Resource | Contents | Access |
|------|-------|-------|
| **ChEMBL** | 2.4M compounds, 20M bioactivities, 15K targets | `bioservices` / `deepchem` |
| **PubChem** | 115M compounds, bioassays | PubChem API |
| **DrugBank** | 500K+ drugs, targets, pathways | Free/academic license |
| **BindingDB** | 2.6M binding affinities, protein-ligand | Web / download |
| **PDBbind** | 23K protein-ligand structures with binding data | PDBbind website |
| **Therapeutic Data Commons (TDC)** | 80+ ML-ready benchmarks across ADMET, drug-target interaction | `pytdc` skill |
| **ZINC20 / ZINC22** | 1.4B purchasable compounds, 3D conformers | ZINC website |
| **Open Targets** | Target-disease association scores, genetic evidence | Open Targets API |
| **PrimeKG** | Precision medicine knowledge graph | `primekg` skill |
| **AlphaFold DB** | 200M+ predicted protein structures | `alphafold` skill |

---

## Part 6: Clinical Trials

| Resource | Contents | Access |
|------|-------|-------|
| **ClinicalTrials.gov** | 480K+ trials, arms, outcomes | `clinical-trials` skill / API |
| **WHO ICTRP** | International trial registry | Web |
| **EU Clinical Trials Register** | EU interventional trials | Web |
| **AACT** (Duke) | ClinicalTrials.gov in relational DB | Free download |
| **Vivli** | Individual patient data sharing | Application-based |

---

## Part 7: Multi-Modal and Specialized

| Dataset | Modalities | Use Case |
|------|-------|-------|
| **Symile-MIMIC** | CXR + ECG + labs (from MIMIC-IV) | Multimodal clinical ML |
| **CLIMB** | Large-scale multimodal clinical foundation models | Radiology + notes + labs |
| **MedTrinity-25M** | 25M multimodal medical samples | Multimodal pretraining |
| **PMC-OA** (PubMed Central) | 2.7M articles + figures | Biomedical text + image |
| **OpenI** | 8K chest X-rays + reports from Indiana | Image-to-text |
| **ROCO** | 81K radiology figures + captions | Radiology image captioning |

### Audio / Speech

| Dataset | Contents | Use Case |
|------|-------|-------|
| **PhysioNet / CinC** | ECG, EEG, PPG signals | Physiological signal analysis |
| **MIMIC Waveform** | Bedside monitor waveforms from ICU | Arrhythmia detection |
| **KAGGLE Heart Sound** | Phonocardiogram | Murmur classification |
| **Coswara** | Cough/speech sounds | COVID detection from audio |
| **Mozilla Common Voice** | 30K+ hours multilingual speech | Medical ASR fine-tuning |

---

## Part 8: How to Access — Fast Path

### By Access Tier

| Tier | Examples | How |
|-------|-------|-------|
| **Open / No Auth** | TCGA (via GDC), gnomAD, CELLxGENE, NCT, ChEMBL | Direct download or API |
| **Free + Credentialed** | MIMIC-IV, eICU, All of Us, UK Biobank | Complete training + sign DUA |
| **Application + Fee** | UK Biobank, some dbGaP studies | Submit research proposal + pay |
| **Simulated / Synthetic** | Synthea, MDClone, synthea-international | Generate locally |

### By Tool

```bash
# TCGA data via gget
gget ref --db tcga --organism human

# Single-cell via cellxgene-census
import cellxgene_census
census = cellxgene_census.open_soma()
adata = cellxgene_census.get_anndata(census, organism="homo_sapiens", tissue="lung")

# MIMIC via BigQuery (Google Cloud)
# SELECT * FROM `physionet-data.mimic_core.admissions`

# Clinical trials via clinical-trials skill
# Search for completed oncology phase II trials with results

# Drug data via DeepChem
import deepchem as dc
tasks, datasets, transformers = dc.molnet.load_tox21()

# HuggingFace medical datasets
from datasets import load_dataset
dataset = load_dataset("alkzar90/NIH-Chest-X-ray-dataset")
dataset = load_dataset("candle-asch/camelyon17")
```

---

## Quality Gate

A dataset source is valid for training when:
- **License** is verified and compatible with your use case (academic vs. commercial).
- **Access** is secured — credentialing complete, DUA signed if required.
- **De-identification** method is documented — know what was removed.
- **Population** is described — age, sex, ethnicity, geography represented.
- **Limitations** are documented — what this dataset CANNOT be used for.
