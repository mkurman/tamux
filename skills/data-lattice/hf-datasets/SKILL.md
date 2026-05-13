---
name: hf-datasets
description: Load, stream, process, and publish datasets with the HuggingFace datasets library. Covers Apache Arrow-backed streaming for large datasets, map/filter operations, train/val/test splitting, interleaving, concatenation, and pushing to the Hub.
tags: [hf-datasets, huggingface, data-loading, streaming, arrow, dataset-curation, transformers-ecosystem]
---

# HuggingFace Datasets

## Overview

The `datasets` library by HuggingFace provides memory-efficient, Apache Arrow-backed access to tens of thousands of public datasets and your own local/remote data. It supports streaming for datasets that don't fit in RAM and integrates directly with `transformers` tokenizers and training loops.

## When to Use

Use this skill when:
- Loading datasets from the HuggingFace Hub or local files (Parquet, JSONL, CSV, Arrow).
- Streaming datasets too large for memory.
- Applying preprocessing, tokenization, or filtering at scale with `.map()` and `.filter()`.
- Pushing curated datasets to the Hub for sharing or versioning.
- Interleaving or concatenating multiple datasets.

Do not use for:
- Small in-memory pandas DataFrames with < 100K rows — use pandas directly.
- Model inference — use `transformers` or `vllm` skills.
- Embedding computation — use `embedding-analysis` skill.

## Installation

```bash
uv pip install datasets pyarrow huggingface_hub
```

## Quick Start

### Loading

```python
from datasets import load_dataset, load_from_disk

# From the Hub
dataset = load_dataset("imdb", split="train")

# From local files
dataset = load_dataset("parquet", data_files="data/*.parquet", split="train")

# From disk (pre-saved)
dataset = load_from_disk("./my_saved_dataset")

# Streaming (never loads full dataset into memory)
dataset = load_dataset("c4", "en", split="train", streaming=True)
```

### Inspection

```python
print(dataset)                    # Dataset info
print(dataset.shape)              # (n_rows, n_columns)
print(dataset.features)           # Schema
print(dataset[0])                 # First row
print(dataset[:5])                # First 5 rows as dict
print(dataset.column_names)       # Column list
```

## Core Operations

### Map (Apply Function)

```python
# Basic mapping
def add_length(example):
    example["text_length"] = len(example["text"])
    return example

dataset = dataset.map(add_length)

# Batched mapping (faster for large datasets)
dataset = dataset.map(add_length, batched=True, batch_size=1000)

# Tokenization with transformers
from transformers import AutoTokenizer
tokenizer = AutoTokenizer.from_pretrained("bert-base-uncased")

def tokenize(batch):
    return tokenizer(batch["text"], truncation=True, padding="max_length", max_length=512)

dataset = dataset.map(tokenize, batched=True)
```

### Filter

```python
# Quality filter: remove short or low-quality examples
dataset = dataset.filter(lambda x: len(x["text"]) > 100)
dataset = dataset.filter(lambda x: x["score"] > 0.5 if x["score"] is not None else False)

# Language filter
from langdetect import detect
dataset = dataset.filter(lambda x: detect(x["text"]) == "en")
```

### Split

```python
# Random split
train_test = dataset.train_test_split(test_size=0.2, seed=42)
train = train_test["train"]
test = train_test["test"]

# Stratified split (requires class labels)
from datasets import ClassLabel, DatasetDict

split_dataset = dataset.class_encode_column("label").train_test_split(
    test_size=0.2, stratify_by_column="label", seed=42
)
```

### Select, Shuffle, Sort

```python
# Shuffle
dataset = dataset.shuffle(seed=42)

# Select specific indices
dataset = dataset.select(range(1000))

# Sort
dataset = dataset.sort("timestamp")
```

### Concatenate and Interleave

```python
from datasets import concatenate_datasets, interleave_datasets

# Concatenate
combined = concatenate_datasets([dataset_a, dataset_b])

# Interleave (round-robin mixing)
mixed = interleave_datasets([dataset_a, dataset_b], probabilities=[0.7, 0.3], seed=42)
```

## Streaming Pipeline Pattern

For datasets that don't fit in RAM, compose a streaming pipeline:

```python
dataset = load_dataset("bigcode/the-stack", "python", split="train", streaming=True)

# Filter first (cheap operation)
dataset = dataset.filter(lambda x: len(x["content"]) > 500)

# Then map (expensive operation only on filtered data)
dataset = dataset.map(enrich_with_heuristics)

# Take only what you need
for i, example in enumerate(dataset):
    if i >= 10000:
        break
    process(example)
```

## Save and Publish

### Save Locally

```python
# Save to disk (Arrow format, fastest reload)
dataset.save_to_disk("./my_dataset")

# Export to other formats
dataset.to_parquet("./my_dataset.parquet")
dataset.to_json("./my_dataset.jsonl")
dataset.to_csv("./my_dataset.csv")
```

### Push to HuggingFace Hub

```python
from huggingface_hub import login
login()

dataset.push_to_hub("my-username/my-curated-dataset")

# With a DatasetDict (train/val/test)
dataset_dict = DatasetDict({
    "train": train_dataset,
    "validation": val_dataset,
    "test": test_dataset,
})
dataset_dict.push_to_hub("my-username/my-curated-dataset")
```

## Performance Tips

1. **Use `batched=True`** in `.map()` — 10-100x faster than row-level mapping.
2. **Filter before map** — avoid expensive operations on data you'll discard.
3. **Use streaming + take** for exploration before committing to a full load.
4. **Prefer Parquet** over JSONL/CSV for on-disk storage — columnar access is faster.
5. **Set `num_proc`** for CPU-bound map operations:

```python
dataset = dataset.map(heavy_function, batched=True, num_proc=4)
```

## Quality Gate

A dataset load pipeline is correct when:
- `.features` matches the expected schema.
- Streaming mode works for datasets > available RAM.
- Filter operations are deterministic (fixed seed where relevant).
- The dataset is saved to disk or pushed to Hub with all splits.
