---
name: dataset-release-checklist
description: Aggregated launch checklist for dataset releases — combines quality gates from all curation guidelines into a single sign-off document.
recommended_guidelines:
  - dataset-creation-curation-task
  - training-data-design-principles
  - dataset-governance-task
  - annotation-management-task
---

## Overview

Before shipping a dataset, every gate must pass. This checklist aggregates quality gates from every curation guideline into a single launch document. No dataset should be released with unchecked items.

## Release Checklist

### Specification & Design
- [ ] Purpose and intended use documented.
- [ ] Explicit non-use cases listed.
- [ ] Schema defined with types, constraints, allowed values.
- [ ] Target metrics for quality, volume, diversity defined.

### Provenance
- [ ] Source of every data component documented.
- [ ] Collection dates and methods recorded.
- [ ] Processing pipeline version recorded.
- [ ] All transformations documented with rationale.

### Cleaning & Curation
- [ ] Missing value strategy documented and applied.
- [ ] Deduplication applied (exact + near + cross-dataset).
- [ ] Outliers handled per domain knowledge.
- [ ] Cleaning audit log saved with dataset.

### Splitting
- [ ] Train/val/test splits created, seed fixed.
- [ ] Stratification verified.
- [ ] No leakage across splits (entity, temporal, group).
- [ ] Split indices saved and versioned.

### Quality & Validation
- [ ] Schema conformance verified.
- [ ] Distribution comparison across splits.
- [ ] Label quality audited (confident learning or IAA).
- [ ] Bias audit completed for protected attributes.
- [ ] Benchmark contamination scan clean.

### Documentation
- [ ] Data card completed (all sections).
- [ ] Limitations section honest and specific.
- [ ] License verified and included.
- [ ] Citation format provided.

### Governance & Legal
- [ ] License compatible with all source data.
- [ ] Consent basis documented.
- [ ] De-identification validated (re-ID attack).
- [ ] EU AI Act risk category assessed.
- [ ] DPAs signed with data processors.

### Versioning & Release
- [ ] Semantic version assigned.
- [ ] Manifest with checksums created.
- [ ] Release tag pushed.
- [ ] Changelog / diff from previous version.
- [ ] Access instructions documented.

### Pipeline Health
- [ ] Schema drift monitoring active.
- [ ] Distribution drift monitoring active.
- [ ] Freshness SLA defined.
- [ ] Backfill protocol documented.

## Sign-Off

| Role | Name | Date | Signature |
|------|-------|-------|-------|
| Data Owner | | | |
| Technical Reviewer | | | |
| Legal/Compliance | | | |
| Domain Expert | | | |

## Post-Release

- [ ] Release announced to stakeholders.
- [ ] Known issues / errata page created.
- [ ] Support contact published.
- [ ] Deprecation timeline defined (if applicable).
