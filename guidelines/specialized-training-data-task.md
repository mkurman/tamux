---
name: specialized-training-data-task
description: Curate datasets for contrastive learning, knowledge distillation, continual/federated learning, and anomaly detection. Covers positive pair augmentation strategies, teacher-student transfer sets, task sequencing for continual learning, non-IID partitioning for federated, and anomaly injection protocols.
recommended_skills:
  - embedding-analysis
  - hf-datasets
  - llm-assisted-curation
  - dataset-splitting
recommended_guidelines:
  - training-data-design-principles
  - dataset-creation-curation-task
  - rl-alignment-data-task
---

## Overview

Specialized training paradigms each demand unique data construction patterns. This guideline covers four: contrastive learning (how to build positives), knowledge distillation (how to generate teacher targets), continual learning (how to sequence tasks), federated learning (how to partition across clients), and anomaly detection (how to construct clean baselines).

---

## Part A: Contrastive Learning

### Positive Pair Strategies by Modality

| Modality | Strategy | Example |
|------|-------|-------|
| **Images** | Augmentation (SimCLR) | Crop + color jitter + flip → positive |
| **Text** | Back-translation, span deletion | "The cat sat" → "The feline was seated" |
| **Graphs** | Node/edge dropout, subgraph sampling | Drop 20% edges → positive |
| **Time series** | Time warping, jitter, cropping | Stretch 10% → positive |
| **Multimodal** | Co-occurrence (CLIP) | Image + caption from same page |

### Negative Strategies

| Strategy | Risk |
|------|-------|
| In-batch negatives | Can push apart similar-but-different items |
| Hard negative mining | Risk of false negatives (class collision) |
| Debiased contrastive (DCL) | Corrects for sampling bias in negatives |
| SupCon (Supervised Contrastive) | Uses labels to avoid false negatives |

### Quality Checks

- Augmentation doesn't change the semantic label (validate on 100 samples).
- No class collision in negative pairs (use labels if available).
- Temperature parameter tuned (0.07-0.5 range typical).

---

## Part B: Knowledge Distillation

### Teacher Data Construction

| Strategy | When | Quality |
|------|-------|-------|
| **Teacher predictions on unlabeled data** | Abundant unlabeled, expensive teacher | Soft labels with temperature |
| **Teacher on training set** | Compare student to teacher behavior | Measures fidelity |
| **Ensemble of teachers** | Reduce individual teacher bias | Weighted or majority |
| **Adversarial distillation** | Student must match on hard cases | Expensive |

### Soft Label Quality

- **Temperature tuning**: Higher T → softer distribution, more information per sample.
- **Confidence calibration**: Check that teacher confidence correlates with accuracy.
- **Dark knowledge audit**: Does the student learn non-obvious patterns from soft labels?

```python
# Soft label generation with temperature
import torch.nn.functional as F

with torch.no_grad():
    logits = teacher_model(inputs)
    soft_labels = F.softmax(logits / temperature, dim=-1)

# Student trains on: (input, soft_label)
# Loss: KL(soft_label || student_logits/T) * T^2
```

---

## Part C: Continual Learning

### Task Sequencing Rules

| Rule | Rationale |
|------|-------|
| **Similar tasks grouped** | Avoids catastrophic interference |
| **Increasing difficulty** | Curriculum: easy → hard |
| **Interleaved similar tasks** | Forces model to learn task boundaries |
| **Replay buffer ≥ 1% of old data** | Prevents forgetting |

### Replay Buffer Design

- Store raw examples (not generated) — generative replay introduces cascading errors.
- Stratify buffer by task, class, and difficulty.
- Update buffer with reservoir sampling: each new example has probability k/N of replacing a stored one.

### Evaluation Protocol

- **Forward transfer**: Does task A help task B?
- **Backward transfer**: Does learning task B degrade performance on task A?
- **Forgetting metric**: Drop in task A accuracy after learning task B.

---

## Part D: Federated Learning

### Client Partitioning

| Strategy | Simulates | Use |
|------|-------|-------|
| **IID** (uniform random) | Idealized baseline | Sanity check |
| **Label skew** (Dirichlet α) | Different clients have different label distributions | Realistic for apps |
| **Quantity skew** | Different clients have different amounts of data | Mobile/IoT |
| **Feature skew** | Same labels, different input distributions | Different devices/sensors |
| **Temporal skew** | Concept drift over time | Production deployment |

```python
# Dirichlet partitioning for label skew
import numpy as np

def dirichlet_partition(labels, n_clients, alpha=0.5):
    n_classes = len(np.unique(labels))
    proportions = np.random.dirichlet([alpha] * n_clients, n_classes)
    # Each row: class distribution across clients
    # Lower alpha = more skewed
```

### Differential Privacy Thresholds

- ε = 1-10: meaningful privacy protection, measurable accuracy cost.
- ε < 1: strong privacy, significant utility loss.
- Clip gradients to bound sensitivity (typical C = 1.0).
- Add noise proportional to C/ε.

---

## Part E: Anomaly Detection

### Clean Baseline Construction

| Approach | Risk |
|------|-------|
| Expert-validated normal samples | Expensive, may miss rare normals |
| Remove top-k outliers by density | Removes genuine rare-but-normal |
| Autoencoder reconstruction error | Circular — model defines what's anomalous |

### Anomaly Injection

- Inject known anomalies at controlled rates (1%, 5%, 10%).
- Mix anomaly types: point anomalies, contextual anomalies, collective anomalies.
- NEVER train on injected anomalies — they contaminate the "normal" model.

### Label Integrity

- Anomaly labels are often wrong: "unknown" ≠ "anomalous".
- Validate with domain expert on a stratified sample.
- Report precision/recall for each anomaly type separately.
