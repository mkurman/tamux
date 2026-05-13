---
name: model-compression
description: "Model compression techniques: pruning, knowledge distillation, and quantization. Covers compression ordering (P-KD-Q), tools, and evaluation metrics."
tags: [model-compression, pruning, distillation, quantization, llm, optimization, zorai]
---

## Overview

Model compression reduces LLM size and inference cost through three main techniques: **pruning** (removing parameters), **knowledge distillation** (training smaller student models), and **quantization** (lower precision weights). A 2025 study on Qwen2.5 3B found the optimal ordering: **Pruning → Knowledge Distillation → Quantization** (P-KD-Q) achieves 3.68x compression while preserving instruction-following and language understanding.

## The Three Techniques

| Technique | What it does | Compression | Quality Impact |
|---|---|---|---|
| **Quantization** | Lower precision (FP16→INT4/FP8) | Highest standalone | Low with PTQ, recoverable with QAT |
| **Structured Pruning** | Remove attention heads, layers, neurons | Moderate | Moderate degradation, recoverable with KD |
| **Knowledge Distillation** | Train smaller model on larger model outputs | Architecture-dependent | Best quality recovery post-pruning |

## Compression Ordering (P-KD-Q)

The paper shows ordering matters critically:

1. **Prune first** — removes redundant parameters, creating a smaller base
2. **Distill** — recover quality by training the pruned model on the original model's outputs
3. **Quantize last** — reduces precision on the already-optimized model

Applying quantization early causes irreversible information loss that impairs subsequent training.

## Available Toolkits

| Toolkit | Features | Use Case |
|---|---|---|
| `angelslim` (Tencent) | PTQ/QAT, pruning, Eagle3 speculative decoding | Full compression pipeline, 1.25-bit to FP8 |
| `nvidia-model-optimizer` | PTQ, QAT, pruning, distillation, speculative decoding | NVIDIA ecosystem, TensorRT-LLM deployment |
| `intel-neural-compressor` | INT8/FP8/INT4 quantization, pruning, distillation | Intel hardware, ONNX Runtime |
| `peft` + `unsloth` | QLoRA fine-tuning | Training-time efficiency, adapter-based |

## Evaluation Metrics

- **Perplexity**: language modeling quality
- **G-Eval / Clarity**: instruction-following quality
- **Compression ratio**: original size / compressed size
- **Inference speed**: tokens/second
- **Accuracy on benchmarks**: MMLU, GSM8K, HumanEval