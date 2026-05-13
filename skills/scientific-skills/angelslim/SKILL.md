---
name: angelslim
description: "Tencent AngelSlim — accessible, comprehensive, and efficient toolkit for large model compression. Quantization (FP8/INT4/NVFP4/1.25-bit), pruning, speculative decoding (Eagle3), and diffusion model compression."
tags: [angelslim, model-compression, quantization, pruning, speculative-decoding, tencent, zorai]
---

## Overview

AngelSlim integrates mainstream compression algorithms into a unified framework with one-click access. Supports **FP8/INT8/INT4/NVFP4/1.25-bit** quantization, pruning, Eagle3 speculative decoding, and diffusion model compression for LLMs, VLMs, and audio models.

## Installation

```bash
uv pip install angelslim
```

## Basic Quantization (PTQ)

```bash
import angelslim as slim

# FP8 static quantization
model = slim.quantize(model, dtype="fp8_static", qconfig="default")

# INT4 GPTQ
model = slim.quantize(model, dtype="int4_gptq", dataset="wikitext2")
```

## Compression Strategies

| Method | Precision | Best For |
|---|---|---|
| FP8-Static/Dynamic | 8-bit | General LLM deployment |
| INT4 GPTQ/AWQ/GPTAQ | 4-bit | Memory-constrained serving |
| NVFP4 | 4-bit (NVIDIA) | Blackwell GPUs |
| Sherry | 1.25-bit | Extreme compression |
| STQ1_0 | 1.25-bit | On-device deployment |

## Speculative Decoding (Eagle3)

```python
# Train Eagle3 draft model
slim.eagle3.train(model, draft_model_config)

# Inference with Eagle3
output = model.generate_with_eagle3(input_ids, max_new_tokens=256)
```

## References
- [AngelSlim GitHub](https://github.com/Tencent/AngelSlim)
- [AngelSlim docs](https://angelslim.readthedocs.io/)
- [Paper: arXiv 2602.21233](https://arxiv.org/abs/2602.21233)