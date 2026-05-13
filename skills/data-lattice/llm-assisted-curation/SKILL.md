---
name: llm-assisted-curation
description: Use locally-hosted LLMs (vLLM/SGLang) for dataset filtering, quality scoring, rewriting, labeling, and synthetic data generation. Covers LLM-as-judge scoring, structured output filtering, batch inference pipelines, and 2025-2026 techniques (DataRater, perplexity filtering, curriculum scoring, LLM-based dedup).
tags: [llm-curation, vllm, sglang, data-quality, llm-as-judge, synthetic-data, dataset-filtering, dataset-curation, mlops]
---

# LLM-Assisted Dataset Curation

## Overview

Modern dataset curation uses LLMs as quality filters, rewriters, labelers, and synthetic data generators. This skill covers hosting models locally with vLLM/SGLang and using them for dataset work — not for interactive chat, but for batch, structured, reproducible data operations.

## When to Use

Use this skill when:
- Scoring or filtering dataset examples with an LLM quality judge.
- Rewriting noisy text (queries, answers, reasoning traces) in bulk.
- Generating synthetic examples to balance classes or fill gaps.
- Extracting structured labels from unstructured text.
- Running curriculum scoring (difficulty, complexity, educational value).
- Implementing DataRater-style learned quality scoring (2025).

Do not use for:
- Interactive chat or single-example inspection — use a UI.
- Exact-match deduplication — use hashing.
- Embedding-based dedup — use `embedding-analysis` skill.

## Prerequisites

Requires a running vLLM or SGLang server. See `vllm` and `sglang` skills for server setup.

```bash
# vLLM (high throughput)
vllm serve Qwen/Qwen2.5-7B-Instruct --port 8000 --max-model-len 8192

# SGLang (structured output)
python -m sglang.launch_server --model-path Qwen/Qwen2.5-7B-Instruct --port 30000
```

## Core Patterns

### 1. LLM-as-Judge Quality Scoring

Score each example on clarity, correctness, and usefulness.

```python
from openai import OpenAI
import json
from datasets import load_dataset

client = OpenAI(base_url="http://localhost:8000/v1", api_key="not-needed")

QUALITY_PROMPT = """Score the following example on these dimensions (1-5 each):
- clarity: Is the text well-written and understandable?
- correctness: Are the facts accurate?
- usefulness: Would this help someone learn or solve a problem?

Respond with ONLY valid JSON: {"clarity": N, "correctness": N, "usefulness": N}

Example:
{sample}
"""

def score_example(sample: dict) -> dict:
    prompt = QUALITY_PROMPT.format(sample=json.dumps(sample))
    response = client.chat.completions.create(
        model="Qwen/Qwen2.5-7B-Instruct",
        messages=[{"role": "user", "content": prompt}],
        temperature=0.0,  # deterministic
        max_tokens=128,
    )
    try:
        scores = json.loads(response.choices[0].message.content)
    except json.JSONDecodeError:
        scores = {"clarity": 0, "correctness": 0, "usefulness": 0}
    return {**sample, **scores}

# Batch scoring with datasets
dataset = load_dataset("my-dataset", split="train")
scored = dataset.map(score_example)

# Filter low-quality examples
filtered = scored.filter(lambda x: x["clarity"] >= 3 and x["correctness"] >= 3)
```

### 2. Structured Output Filtering (SGLang)

Use SGLang's constrained decoding for guaranteed JSON schema output.

```python
import sglang as sgl

@sgl.function
def classify_quality(s, text: str):
    s += sgl.system("You classify dataset examples. Output ONLY valid JSON.")
    s += sgl.user(f"Classify this example:\n\n{text}")
    s += sgl.gen("result", max_tokens=256, temperature=0.0, schema=json.dumps({
        "type": "object",
        "properties": {
            "quality": {"type": "string", "enum": ["high", "medium", "low", "noise"]},
            "language": {"type": "string", "enum": ["en", "code", "other"]},
            "topic": {"type": "string"},
            "issues": {"type": "array", "items": {"type": "string"}},
        },
        "required": ["quality", "language", "topic", "issues"],
    }))

state = classify_quality.run(text=example["text"])
result = state["result"]  # guaranteed valid JSON
```

### 3. Batch Rewriting/Refinement

Clean noisy data by rewriting through an LLM.

```python
REWRITE_PROMPT = """Rewrite the following text to be clear, grammatical, and well-structured.
Preserve all factual information. Fix typos, grammar, and awkward phrasing.

Original: {text}

Rewritten:"""

def rewrite_text(sample: dict, client, model: str) -> dict:
    prompt = REWRITE_PROMPT.format(text=sample["text"])
    response = client.chat.completions.create(
        model=model,
        messages=[{"role": "user", "content": prompt}],
        temperature=0.3,
        max_tokens=1024,
    )
    sample["text_rewritten"] = response.choices[0].message.content
    return sample

# Process with concurrency
from concurrent.futures import ThreadPoolExecutor, as_completed

def batch_rewrite(dataset, client, model, max_workers=8):
    with ThreadPoolExecutor(max_workers=max_workers) as executor:
        futures = {
            executor.submit(rewrite_text, example, client, model): i
            for i, example in enumerate(dataset)
        }
        results = [None] * len(dataset)
        for future in as_completed(futures):
            idx = futures[future]
            results[idx] = future.result()
    return results
```

### 4. Synthetic Data Generation

Generate additional examples to fill class imbalances or cover edge cases.

```python
SYNTHETIC_PROMPT = """Given this REAL example, generate {n} NEW examples that are:
- Semantically different (new variations, not paraphrases)
- Same difficulty level
- Same format and style
- Realistic and useful

REAL example:
{seed}

Generate {n} new examples as a JSON array of objects with the same keys.
Output ONLY the JSON array."""

def generate_synthetic(seed_examples, client, model, n_per_seed=5):
    synthetic = []
    for seed in seed_examples:
        prompt = SYNTHETIC_PROMPT.format(n=n_per_seed, seed=json.dumps(seed))
        response = client.chat.completions.create(
            model=model,
            messages=[{"role": "user", "content": prompt}],
            temperature=0.8,  # higher for diversity
            max_tokens=2048,
        )
        try:
            generated = json.loads(response.choices[0].message.content)
            synthetic.extend(generated)
        except json.JSONDecodeError:
            continue
    return synthetic
```

### 5. Curriculum Difficulty Scoring

Score examples by difficulty to enable curriculum learning.

```python
DIFFICULTY_PROMPT = """Rate the difficulty of this example on a scale of 1-5:
1 = Trivial, basic knowledge
2 = Easy, common knowledge
3 = Moderate, requires some reasoning
4 = Hard, requires deep understanding
5 = Expert, requires specialized knowledge

Example: {sample}

Difficulty (number only):"""

def score_difficulty(sample, client, model):
    response = client.chat.completions.create(
        model=model,
        messages=[{"role": "user", "content": DIFFICULTY_PROMPT.format(sample=sample["text"])}],
        temperature=0.0,
        max_tokens=4,
    )
    try:
        return int(response.choices[0].message.content.strip())
    except ValueError:
        return 3  # default moderate

# Build curriculum: sort by difficulty
scored = dataset.map(lambda x: {"difficulty": score_difficulty(x, client, model)})
curriculum = scored.sort("difficulty")
```

### 6. LLM-Based Label Extraction

Extract structured labels from unstructured text.

```python
LABELING_PROMPT = """Extract the following labels from this text.
Respond with ONLY valid JSON.

Text: {text}

Labels to extract:
- sentiment: "positive", "negative", or "neutral"
- has_code: true if contains code snippets, false otherwise
- domain: one of ["science", "technology", "business", "arts", "other"]
- entities: list of named entities mentioned
"""

def extract_labels(sample, client, model):
    response = client.chat.completions.create(
        model=model,
        messages=[{"role": "user", "content": LABELING_PROMPT.format(text=sample["text"])}],
        temperature=0.0,
        max_tokens=256,
    )
    try:
        labels = json.loads(response.choices[0].message.content)
        return {**sample, **labels}
    except json.JSONDecodeError:
        return {**sample, "sentiment": None, "has_code": None, "domain": None, "entities": []}
```

## Optimization Patterns

### Openai Batch API (vLLM)

```python
# vLLM supports batch API for cost efficiency on large jobs
# Upload a JSONL file of requests
requests = []
for example in dataset:
    requests.append({
        "custom_id": str(example["id"]),
        "method": "POST",
        "url": "/v1/chat/completions",
        "body": {
            "model": "Qwen/Qwen2.5-7B-Instruct",
            "messages": [{"role": "user", "content": QUALITY_PROMPT.format(sample=example["text"])}],
            "temperature": 0.0,
            "max_tokens": 128,
        }
    })

import tempfile, json
with tempfile.NamedTemporaryFile(mode="w", suffix=".jsonl", delete=False) as f:
    for req in requests:
        f.write(json.dumps(req) + "\n")
    batch_file = f.name

batch = client.files.create(file=open(batch_file, "rb"), purpose="batch")
job = client.batches.create(input_file_id=batch.id, endpoint="/v1/chat/completions", completion_window="24h")
```

## 2025-2026 Literature Integration

This skill integrates techniques from:

| Paper | Venue | Technique | How Applied |
||--------|--------|-------|
| **DataRater** (Calian et al.) | NeurIPS 2025 | Meta-learned quality scoring | `embedding_quality_score()` in `embedding-analysis`; LLM judge as proxy |
| **Why Less is More** (Dohmatob et al.) | 2025 | Theory of data curation thresholds | Informs filtering aggressiveness |
| **GRAPE Score** | 2025 | Perplexity-based filtering | `grape_score()` in `embedding-analysis` |
| **NeMo Curator SemDedup** | 2024-2025 | Clustering-based semantic dedup | `semantic_dedup()` in `embedding-analysis` |
| **LSHBloom** (Khan et al.) | 2025 | Internet-scale text dedup | `lsh_semantic_dedup()` for >100M scale |
| **Blu-WERP** (Rupesh et al.) | 2025 | Scalable preprocessing pipeline | Streaming + batched map pattern |
| **TBDFiltering** (Busa-Fekete et al.) | 2025 | Tree-based data filtering | LLM scoring as tree node condition |
| **Ensembled Multimodal Curation** (Xu et al.) | 2025 | Multi-signal quality fusion | Combine LLM scores + embedding scores + perplexity |

## Quality Gate

An LLM-assisted curation run is complete when:
- The LLM server (vLLM/SGLang) is healthy and reachable.
- Scoring prompts are versioned and produce structured, parseable output.
- Filtered examples are saved with their scores for auditability.
- Synthetic data is flagged with a `synthetic: true` field.
- Batch results are reproducible (temperature=0 for scoring, fixed seed for generation).
- A before/after dataset card documents what was filtered and why.
