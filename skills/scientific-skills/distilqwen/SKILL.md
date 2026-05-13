---
name: distilqwen
description: "DistilQwen2.5 — Alibaba's industrial practices for training distilled open lightweight language models. Knowledge distillation from Qwen2.5 72B into smaller 0.5B-7B models."
tags: [distilqwen, knowledge-distillation, qwen, lightweight-llm, alibaba, zorai]
---

## Overview

DistilQwen2.5 (Alibaba, arXiv:2504.15027) provides industrial practices for training distilled open lightweight LLMs. The approach distills Qwen2.5-72B-Instruct into smaller models (0.5B, 1.5B, 3B, 7B) with strong performance retention.

## Key Techniques

- **Logit-level distillation**: transfer output distribution from teacher to student
- **Representation alignment**: align hidden states between teacher and student layers
- **Data curriculum**: progressive difficulty in training data selection
- **Multi-stage training**: pre-training distillation → instruction tuning → preference alignment

## Usage

The distilled models are available on HuggingFace as `distilqwen/distilqwen2.5-*-instruct` and can be used directly:

```python
from transformers import AutoModelForCausalLM, AutoTokenizer

model = AutoModelForCausalLM.from_pretrained("distilqwen/distilqwen2.5-1.5b-instruct")
tokenizer = AutoTokenizer.from_pretrained("distilqwen/distilqwen2.5-1.5b-instruct")
```

## References
- [DistilQwen2.5 paper](https://arxiv.org/abs/2504.15027)
- [DistilQwen HuggingFace](https://huggingface.co/distilqwen)