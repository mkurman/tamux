---
name: fewshot-retrieval-data-task
description: Curate data for few-shot and retrieval-augmented ML — few-shot validation set construction, in-context learning quality, prompt sensitivity testing, retrieval corpus quality auditing, relevance scoring validation, and retrieval diversity metrics.
recommended_skills: [embedding-analysis, hf-datasets, llm-assisted-curation]
recommended_guidelines: [rag-retrieval-data-task, evaluation-dataset-design-task, synthetic-data-generation-task]
---

## Few-Shot Learning

### Validation Set Construction

```python
def construct_fewshot_benchmark(examples, labels, n_ways=5, n_shots=[1, 5, 10], n_episodes=100):
    """Build N-way K-shot episodes for systematic evaluation."""
    episodes = []
    unique_labels = np.unique(labels)
    for _ in range(n_episodes):
        selected_classes = np.random.choice(unique_labels, n_ways, replace=False)
        support, query = [], []
        for cls in selected_classes:
            cls_examples = examples[labels == cls]
            idx = np.random.permutation(len(cls_examples))
            support.extend(zip(cls_examples[idx[:max(n_shots)]], [cls]*max(n_shots)))
            query.extend(zip(cls_examples[idx[max(n_shots):max(n_shots)+5]], [cls]*5))
        episodes.append({"support": support, "query": query, "classes": selected_classes})
    return episodes

def validate_incontext_learning(model, examples, templates, n_trials=50):
    """Does demonstration format matter?"""
    results = {}
    for template_name, template_fn in templates.items():
        scores = []
        for _ in range(n_trials):
            demo = template_fn(random.sample(examples, 5))
            scores.append(evaluate_with_demo(model, demo))
        results[template_name] = {"mean": np.mean(scores), "std": np.std(scores),
                                   "sensitive": np.std(scores) > 0.05}
    return results
```

### Prompt Sensitivity

```python
def audit_prompt_sensitivity(model, task_examples, prompt_variations):
    """How much does output change with prompt wording?"""
    outputs = {var: [model.generate(prompt.format(**ex)) for ex in task_examples]
               for var, prompt in prompt_variations.items()}
    consistency = {}
    for i in range(len(task_examples)):
        responses = [outputs[var][i] for var in prompt_variations]
        consistency[i] = len(set(responses)) / len(responses)
    return {"mean_consistency": np.mean(list(consistency.values())),
            "prompt_sensitive": np.mean(list(consistency.values())) < 0.8}
```

## Retrieval-Augmented Quality

```python
def audit_retrieval_corpus(corpus, queries, gold_docs):
    """Is retrieved content accurate and relevant?"""
    results = []
    for query, gold in zip(queries, gold_docs):
        retrieved = retrieve(corpus, query, top_k=5)
        results.append({"recall": len(set(retrieved) & set(gold)) / len(gold),
                         "precision": len(set(retrieved) & set(gold)) / len(retrieved),
                         "mrr": 1 / (min([retrieved.index(g) for g in gold if g in retrieved], default=99) + 1)})
    return {"mean_recall": np.mean([r["recall"] for r in results]),
            "mean_precision": np.mean([r["precision"] for r in results]),
            "mrr": np.mean([r["mrr"] for r in results])}

def measure_retrieval_diversity(retrieved_sets):
    """Are retrieved results from the same queries redundant?"""
    diversities = []
    for retrieved_docs in retrieved_sets:
        if len(retrieved_docs) < 2: diversities.append(1.0); continue
        embs = embed(retrieved_docs)
        sims = cosine_similarity(embs)
        mask = ~np.eye(len(embs), dtype=bool)
        diversities.append(1 - sims[mask].mean())
    return {"mean_diversity": float(np.mean(diversities)),
            "redundant": float(np.mean(diversities)) < 0.3}
```

## Quality Gate

- Few-shot benchmark covers ≥ 20 classes, 3+ shot numbers.
- Prompt consistency > 80% across variations.
- Retrieval recall > 0.8 for gold documents.
- Retrieval diversity > 0.3 (not redundant).
