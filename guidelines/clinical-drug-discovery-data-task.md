---
name: clinical-drug-discovery-data-task
description: Curate datasets for clinical drug discovery — compound libraries, high-throughput screening assays, ADMET predictions, molecular docking, and clinical trial data. Covers cheminformatics QC, assay reproducibility, structure standardization, and regulatory-grade data management.
recommended_skills:
  - deepchem
  - pytdc
  - datamol
  - rdkit
  - medchem
  - torchdrug
  - diffdock
  - esm
  - primekg
  - depmap
  - clinical-trials
  - dataset-versioning
recommended_guidelines:
  - medical-bio-data-task
  - drug-discovery-task
  - clinical-research-task
  - training-data-design-principles
---

## Overview

Drug discovery datasets span chemistry, biology, and clinical medicine. A single dataset might combine molecular structures, in-vitro assay results, in-vivo pharmacokinetics, and clinical outcomes. Errors at any level cascade: a mislabeled compound wastes months of follow-up work. Curation here demands cheminformatics rigor, assay statistics, and clinical data integrity.

---

## Phase 1: Compound Library Curation

### 1a. Chemical Structure Standardization

Every compound entering a dataset must pass these checks:

| Issue | Detection | Action |
|-------|-------|-------|
| **Invalid valence** | RDKit `SanitizeMol` | Remove or fix if fixable |
| **Missing stereochemistry** | RDKit `FindPotentialStereo` | Flag — racemic mixtures ≠ pure enantiomers |
| **Counterions / salts** | RDKit `SaltRemover` | Strip to parent compound |
| **Tautomers** | RDKit `TautomerEnumerator` | Canonicalize to one form |
| **Duplicate structures** | InChI key collision | Merge records |
| **Metals / organometallics** | Contains transition metal | Flag — force fields may fail |
| **PAINS (Pan-Assay Interference)** | PAINS filters | Flag as frequent-hitter risk |

```python
from rdkit import Chem
from rdkit.Chem import SaltRemover, Descriptors, AllChem
from rdkit.Chem.FilterCatalog import FilterCatalog, FilterCatalogParams

def standardize_compound(smiles: str) -> dict:
    """Standardize and QC a single compound."""
    mol = Chem.MolFromSmiles(smiles)
    if mol is None:
        return {"status": "invalid_smiles", "smiles": smiles}
    
    # Strip salts
    remover = SaltRemover.SaltRemover()
    mol = remover.StripMol(mol, dontRemoveEverything=True)
    
    # Sanitize
    try:
        Chem.SanitizeMol(mol)
    except Exception as e:
        return {"status": f"sani_failed: {e}", "smiles": smiles}
    
    # PAINS check
    pains_params = FilterCatalogParams()
    pains_params.AddCatalog(FilterCatalogParams.FilterCatalogs.PAINS)
    catalog = FilterCatalog(pains_params)
    entry = catalog.GetFirstMatch(mol)
    is_pains = entry is not None
    
    # Properties
    mw = Descriptors.MolWt(mol)
    logp = Descriptors.MolLogP(mol)
    hbd = Descriptors.NumHDonors(mol)
    hba = Descriptors.NumHAcceptors(mol)
    rot_bonds = Descriptors.NumRotatableBonds(mol)
    tpsa = Descriptors.TPSA(mol)
    
    # Lipinski rule-of-five
    lipinski_violations = sum([
        mw > 500,
        logp > 5,
        hbd > 5,
        hba > 10
    ])
    
    return {
        "status": "ok",
        "smiles": Chem.MolToSmiles(mol, canonical=True),
        "inchi_key": Chem.MolToInchiKey(mol),
        "mw": mw, "logp": logp,
        "hbd": hbd, "hba": hba,
        "rot_bonds": rot_bonds, "tpsa": tpsa,
        "lipinski_violations": lipinski_violations,
        "is_pains": is_pains,
    }
```

### 1b. Compound Library-Level QA

- **Diversity**: Measure Tanimoto similarity distribution. If > 30% of pairs have similarity > 0.8, flag as redundant.
- **Physicochemical space coverage**: Plot MW vs. logP. Gaps may indicate unexplored chemistry.
- **Known drugs/controls**: Ensure positive and negative controls are present and correctly labeled.
- **Vendor metadata**: Source, catalog number, purity, batch ID for every compound.

---

## Phase 2: High-Throughput Screening (HTS) Data

### 2a. Assay Quality Metrics

| Metric | Meaning | Threshold |
|-------|-------|-------|
| **Z' factor** | Assay quality (dynamic range + variability) | > 0.5 = excellent; 0-0.5 = marginal; < 0 = unusable |
| **Signal-to-background (S/B)** | Ratio of positive to negative control means | > 3 |
| **Signal-to-noise (S/N)** | (mean_pos - mean_neg) / SD_neg | > 10 |
| **CV of controls** | Coefficient of variation | < 10% |
| **Edge effects** | Row/column positional bias | Flag if systematic |

```python
def z_prime(pos_controls, neg_controls):
    """Z' factor — the universal assay quality metric."""
    mu_p, sd_p = np.mean(pos_controls), np.std(pos_controls)
    mu_n, sd_n = np.mean(neg_controls), np.std(neg_controls)
    return 1 - (3 * (sd_p + sd_n)) / abs(mu_p - mu_n)
```

### 2b. Hit Calling

- **Percent inhibition / activation** relative to controls.
- **Multiple concentrations**: IC50/EC50 from dose-response curves. Single-concentration data is screening, not pharmacology.
- **Curve quality**: Hill slope, R² of fit. Flag curves with poor fit.
- **Re-test rate**: Primary hits should be re-tested. Report confirmation rate.

### 2c. Plate-Level Normalization

```python
# Per-plate normalization to controls
def normalize_plate(plate_data, pos_cols, neg_cols):
    plate_mean_pos = plate_data[pos_cols].mean()
    plate_mean_neg = plate_data[neg_cols].mean()
    return (plate_data - plate_mean_neg) / (plate_mean_pos - plate_mean_neg) * 100
```

---

## Phase 3: ADMET Data

### 3a. In-Vitro ADMET Assays

| Assay | Units | Key QC Check |
|-------|-------|-------|
| **Aqueous solubility** | μM or LogS | Check for precipitation at high concentrations |
| **LogD / LogP** | Unitless | pH must be reported (LogD is pH-dependent) |
| **Caco-2 / MDCK permeability** | 10⁻⁶ cm/s | Include reference compounds (high/low permeability) |
| **Microsomal stability** | t₁/₂ (min) or CLint (μL/min/mg) | Species must be reported (human vs. mouse) |
| **CYP inhibition** | IC50 (μM) | Report which isoform (e.g., CYP3A4) |
| **hERG** | IC50 (μM) | Cardiac safety — clinical gatekeeper |
| **Plasma protein binding** | % bound | Species-specific |
| **Ames mutagenicity** | Positive/negative (with S9 ±) | Frame shift vs. base-pair substitution |

### 3b. In-Vivo PK Data

- **Species, strain, sex, and route** are mandatory metadata.
- **Dose**: Report as mg/kg with formulation.
- **Key parameters**: Cmax, Tmax, AUC, t₁/₂, Vd, CL, F% (bioavailability).
- **Inter-animal variability**: Report SD/SE, not just mean.
- **Cassette dosing**: Flag — PK parameters from cassette dosing differ from discrete dosing.

---

## Phase 4: Molecular Docking and Structural Data

### 4a. Protein Structure QC

| Issue | Detection | Action |
|-------|-------|-------|
| Resolution > 3.0 Å | PDB header | Flag — low confidence in side-chain positions |
| Missing loops/residues | PDB REMARK 465 | Model missing regions or exclude from binding site |
| Alternate conformations | PDB ANISOU records | Choose dominant conformation |
| Crystallographic artifacts | Ligand clashes with protein | Energy-minimize complex |
| Water molecules in binding site | Structural waters | Keep if conserved; remove if not |

### 4b. Docking Quality

- **Redocking**: Can you reproduce the crystal pose? RMSD < 2.0 Å = success.
- **Enrichment**: Do known actives rank above decoys? Report ROC-AUC or BEDROC.
- **Pose clustering**: Multiple similar top poses → well-defined binding mode. Scattered poses → weak prediction.

---

## Phase 5: Clinical Trial Data

### 5a. Data Sources

| Source | What It Provides | Caveats |
|-------|-------|-------|
| **ClinicalTrials.gov** | Study design, arms, outcomes, results | Self-reported; may differ from publications |
| **FDA review documents** | Detailed clinical pharmacology, safety | Gold standard but limited availability |
| **ChEMBL / DrugBank** | Structured bioactivity data | Curated but may lag behind literature |
| **Open Targets** | Target-disease associations | Multi-source integration; check provenance |
| **TCGA / DepMap** | Genomics + drug sensitivity | Cell lines ≠ patients |
| **EHR-derived datasets** | Real-world evidence | Confounding by indication dominates |

### 5b. Clinical Outcome Data

| Issue | How to Handle |
|-------|-------|
| **Missing outcome data** | Report proportion missing. Use sensitivity analysis (best/worst case), not single imputation |
| **Competing risks** | Death precludes disease progression. Use competing-risks models, not Kaplan-Meier ignoring competing events |
| **Informative censoring** | Patients lost to follow-up may differ systematically from completers. Flag. |
| **Time-varying confounding** | Treatment switches common. Standard analysis assumes no switching. |
| **Surrogate endpoints** | PFS ≠ OS. Document when surrogates are used. |

### 5c. Adverse Event Data

- **Preferred Term (PT) and System Organ Class (SOC)** from MedDRA.
- **Grade**: CTCAE v5.0.
- **Attribution**: Related / possibly related / unrelated.
- **Serious AEs**: Death, life-threatening, hospitalization, disability, congenital anomaly.

---

## Phase 6: Knowledge Graph Integration

For multi-modal drug discovery, structured knowledge graphs connect compounds → targets → pathways → diseases:

- **PrimeKG**: Precision medicine knowledge graph.
- **Hetionet**: Drug repurposing knowledge graph.
- **Use `primekg` skill** for KG queries.

---

## Quality Gate

Drug discovery data is ready when:
- All compounds are standardized (stripped salts, canonical SMILES, InChI keys).
- PAINS and frequent-hitter filters are applied with flags.
- HTS plates pass Z' > 0.5, edge effects are checked.
- ADMET assay metadata includes species, pH, and reference compounds.
- Protein structures meet resolution threshold and missing regions are handled.
- Clinical trial data includes dropout rates and sensitivity analysis.
- Compound provenance (vendor, batch, purity) is recorded.
- All data is versioned with assay date, instrument, and operator.
