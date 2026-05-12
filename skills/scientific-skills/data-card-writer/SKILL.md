---
name: data-card-writer
description: Generate structured datasheets for datasets (Gebru et al. "Datasheets for Datasets" format) — purpose, composition, collection process, preprocessing, limitations, and licensing.
tags: [data-card, datasheet, documentation, dataset-curation, metadata, transparency]
---

# Data Card Writer

## Overview

Generates a structured datasheet following the "Datasheets for Datasets" framework (Gebru et al., 2021). Every dataset should ship with a data card that answers: who made this, what's in it, how was it collected, what are the limitations.

## Template

```markdown
# [Dataset Name] — Data Card v[Version]

## Motivation
- Purpose: [What task is this for?]
- Creator: [Who created it?]
- Funding: [Who funded it?]

## Composition
- Instances: [N] examples, [M] features
- Target: [What is being predicted?]
- Protected attributes: [Gender, race, age, etc. — or "not collected"]
- Missing data: [X% missing overall, per-column breakdown]
- Class balance: [Distribution]

## Collection Process
- Source: [URL, database, API, instrument]
- Collection date: [YYYY-MM-DD to YYYY-MM-DD]
- Sampling strategy: [Random, stratified, convenience]
- Ethical review: [IRB protocol # or "not applicable"]
- Consent: [How was consent obtained?]

## Preprocessing
- Raw → cleaned pipeline: [Steps applied]
- Exclusions: [What was removed and why?]
- Transformations: [Normalization, imputation, encoding]
- Cleaning script hash: [sha256]

## Uses
- Recommended uses: [What this dataset is validated for]
- Discouraged uses: [What this should NOT be used for]
- Out-of-scope: [Inappropriate applications]

## Distribution
- License: [e.g., CC-BY-4.0, CDLA-Permissive]
- Access: [URL or process]
- Version: [Semantic version]

## Limitations
- Known biases: [Demographic, temporal, geographic]
- Coverage gaps: [Missing populations, conditions, domains]
- Label quality: [Inter-rater agreement, noise estimates]

## Maintenance
- Maintainer: [Contact or organization]
- Update frequency: [Monthly, annually, none]
- Errata: [Link to corrections]

## Citation
[BibTeX or DOI]
```

## Validation Checklist

- [ ] All sections populated (none left as placeholder).
- [ ] Limitations section is honest — not "no known limitations".
- [ ] License is verified (not guessed).
- [ ] Provenance traces to source data.
- [ ] Contact information is current.
- [ ] Version matches the dataset release tag.
