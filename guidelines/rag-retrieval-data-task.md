---
name: rag-retrieval-data-task
description: Curate datasets for RAG and dense retrieval — query-document pair mining, hard negative construction, chunk strategy, cross-encoder distillation, and BEIR/MTEB evaluation benchmarks.
recommended_skills:
  - embedding-analysis
  - chromadb
  - hf-datasets
  - llm-assisted-curation
recommended_guidelines:
  - training-data-design-principles
  - specialized-modality-data-task
---

## Overview

RAG depends on retrieval quality. The retrieval model is only as good as its positive pairs and hard negatives.

## Positive Pair Construction

| Strategy | Quality |
|------|-------|
| Click data (MS MARCO) | High (organic) |
| LLM-generated synthetic queries | Medium-High |
| QA pairs (Stack Overflow, Reddit) | High |
| Summarization: doc → summary as query | Medium |
| Title-body pairs (articles, papers) | Low-Medium |

## Negative Mining

| Strategy | Difficulty |
|------|-------|
| BM25 top-50 (lexical) | Medium |
| In-batch negatives | Medium-hard (free) |
| Hard negatives from current bi-encoder | Hard |
| Cross-encoder scored negatives | Calibrated (highest quality) |

**Rule**: Keep negatives with cross-encoder score 0.1-0.4 — hard enough to learn, not accidentally relevant.

## Chunk Strategy

| Strategy | Size | Best For |
|------|-------|-------|
| Fixed-length | 256-512 tokens | Simple |
| Sentence-boundary | 3-5 sentences | Natural breaks |
| Semantic (embedding split) | Variable | Topic-coherent |
| Hierarchical | Parent + children | Long documents |

**Validate**: Answer spans intact after chunking.

## Evaluation Benchmarks

| Benchmark | Scale | Use |
|------|-------|-------|
| **BEIR** | 18 datasets, 9 domains | Zero-shot retrieval |
| **MTEB** | 58 datasets, 8 tasks | Embedding evaluation |
| **MS MARCO** | Passage/document ranking | Web search |
| **Natural Questions** | Open-domain QA | Wikipedia |
| **SciFact** | Claim verification | Science |

## Quality Gate

- Positives: cross-encoder > 0.7. Hard negatives: cross-encoder < 0.3.
- Chunk boundaries preserve answer spans.
- No benchmark contamination in training data.
- BEIR/MTEB zero-shot baselined before training.
