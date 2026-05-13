---
name: knowledge-distillation
description: "Knowledge distillation techniques for model compression: logit-level, feature-level, and relation-based distillation. KD-Lib library and practical workflows for training student models."
tags: [knowledge-distillation, model-compression, student-teacher, kd-lib, pytorch, zorai]
---

## Overview

Knowledge distillation transfers knowledge from a larger teacher model to a smaller student model. Combined with pruning and quantization, it forms the critical middle step in the P-KD-Q compression pipeline.

## Installation

```bash
uv pip install kd-lib
```

## Logit Distillation

```python
from kd_lib import distill
import torch.nn.functional as F

def kd_loss(student_logits, teacher_logits, labels, temperature=4.0, alpha=0.5):
    soft_targets = F.softmax(teacher_logits / temperature, dim=-1)
    soft_prob = F.log_softmax(student_logits / temperature, dim=-1)
    kd = F.kl_div(soft_prob, soft_targets, reduction="batchmean") * (temperature ** 2)
    ce = F.cross_entropy(student_logits, labels)
    return alpha * kd + (1 - alpha) * ce
```

## Distillation Methods

| Method | What it transfers | Best For |
|---|---|---|
| **Logit distillation** | Output probability distribution | Classification, generation |
| **Feature distillation** | Intermediate hidden states | Transformer layers |
| **Relation distillation** | Relationships between representations | Structured outputs |
| **Self-distillation** | Model teaches itself | No teacher needed |
| **Online distillation** | Teacher & student train jointly | Both models improve |

## References
- [KD-Lib docs](https://kd-lib.readthedocs.io/)
- [DistilQwen2.5 paper](https://arxiv.org/abs/2504.15027)