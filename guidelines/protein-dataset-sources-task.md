---
name: protein-dataset-sources-task
description: Find protein datasets for ML training — structures (PDB, AlphaFold DB, ESM Atlas), sequences (UniProt, Pfam), interactions (STRING, BioGRID, IntAct), binding (PDBbind, BindingDB), function (GO, EC), PTMs (PhosphoSitePlus), dynamics (ATLAS, MoDEL), and design benchmarks (Mega-scale, ProTherm, FireProtDB).
recommended_skills:
  - esm
  - alphafold
  - torchdrug
  - diffdock
  - bioservices
  - biopython
  - gget
  - database-lookup
  - hf-datasets
recommended_guidelines:
  - medical-dataset-sources-task
  - proteomics-metabolomics-data-task
  - clinical-drug-discovery-data-task
  - scientific-database-lookup-task
---

## Overview

Proteins are the molecular machines of life — and the richest data domain in biology. We have billions of sequences, hundreds of millions of predicted structures, tens of millions of experimentally determined interactions, and curated function annotations spanning decades. This guideline catalogs the major protein data resources by what you can train on.

---

## Part 1: Protein Structures

### 1a. Experimental Structures (PDB)

| Resource | Contents | Size | Format |
|------|-------|-------|-------|
| **PDB** (Protein Data Bank) | Experimentally determined (X-ray, cryo-EM, NMR) | 225K+ structures | `.pdb`, `.cif`, `.mmCIF` |
| **PDBe / RCSB** | Same data, different portals | Same as PDB | REST API, GraphQL |
| **PDB-REDO** | Re-refined, optimized PDB structures | 200K+ | `.cif` |

```bash
# Download PDB structures
curl -O https://files.rcsb.org/download/1ake.pdb

# Batch download via API
# All human proteins, resolution < 2.0Å
# https://search.rcsb.org/rcsbsearch/v2/query

# Via gget
gget pdb --id 1ake --out 1ake.pdb
```

### 1b. Predicted Structures

| Resource | Contents | Size | Access |
|------|-------|-------|-------|
| **AlphaFold DB** (DeepMind/EMBL-EBI) | AF2 predicted structures | 200M+ structures (v6, 2025) | Bulk download, API, `alphafold` skill |
| **AlphaFold3 predictions** | AF3 with ligands/ions/modifications | Growing | AlphaFold Server |
| **ESM Atlas** (Meta) | ESMFold predicted metagenomic structures | 772M+ structures | `esm` skill, esmatlas.com |
| **ESM3 generated** | Evolutionary-scale model, structure + function | Embeddings + structures | `esm` skill |
| **UniProt + AlphaFold** | All UniProt sequences mapped to AF structures | 245M+ entries | UniProt API, `gget` |

```python
# AlphaFold via gget
import gget
gget.alphafold(uniprot_id="P04637")  # p53

# Via ESM embeddings (pre-computed)
from esm import pretrained
model, alphabet = pretrained.esm2_t33_650M_UR50D()
# Use model to embed any protein sequence
```

### 1c. Structure Quality Metrics

When using predicted structures for training, always check:
- **pLDDT** (AF2/AF3): per-residue confidence, 0-100. > 70 = backbone reliable; > 90 = side-chain reliable.
- **pTM / ipTM**: global fold confidence. > 0.8 = high confidence.
- **PAE** (Predicted Aligned Error): inter-residue distance uncertainty. Low PAE between domains = domain orientation confident.
- **ESMFold quality**: pLDDT equivalent from ESM.

---

## Part 2: Protein Sequences

| Resource | Contents | Size | Access |
|------|-------|-------|-------|
| **UniProtKB/Swiss-Prot** | Manually curated, reviewed | 570K+ entries | `bioservices`, REST API |
| **UniProtKB/TrEMBL** | Automatically annotated | 250M+ entries | `bioservices`, REST API |
| **UniRef** | Clustered at 50/90/100% identity | 50%/90%/100% clusters | Download |
| **Pfam** (now InterPro) | Protein families, HMM profiles | 19K families | InterPro API |
| **InterPro** | Integrated protein domain/family DB | 40K+ entries | API |
| **NCBI Protein / RefSeq** | GenBank-derived; curated reference | 500M+ sequences | NCBI E-utilities |
| **eggNOG** | Orthologous groups, functional annotation | 12K+ organisms | API / download |

```python
# UniProt via bioservices
from bioservices import UniProt
u = UniProt()
entry = u.search("P04637").split()  # search p53
fasta = u.retrieve("P04637", frmt="fasta")

# InterPro scan via gget
gget.search("kinase", db="interpro")
```

---

## Part 3: Protein-Protein Interactions (PPI)

| Resource | Contents | Links | Access |
|------|-------|-------|-------|
| **STRING** | Known + predicted PPIs, 12K species | 59M proteins, 20B+ interactions | `string-db.org`, API, bulk download |
| **BioGRID** | Curated physical + genetic interactions | 2.3M+ interactions, 80+ species | `bioservices`, download |
| **IntAct** | Molecular interaction database (IMEx) | 1.5M+ curated interactions | `bioservices`, PSICQUIC |
| **DIP** | Database of Interacting Proteins | 80K interactions | Download |
| **MINT** | Molecular interactions | 130K interactions | Download |
| **HuRI** (Human Reference Interactome) | Systematic yeast two-hybrid, human | 64K binary PPIs | Download |
| **CORUM** | Mammalian protein complexes | 5K complexes | Download |
| **Predictomes** | Classifier-curated AF2 PPI predictions | 1.7M predicted PPIs (2025) | PMC |
| **HIPPIE** | Human PPI with confidence scores | 500K interactions | Download |

```python
# STRING via API
# Get interaction partners for TP53 (human)
curl "https://string-db.org/api/json/network?identifiers=9606.ENSP00000269305"

# BioGRID via bioservices
from bioservices import BioGRID
b = BioGRID()
interactions = b.get_interaction("TP53", "Homo sapiens")
```

**Best first choice**: STRING for coverage + scores. BioGRID for curation quality. HuRI for human binary interactions.

---

## Part 4: Protein-Ligand Binding

| Resource | Contents | Size | Use |
|------|-------|-------|-------|
| **PDBbind** | PDB structures with binding affinity | 23K complexes, Kd/Ki/IC50 | Binding affinity prediction, docking validation |
| **Leak-Proof PDBbind (LP-PDBbind)** | PDBbind with data leakage removed (2024) | 10K+ leak-free complexes | Generalizable affinity models |
| **BindingDB** | Binding affinities from literature | 2.6M affinities | Binding prediction, target profiling |
| **ChEMBL** | Bioactivity data, drug targets | 20M bioactivities | `bioservices`, `deepchem` |
| **Kinase Database (KLIFS)** | Kinase structures, inhibitors | 8K+ kinase structures | Kinase-specific drug design |
| **GPCRdb** | GPCR structures, ligands | 900+ GPCR structures | GPCR drug design |
| **PROTAC-DB** | PROTAC degraders | 4K+ PROTACs | Targeted protein degradation |
| **BioLip** | Biologically relevant ligand-protein interactions | 600K+ entries | Ligand binding site database |
| **CAMEO** | Continuous ligand blind evaluation | Weekly | Model benchmarking |

```python
# ChEMBL via bioservices
from bioservices import ChEMBL
c = ChEMBL()
target = c.get_target_by_name("ABL1")
activities = c.get_activities_by_target(target["target_chembl_id"])

# PDBbind
# Download from pdbbind.org.cn
# Filter: resolution < 2.5Å, Kd/Ki available
```

---

## Part 5: Protein Function and Annotation

| Resource | Contents | Use |
|------|-------|-------|
| **Gene Ontology (GO)** | Functional annotation: BP, CC, MF | Function prediction, enrichment |
| **UniProtKB keywords** | Controlled vocabulary function terms | Multi-label classification |
| **EC numbers** | Enzyme Commission classification | Enzyme function prediction |
| **CAZy** | Carbohydrate-Active enZymes | Glycobiology |
| **MEROPS** | Peptidase database | Protease classification |
| **TCDB** | Transporter Classification | Membrane transport |
| **KEGG** | Pathways, modules, orthologs | Pathway mapping |
| **Reactome** | Curated pathway database | Pathway enrichment |
| **DisProt** | Intrinsically disordered proteins | IDP prediction |
| **MobiDB** | Protein disorder + mobility | Disorder region prediction |

---

## Part 6: Post-Translational Modifications (PTMs)

| Resource | Contents | Size |
|------|-------|-------|
| **PhosphoSitePlus** | Phosphorylation, acetylation, ubiquitylation | 500K+ sites, human/mouse |
| **dbPTM** | Integrated PTM database | 2M+ PTM sites |
| **Phospho.ELM** | Phosphorylation sites | 50K+ sites |
| **GPS-SUMO** | SUMOylation sites | 60K+ sites |
| **UniProt PTM** | Curated modifications in Swiss-Prot | 100K+ curated modifications |
| **GlyConnect** | Glycosylation site database | Human N/O-linked |

---

## Part 7: Protein Dynamics

| Resource | Contents | Use |
|------|-------|-------|
| **MoDEL** | MD simulation trajectories, 1,500+ proteins | Dynamics analysis, ensemble methods |
| **ATLAS** (GPCRmd) | GPCR MD simulations | GPCR dynamics |
| **MemProtMD** | Membrane protein simulations | Membrane protein dynamics |
| **Dynameomics** | Fold-family MD simulations | Conformational sampling |
| **BioSimDB** | Simulation metadata database | Simulation discovery |
| **MDsrv** | Web-based MD trajectory visualization | Visual inspection |

---

## Part 8: Protein Design and Engineering

| Resource | Contents | Size | Use |
|------|-------|-------|-------|
| **Mega-scale** (Tsuboyama et al., 2023) | Experimental stability for 700K+ designed miniproteins | 700K+ sequences + stability | Stability prediction, inverse folding |
| **ProTherm / ProThermDB** | Thermodynamic parameters for WT and mutant proteins | 30K+ entries | Stability change prediction (ΔΔG) |
| **FireProtDB** | Experimentally validated stabilizing mutations | 15K+ mutations | Stability engineering |
| **SKEMPI / SKEMPI 2.0** | Binding affinity changes upon mutation | 7K mutations | Binding ΔΔG prediction |
| **AB-Bind** | Antibody-antigen binding ΔΔG | 1K+ mutations | Antibody engineering |
| **S669 / S2648** | Deep mutational scanning | Fitness landscapes | Mutational effect prediction |
| **ProteinGym** | 200+ DMS datasets, substitution scores | 200+ assays | Variant effect prediction benchmark |

```python
# ProteinGym via HuggingFace
from datasets import load_dataset
ds = load_dataset("oxford/proteingym", "substitution_scores")

# FireProtDB via REST
curl "https://loschmidt.chemi.muni.cz/fireprotdb/api/protein?uniprot_id=P04637"
```

---

## Part 9: Protein Embeddings (Pre-Computed)

| Resource | Model | Contents | Access |
|------|-------|-------|-------|
| **ESM2 embeddings** (Meta) | ESM-2 3B/650M | Per-residue + per-protein embeddings | `esm` skill |
| **ESM3 embeddings** | ESM-3 with structure/function | Structure-aware embeddings | `esm` skill |
| **ProtT5 / ProtBERT** | ProtTrans models | Per-residue embeddings | HuggingFace Hub |
| **AlphaFold2 representations** | AF2 internal representations | Structure + MSA embeddings | `alphafold` skill / ColabFold |
| **UniRep** | mLSTM-based | Universal protein representations | GitHub |
| **ProteinDT** | Text-aligned protein embeddings | Text-protein embeddings | GitHub |

```python
# ESM embeddings
from esm import pretrained

model, alphabet = pretrained.esm2_t33_650M_UR50D()
batch_converter = alphabet.get_batch_converter()

data = [("protein1", "MKTVRQERLKSIVRILERSKEPVSGAQLAEELSVSRQVIVQDIAYLRSLGYNIVATPRGYVLAGG")]
batch_labels, batch_strs, batch_tokens = batch_converter(data)

with torch.no_grad():
    results = model(batch_tokens, repr_layers=[33])
token_embeddings = results["representations"][33]  # per-residue
```

---

## Part 10: Benchmarks and Standardized Splits

| Benchmark | Task | Use |
|------|-------|-------|
| **ProteinGym** | 217 DMS assays, variant effect prediction | Mutational effect benchmarking |
| **CASP** (15+) | Blind structure prediction | Structure prediction evaluation |
| **CAFA** (5+) | Blind function prediction | Function annotation evaluation |
| **CAPRI** | Blind docking | Protein-protein docking evaluation |
| **AlphaFold2 benchmark set** | CASP14 + CAMEO targets | Structure prediction |
| **FLIP** (2024) | Fitness landscape prediction | Engineering benchmarks |
| **ATOM3D** | 3D molecular tasks, standardized splits | Structural biology ML tasks |
| **TAPE** | Tasks Assessing Protein Embeddings | Embedding evaluation |
| **PEER** | Protein sequence understanding | Comprehensive benchmark suite |
| **ProteinDT** | Text-to-protein generation | Multi-modal protein design |

---

## Quick Access by Use Case

| What You Want to Build | Start Here |
|--------|-------|
| **Protein structure prediction** | PDB (train) + CASP (evaluate) + AlphaFold DB (supplement) |
| **Variant effect prediction** | ProteinGym + DMS datasets (S669/S2648) + ClinVar |
| **Binding affinity prediction** | PDBbind + BindingDB + ChEMBL |
| **Protein design / engineering** | Mega-scale + ProThermDB + FireProtDB |
| **PPI network analysis** | STRING + BioGRID + HuRI |
| **Protein function prediction** | GO + UniProt + CAFA benchmarks |
| **Protein embedding / representation** | UniProt sequences + ProteinGym evaluation + ESM embeddings |
| **Antibody engineering** | SAbDab + AB-Bind + OAS (Observed Antibody Space) |
| **Enzyme design** | BRENDA + UniProt EC + CAZy |
| **Protein language model training** | UniRef50/90/100 + BFD + MGnify |

---

## Quality Gate

Protein data is ready for training when:
- **Version locked**: PDB date stamp, UniProt release, AlphaFold DB version (e.g., v6).
- **Redundancy removed**: sequences clustered at appropriate identity threshold (40% for structure, 50% for function).
- **Train/test leakage prevented**: proteins from the same superfamily stay in the same split; use time-split for PDB (train on pre-2020, test on post-2021).
- **Sequence-level splitting** (not residue-level): all residues of one protein in the same split.
- **Quality filtered**: X-ray resolution < 3.0Å, AF2 pLDDT > 70, no incomplete chains.
