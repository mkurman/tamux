---
name: model-pruning
description: "Structured and unstructured model pruning: weight pruning, attention head pruning, layer removal, and neural architecture search for efficient LLMs."
tags: [model-pruning, model-compression, sparsity, structured-pruning, unstructured-pruning, zorai]
---

## Overview

Pruning removes redundant parameters from neural networks. **Structured pruning** removes entire heads/layers (hardware-friendly), while **unstructured pruning** sets individual weights to zero (higher compression, needs sparse hardware support).

## Pruning Types

| Type | What's removed | Hardware Support | Compression |
|---|---|---|---|
| Unstructured (N:M sparsity) | Individual weights | NVIDIA 2:4 sparse cores | 2x (2:4) |
| Structured — Head pruning | Attention heads | Any GPU | 1.5-3x |
| Structured — Layer pruning | Transformer layers | Any GPU | 1.5-5x |
| Structured — Neuron pruning | FFN neurons | Any GPU | 1.5-2x |

## Example: Unstructured Pruning with SparseGPT

```python
from transformers import AutoModelForCausalLM
import torch

model = AutoModelForCausalLM.from_pretrained("Qwen/Qwen2.5-1.5B-Instruct")

# Apply SparseGPT (one-shot, no training needed)
from sparsegpt import SparseGPT
sparsifier = SparseGPT(model)
sparsifier.compress(sparsity=0.5)  # 50% weight sparsity

# Or use NVIDIA 2:4 sparsity
for name, param in model.named_parameters():
    if param.dim() >= 2:
        param.data = torch.nn.utils.parametrizations.prune_2in4(param.data)
```

## Structured Pruning

```python
# Layer removal example
def prune_layers(model, layers_to_keep):
    model.model.layers = model.model.layers[:layers_to_keep]
    model.config.num_hidden_layers = layers_to_keep
    return model

# After pruning, re-train or distill to recover quality
```

## References
- [SparseGPT paper](https://arxiv.org/abs/2301.00774)
- [LLM Pruning survey](https://arxiv.org/abs/2406.13794)
- [NVIDIA 2:4 sparsity](https://developer.nvidia.com/blog/achieving-fp32-inference-performance-with-sparse-float-core-on-ampere/)