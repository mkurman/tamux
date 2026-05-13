---
name: chemistry-materials-sources-task
description: Find chemistry and materials datasets — crystal structures (COD 500K, ICSD, Materials Project 150K), quantum properties (QM9 134K, ANI-1x 5M), and benchmarks (MoleculeNet, OC20, MatBench).
recommended_skills:
  - pymatgen
  - database-lookup
  - rdkit
  - deepchem
recommended_guidelines:
  - clinical-drug-discovery-data-task
  - scientific-database-lookup-task
---

## Overview

Beyond drug-like organic molecules lies the broader chemistry and materials landscape: crystal structures, quantum calculations, catalysts, polymers, and solid-state materials. Each has its own data ecosystem.

## Crystal Structures

| Resource | Description | Size | Access |
|------|-------|-------|-------|
| **COD** | Crystallography Open Database, small-molecule crystals | 500K+ | `bioservices` / REST |
| **ICSD** | Inorganic Crystal Structure Database | 280K+ | Subscription |
| **Materials Project** | DFT properties for known + novel materials | 150K+ materials | `pymatgen` + API key |
| **OQMD** | DFT thermodynamics | 1M+ entries | REST API |
| **AFLOW** | Automatic FLOW, 3.5M+ entries | 3.5M+ materials | REST API |
| **NOMAD** | Multi-code materials data | 100M+ calculations | REST API |
| **CSD** | Cambridge Structural Database | 1.2M+ organic/metal-organic | Subscription |

```python
# Materials Project
from pymatgen.ext.matproj import MPRester
with MPRester("API_KEY") as m:
    entries = m.query(criteria={"elements": {"$in": ["Li"]}},
                      properties=["material_id", "formula_pretty", "band_gap"])
```

## Quantum Chemistry

| Resource | Description | Size | Use |
|------|-------|-------|-------|
| **QM9** | DFT properties for small organic molecules | 134K | Quantum property prediction |
| **QM7 / QM7b** | DFT for 7K molecules | 7K | Fast benchmarks |
| **ANI-1x / ANI-1ccx** | DFT energies/forces for organics | 5M+ conformations | Neural network potentials |
| **MD17** | Ab-initio MD trajectories | 150K frames | Force field learning |
| **PCQM4Mv2** | DFT HOMO-LUMO gap (OGB-LSC) | 3.7M molecules | Graph regression benchmark |
| **ISO17** | Conformational dynamics | 500K+ frames | Conformational ML |
| **PubChemQC** | DFT for PubChem molecules | 3.8M | Large-scale quantum ML |

## Polymers and Soft Matter

| Resource | Contents |
|------|-------|
| **PolyInfo** | 19K+ homopolymers, 500K+ data points |
| **PI1M** | 1M polymer MD simulations |
| **NIST Synthetic Polymer Library** | Polymer mass spectra |

## Benchmarks

| Benchmark | Task | Size |
|------|-------|-------|
| **MoleculeNet** | Multi-task molecular benchmarks | 17 datasets, 700K+ compounds |
| **Open Catalyst (OC20/OC22)** | Catalyst screening | 1.3M DFT relaxations |
| **MatBench** | Materials property prediction | 13 tasks |
| **TDC** | 80+ drug benchmarks | Via `pytdc` |
| **JARVIS-DFT** | DFT + ML benchmarks | 80K+ materials |
