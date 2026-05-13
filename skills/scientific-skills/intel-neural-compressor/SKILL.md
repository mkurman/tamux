---
name: intel-neural-compressor
description: "Intel Neural Compressor — SOTA low-bit LLM quantization (INT8/FP8/INT4/NVFP4), sparsity, pruning, and distillation for PyTorch, TensorFlow, and ONNX Runtime."
tags: [intel-neural-compressor, quantization, pruning, distillation, inc, intel, zorai]
---

## Overview

Intel Neural Compressor provides low-bit quantization (INT8, FP8, INT4, MXFP4, NVFP4), sparsity, pruning, and knowledge distillation for optimizing models on Intel hardware and beyond.

## Installation

```bash
uv pip install neural-compressor
```

## Basic Quantization

```python
from neural_compressor import Quantization, config

# Post-training quantization
quantizer = Quantization(config)
q_model = quantizer(model)
q_model.save("quantized_model")
```

## Pruning

```python
from neural_compressor import Pruning

pruner = Pruning(model, config={"pruning_type": "snip_momentum", "target_sparsity": 0.3})
pruned_model = pruner.fit()
```

## References
- [Intel NC docs](https://github.com/intel/neural-compressor)
- [Intel NC GitHub](https://github.com/intel/neural-compressor/wiki)