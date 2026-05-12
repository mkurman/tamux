---
name: benchmark-contamination-scan
description: Scan training data for benchmark contamination — n-gram overlap, embedding similarity, and exact canary detection against 60+ evaluation datasets.
tags: [contamination, benchmark, evaluation, leakage, dedup, dataset-curation, llm-training]
---

# Benchmark Contamination Scan

## Overview

Training on benchmarks is the cardinal sin of ML. This skill scans your training data against a curated exclusion list of 60+ benchmarks using n-gram overlap (fast, high recall) and embedding similarity (slow, high precision).

## When to Use

Mandatory before every training run. Run on pre-training data, instruction tuning data, and RL alignment data.

## Exclusion List

```python
BENCHMARKS = {
    "mmlu": {"name": "MMLU", "source": "huggingface://cais/mmlu"},
    "hellaswag": {"name": "HellaSwag", "source": "huggingface://Rowan/hellaswag"},
    "gsm8k": {"name": "GSM8K", "source": "huggingface://gsm8k"},
    "humaneval": {"name": "HumanEval", "source": "huggingface://openai_humaneval"},
    "boolq": {"name": "BoolQ", "source": "huggingface://boolq"},
    "arc": {"name": "ARC", "source": "huggingface://ai2_arc"},
    "winogrande": {"name": "Winogrande", "source": "huggingface://winogrande"},
    "piqa": {"name": "PIQA", "source": "huggingface://piqa"},
    "siqa": {"name": "Social IQA", "source": "huggingface://social_i_qa"},
    "openbookqa": {"name": "OpenBookQA", "source": "huggingface://openbookqa"},
    "squad": {"name": "SQuAD", "source": "huggingface://rajpurkar/squad"},
    "natural_questions": {"name": "Natural Questions", "source": "huggingface://natural_questions"},
    "triviaqa": {"name": "TriviaQA", "source": "huggingface://trivia_qa"},
    "medqa": {"name": "MedQA", "source": "huggingface://bigbio/med_qa"},
    "pubmedqa": {"name": "PubMedQA", "source": "huggingface://pubmed_qa"},
    # Add all evaluation datasets used in your project
}
```

## N-gram Overlap Scan

```python
from collections import defaultdict
import hashlib

class ContaminationScanner:
    def __init__(self, n=13):
        self.n = n
        self.benchmark_ngrams = {}
        self._load_benchmarks()
    
    def _ngrams(self, text):
        tokens = text.split()
        return set(" ".join(tokens[i:i+self.n]) for i in range(len(tokens) - self.n + 1))
    
    def _load_benchmarks(self):
        """Load and hash all benchmark texts."""
        for bid, bm in BENCHMARKS.items():
            try:
                ds = load_dataset(bm["source"], split="test")
                all_ngrams = set()
                for example in ds:
                    text = " ".join(str(v) for v in example.values() if isinstance(v, str))
                    all_ngrams.update(self._ngrams(text))
                self.benchmark_ngrams[bid] = {
                    "ngrams": all_ngrams,
                    "n_examples": len(ds),
                }
            except Exception as e:
                print(f"Could not load {bm['name']}: {e}")
    
    def scan_example(self, text):
        """Check a single training example. Returns list of [benchmark, n_matches]."""
        example_ngrams = self._ngrams(text)
        if not example_ngrams:
            return []
        hits = []
        for bid, bm_data in self.benchmark_ngrams.items():
            overlap = len(example_ngrams & bm_data["ngrams"])
            if overlap > 0:
                hits.append({"benchmark": BENCHMARKS[bid]["name"], "n_matching_ngrams": overlap})
        return hits
    
    def scan_dataset(self, examples, threshold=5):
        """Scan full dataset. threshold = min matching ngrams to flag."""
        contaminated = []
        for i, example in enumerate(examples):
            text = " ".join(str(v) for v in example.values() if isinstance(v, str))
            hits = self.scan_example(text)
            if any(h["n_matching_ngrams"] >= threshold for h in hits):
                contaminated.append({"index": i, "hits": hits})
        return contaminated
```

## Embedding Similarity Scan

For semantic contamination (paraphrased benchmarks):

```python
from sentence_transformers import SentenceTransformer, util

model = SentenceTransformer("all-MiniLM-L6-v2")

def embedding_scan(text, benchmark_texts, threshold=0.85):
    emb_text = model.encode(text)
    emb_bench = model.encode(benchmark_texts)
    scores = util.cos_sim(emb_text, emb_bench)[0]
    hits = [(i, float(s)) for i, s in enumerate(scores) if s > threshold]
    return hits
```

## Reporting

```markdown
# Contamination Scan Report
Date: [YYYY-MM-DD]
N-gram size: 13
Bencharks scanned: [N]
Training examples: [N]
Contaminated examples: [N] (X.X%)
Excluded from training: [N]

## By Benchmark
| Benchmark | Contaminated | Action |
|-----------|-------------|--------|
| MMLU | 142 | Removed |
| HellaSwag | 18 | Removed |
```

## Quality Gate

- All known benchmarks in the exclusion list.
- N-gram scan has run (fast, mandatory).
- Embedding scan has run on a sample (slow, recommended).
- Contaminated examples are REMOVED from training, not just flagged.
- Scan is re-run after every data update.
