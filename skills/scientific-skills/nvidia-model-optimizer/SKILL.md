---
name: nvidia-model-optimizer
description: "NVIDIA Model Optimizer — quantization, pruning, distillation, and speculative decoding for accelerating LLMs, diffusion models, and vision models on NVIDIA GPUs."
tags: [nvidia-model-optimizer, quantization, pruning, distillation, modelopt, tensorrt, zorai]
---

## Overview

NVIDIA Model Optimizer provides state-of-the-art model optimization techniques — quantization (FP8, INT4, NVFP4), pruning, knowledge distillation, and speculative decoding — for models deployable on TensorRT-LLM, SGLang, and vLLM.

## Installation

```bash
uv pip install nvidia-modelopt
```

## Basic Quantization

```python
import modelopt.torch.quantization as mtq

# FP8 post-training quantization
quant_cfg = mtq.FP8_DEFAULT_CFG
mtq.quantize(model, quant_cfg, forward_loop=calib_loop)

# Export for TensorRT-LLM
from modelopt.torch.export import export_tensorrt_llm_checkpoint
export_tensorrt_llm_checkpoint(model, "model.pt", dtype="fp8")
```

## Pruning + Distillation

```python
import modelopt.torch.pruning as mtp
import modelopt.torch.distill as mtd

# Prune
pruned = mtp.prune(model, ratio=0.3, structure="2:4_sparse")

# Distill
teacher = load_teacher_model()
student = mtd.distill(student_model, teacher, kal="logit", alpha=0.5)
```

## References
- [NVIDIA ModelOpt docs](https://nvidia.github.io/Model-Optimizer)
- [NVIDIA ModelOpt GitHub](https://github.com/NVIDIA/Model-Optimizer)