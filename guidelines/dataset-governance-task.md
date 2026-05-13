---
name: dataset-governance-task
description: Govern datasets through their lifecycle — legal compliance, licensing, consent, data processing agreements, EU AI Act obligations, right-to-erasure mechanics, and audit-ready documentation.
recommended_skills:
  - data-card-writer
  - dataset-versioning
recommended_guidelines:
  - medical-bio-data-task
  - training-data-design-principles
---

## Overview

Governance is not a checkbox. It's the system that answers: who can use this data, for what, under what conditions, with what recourse, and with what audit trail. This guideline covers the legal, ethical, and operational dimensions.

## Licensing

| License | Commercial Use | Attribution | Share-Alike | Best For |
|------|-------|-------|-------|-------|
| **CC-BY-4.0** | Yes | Required | No | Open research |
| **CC-BY-SA-4.0** | Yes | Required | Yes | Open, derivative must stay open |
| **CC-BY-NC-4.0** | No | Required | No | Research only |
| **CC0** | Yes | No | No | Maximum openness |
| **CDLA-Permissive** | Yes | No | No | ML data, no restrictions |
| **CDLA-Sharing** | Yes | No | Yes | ML data, copyleft |
| **ODbL** | Yes | Required | Yes | Database right copyleft |
| **Custom / Proprietary** | Case-by-case | Per agreement | Per agreement | Enterprise |

### License Compliance Checklist

- [ ] License file included in dataset release.
- [ ] All source data licenses compatible with the dataset license.
- [ ] Attribution requirements documented and met.
- [ ] Commercial/non-commercial use explicitly stated.
- [ ] Derivative use restrictions stated.

## Consent and GDPR

### Lawful Basis for Processing (GDPR Art. 6)

| Basis | When Applicable | Documentation Required |
|------|-------|-------|
| **Consent** | Opt-in, specific, informed, unambiguous | Consent form, timestamp |
| **Legitimate interest** | Balanced against individual rights | Legitimate Interest Assessment (LIA) |
| **Public task** | Research in public interest | Institutional mandate |
| **Contract** | Data necessary for service | Contract terms |

### Data Subject Rights

Your dataset pipeline must support:

| Right | Implementation |
|------|-------|
| **Right to erasure** (Art. 17) | Remove all data for a given subject ID from all versions |
| **Right to access** (Art. 15) | Export all data for a given subject in machine-readable format |
| **Right to rectification** (Art. 16) | Update incorrect data for a given subject |
| **Data portability** (Art. 20) | Export in structured, commonly used format |

### De-Identification

- **Pseudonymization**: replace identifiers with codes (key stored separately).
- **Anonymization**: irreversibly remove identifiers. Under GDPR, truly anonymized data is not personal data.
- **Re-identification risk assessment**: run attacks on your de-identified data. If a motivated adversary can re-identify, it's not anonymized.

## EU AI Act (Effective 2026)

| Risk Category | Examples | Data Obligations |
|------|-------|-------|
| **Unacceptable** | Social scoring, real-time biometric surveillance | Prohibited |
| **High-risk** | Medical devices, hiring, credit scoring, law enforcement | Risk management, data governance, transparency, human oversight, accuracy |
| **Limited** | Chatbots, emotion recognition | Transparency obligations |
| **Minimal** | Spam filters, video games | None beyond existing law |

For high-risk AI: your dataset must demonstrate fitness for purpose, bias mitigation, and representative coverage. The data card IS your evidence.

## Data Processing Agreements (DPA)

Every third-party data processor must have a signed DPA covering:

- Purpose limitation (data used ONLY for specified purpose).
- Data minimization (only data necessary for the task).
- Retention period (how long data is stored).
- Sub-processor chain (who else has access).
- Breach notification (within 72 hours).
- Deletion certification (data deleted after contract ends).

## Audit Trail

For every dataset release, maintain:

```markdown
# Governance Audit Record
Dataset: [name] v1.0.0
Release Date: [YYYY-MM-DD]

## Legal
- License: CC-BY-4.0
- Source data licenses: [list with compatibility analysis]
- Consent basis: [Consent / Legitimate interest / Public task]
- IRB protocol: [number or "exempt"]
- DPIA completed: [Yes/No, date]

## Privacy
- De-identification method: [method, tools used, re-id risk score]
- PII scan: [tools used, results]
- Data subject rights mechanism: [ticket system, email, API]
- Retention policy: [duration, deletion trigger]

## Compliance
- EU AI Act category: [unacceptable / high / limited / minimal]
- High-risk compliance evidence: [data card, bias audit, accuracy report]
- DPAs with processors: [list, signed dates]
```

## Quality Gate

- License file present and verified.
- Consent basis documented per data source.
- De-identification validated with re-identification attack.
- Data subject rights mechanism operational.
- EU AI Act risk category assessed.
- DPAs signed with all processors.
- Governance audit record complete.
