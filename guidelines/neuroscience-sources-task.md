---
name: neuroscience-sources-task
description: Find neuroscience datasets — electrophysiology (Neuropixels, Allen Brain Observatory), calcium imaging, fMRI (OpenNeuro, HCP), connectomics (MICrONS, FlyEM), and EEG.
recommended_skills:
  - neurokit2
  - nibabel
  - database-lookup
recommended_guidelines:
  - medical-bio-data-task
  - scientific-database-lookup-task
---

## Overview

Neuroscience data spans scales from ion channels to whole-brain imaging. Key modalities: electrophysiology (spike trains, LFP), calcium/voltage imaging, fMRI, EEG/MEG, and connectomics (wiring diagrams).

## Electrophysiology

| Resource | Description | Size | Access |
|------|-------|-------|-------|
| **Allen Brain Observatory** | Multi-area Neuropixels recordings, visual stimulation | 100K+ neurons, mouse | Allen SDK |
| **IBL** (International Brain Lab) | Standardized decision-making task, Neuropixels | 500K+ neurons, mouse | DataJoint / ONE API |
| **DANDI** | Distributed Archives for Neurophysiology Data Integration | 200+ datasets | DANDI API, NWB format |
| **CRCNS** | Collaborative Research in Computational Neuroscience | 200+ curated datasets | Web |
| **NeuroData Without Borders (NWB)** | Standardized format for neurophysiology | Cross-repo | `pynwb` |
| **Brain Initiative Cell Census Network (BICCN)** | Cell-type resolved transcriptomics + electrophysiology | Mouse/human | Download + API |
| **SpikeForest** | Spike sorting benchmarking | 500+ recordings | Web |

## Calcium and Optical Imaging

| Resource | Description |
|------|-------|
| **Allen Brain Observatory (2-photon)** | Visual cortex, 60K+ neurons |
| **MICrONS** | 1 mm³ of mouse visual cortex, EM + functional |
| **BossDB** | Volumetric EM/light microscopy |
| **Neurofinder** | Cell detection in calcium imaging benchmark |

## fMRI / MRI

| Resource | Description | Size | Access |
|------|-------|-------|-------|
| **Human Connectome Project (HCP)** | High-resolution fMRI, DTI, MEG | 1,200 subjects | Open access |
| **OpenNeuro** | Open fMRI/MEG/EEG datasets | 1,000+ datasets | Open, BIDS format |
| **UK Biobank Imaging** | Brain MRI for 50K+ participants | 50K subjects | Application |
| **ADNI** (Alzheimer's Disease Neuroimaging Initiative) | Longitudinal MRI, PET, biomarkers | 2K+ subjects | Application |
| **OASIS** | Brain MRI, Alzheimer's focus | 1K+ subjects | Open |
| **ABCD Study** | Adolescent brain development, 10K+ subjects | 11K+ subjects | Application |
| **fMRIprep outputs (OpenNeuro)** | Preprocessed fMRI ready for ML | 1,000+ datasets | Open |

## EEG / MEG

| Resource | Description | Size |
|------|-------|-------|
| **TUH EEG** (Temple University Hospital) | Clinical EEG corpus | 30K+ recordings |
| **PhysioNet EEG** | EEG, ECG, sleep studies | Diverse |
| **CHB-MIT** | Scalp EEG, pediatric epilepsy | 24 cases |
| **BNCI Horizon 2020** | BCI datasets | Multiple |
| **MOABB** (Mother of All BCI Benchmarks) | Standardized BCI evaluation | Pipeline |

## Connectomics

| Resource | Description | Scale |
|------|-------|-------|
| **MICrONS** | EM reconstruction, mouse V1 | 200K cells, 500M synapses |
| **FlyEM (Hemibrain)** | Drosophila hemibrain connectome | 25K neurons, 20M synapses |
| **FlyWire** | Full adult Drosophila brain | 140K neurons, 50M synapses |
| **H01** | 1 mm³ human temporal cortex EM | 50K cells |
| **Catmaid** | Collaborative connectomics | Multiple species |

## Multi-Species and Comparative

| Resource | Species |
|------|-------|
| **Marmoset Brain Mapping** | Marmoset |
| **Zebrafish Brain Browser (ZBB)** | Zebrafish |
| **WormAtlas / OpenWorm** | C. elegans — only complete connectome |

## Quick Access

```python
# NWB files via pynwb
from pynwb import NWBHDF5IO
io = NWBHDF5IO("dataset.nwb", "r")
nwb = io.read()

# OpenNeuro via openneuro-py
# pip install openneuro-py
import openneuro
ds = openneuro.download(dataset="ds001246", target_dir="./")
```
