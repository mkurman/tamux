---
name: knowledge-engineering-data-task
description: Capture, validate, and encode domain expertise as training data — expert knowledge capture protocols, validation test sets, conflict resolution, procedural knowledge, and tacit knowledge extraction.
recommended_skills: [annotation-management-task, label-quality-audit, llm-assisted-curation]
recommended_guidelines: [annotation-economics-task, dataset-governance-task]
---

## Overview

Domain expertise is the most valuable and hardest-to-capture training data. It lives in experts' heads, not in databases. This guideline covers how to extract, validate, and encode it.

## Phase 1: Expert Knowledge Capture

```python
KNOWLEDGE_CAPTURE_METHODS = {
    "structured_interview": {
        "format": "Expert answers pre-designed questions with follow-ups",
        "captures": "Explicit reasoning, decision rules, edge cases",
        "output": "Q&A pairs with rationale",
        "best_for": "Diagnostic reasoning, classification, decision-making",
    },
    "think_aloud": {
        "format": "Expert verbalizes thought process while performing task",
        "captures": "Implicit strategies, attention patterns, error recovery",
        "output": "Timestamped reasoning traces",
        "best_for": "Procedural tasks, troubleshooting, design",
    },
    "contrastive_pairs": {
        "format": "Expert explains why A is correct and B is incorrect",
        "captures": "Decision boundaries, subtle distinctions",
        "output": "Paired examples with rationale",
        "best_for": "Classification, ranking, preference learning",
    },
    "case_retrospective": {
        "format": "Expert reviews past decisions with outcomes known",
        "captures": "Hindsight bias, outcome-informed reasoning",
        "output": "Case studies with outcome annotation",
        "best_for": "Medical diagnosis, legal analysis, risk assessment",
    },
}
```

## Phase 2: Knowledge Validation

```python
def validate_expert_knowledge(expert_a_answers, expert_b_answers, gold_standard=None):
    """Measure inter-expert agreement and accuracy."""
    from sklearn.metrics import cohen_kappa_score
    
    agreement = cohen_kappa_score(expert_a_answers, expert_b_answers)
    
    if gold_standard:
        accuracy_a = np.mean(expert_a_answers == gold_standard)
        accuracy_b = np.mean(expert_b_answers == gold_standard)
        return {"kappa": agreement, "expert_a_accuracy": accuracy_a, 
                "expert_b_accuracy": accuracy_b,
                "quality": "HIGH" if agreement > 0.8 else "MEDIUM" if agreement > 0.6 else "LOW"}
    
    return {"kappa": agreement, "quality": "accept" if agreement > 0.6 else "reject"}

def resolve_conflicts(experts_answers):
    """When experts disagree, capture the disagreement as data."""
    from scipy.stats import mode
    n_experts = len(experts_answers)
    consensus = mode(experts_answers, axis=0).mode[0]
    disagreement_rate = np.mean([len(set(row)) > 1 for row in experts_answers.T])
    return {"consensus": consensus, "disagreement_rate": disagreement_rate,
            "high_disagreement_indices": np.where([len(set(row)) > 1 for row in experts_answers.T])[0]}
```

## Phase 3: Tacit Knowledge Extraction

Tacit knowledge is what experts can DO but can't easily EXPLAIN. Capture it through behavioral data, not interviews.

| Method | What It Captures | Implementation |
|--------|-----------------|----------------|
| **Mouse tracking** | Attention patterns, hesitation | Record cursor position + timing during task |
| **Eye tracking** | Visual attention, scan paths | Where does expert look vs novice? |
| **Keystroke dynamics** | Fluency, automaticity | Typing speed, correction patterns |
| **Task completion logs** | Strategy differences | Sequence of actions, tool choices |
| **Error recovery patterns** | Resilience strategies | What does expert do after mistake? |

## Phase 4: Knowledge Encoding

```python
def encode_expert_knowledge(captured_data, encoding_format):
    if encoding_format == "rules":
        return [{"condition": ex["context"], "action": ex["decision"], 
                 "rationale": ex["reasoning"], "confidence": ex.get("confidence", 1.0)}
                for ex in captured_data]
    elif encoding_format == "examples":
        return [{"input": ex["scenario"], "output": ex["decision"], 
                 "expert_id": ex["expert"], "is_edge_case": ex.get("is_edge_case", False)}
                for ex in captured_data]
    elif encoding_format == "preferences":
        return [{"prompt": ex["context"], "chosen": ex["correct"], 
                 "rejected": ex["incorrect"], "axis": ex.get("axis", "correctness")}
                for ex in captured_data if ex.get("incorrect")]
```

## Quality Gate

- Inter-expert agreement κ > 0.6 for all captured knowledge.
- Disagreement points identified and documented (they ARE the edge cases).
- At least 2 experts per knowledge domain.
- Knowledge validated against gold standard when available.
- Encoding format appropriate for the target model (rules for symbolic, examples for neural, preferences for RLHF).
