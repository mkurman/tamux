---
name: learning-paradigm-data-task
description: Curate data for specialized learning paradigms — contrastive pair validation, self-supervised pretext quality, diffusion conditioning alignment, autoregressive sequence coherence, semi-supervised label propagation, and weak supervision source quality.
recommended_skills: [embedding-analysis, llm-assisted-curation, label-quality-audit]
recommended_guidelines: [specialized-training-data-task, synthetic-data-generation-task]
---

## Contrastive Pairs

```python
def validate_contrastive_pair(anchor, positive, negative, model):
    """Does the pair correctly encode similarity?"""
    a, p, n = model.encode([anchor, positive, negative])
    pos_sim = np.dot(a, p) / (np.linalg.norm(a) * np.linalg.norm(p))
    neg_sim = np.dot(a, n) / (np.linalg.norm(a) * np.linalg.norm(n))
    return {"valid": pos_sim > neg_sim + 0.1, "margin": pos_sim - neg_sim,
            "issues": ["POSITIVE_TOO_DISSIMILAR"] if pos_sim < 0.5 else [] +
                      (["NEGATIVE_TOO_SIMILAR"] if neg_sim > 0.3 else [])}
```

## Self-Supervised Pretext

```python
def validate_pretext_task(model, downstream_eval, pretext_performance):
    """Does pretext task improvement correlate with downstream improvement?"""
    corr = np.corrcoef(pretext_performance, downstream_eval)[0, 1]
    return {"pretext_downstream_correlation": corr,
            "valid_pretext": corr > 0.3,
            "invalid": corr < 0,  # pretext hurts downstream = wrong task
            "recommendation": "GOOD_PRETEXT" if corr > 0.3 else "REDESIGN_PRETEXT"}
```

## Diffusion Conditioning

```python
def audit_prompt_image_alignment(prompts, images, clip_model):
    """Does the prompt actually describe the image?"""
    alignments = []
    for prompt, image in zip(prompts, images):
        score = clip_model.alignment(prompt, image)
        alignments.append({"prompt": prompt[:100], "score": float(score),
                           "quality": "GOOD" if score > 0.25 else "MISMATCH"})
    return {"mean_alignment": np.mean([a["score"] for a in alignments]),
            "mismatch_rate": np.mean([a["quality"]=="MISMATCH" for a in alignments])}
```

## Autoregressive Quality

```python
def audit_sequence_coherence(sequences, model, window=100):
    """Do tokens flow naturally?"""
    perplexities = [model.perplexity(seq) for seq in sequences]
    token_boundary_errors = 0
    for seq in sequences:
        tokens = model.tokenize(seq)
        # Check BOS/EOS consistency, token boundary correctness
        if tokens[0] != model.bos_token_id: token_boundary_errors += 1
        if tokens[-1] != model.eos_token_id: token_boundary_errors += 1
    
    return {"mean_perplexity": np.mean(perplexities), "token_errors": token_boundary_errors,
            "coherent": np.mean(perplexities) < 50 and token_boundary_errors < len(sequences) * 0.05}
```

## Semi-Supervised Label Propagation

```python
def validate_pseudo_label_quality(model, unlabeled_data, confidence_threshold=0.9):
    predictions = model.predict_proba(unlabeled_data)
    confidence = predictions.max(axis=1)
    pseudo_labels = predictions.argmax(axis=1)
    
    high_conf_mask = confidence >= confidence_threshold
    return {"pseudo_labeled_pct": high_conf_mask.mean(),
            "mean_confidence": float(confidence.mean()),
            "low_confidence_pct": float((confidence < 0.6).mean()),
            "propagation_quality": "GOOD" if high_conf_mask.mean() > 0.5 and confidence.mean() > 0.8 else "NEEDS_REVIEW"}
```

## Weak Supervision

```python
def audit_labeling_functions(lfs, validation_data, gold_labels):
    """Measure each labeling function's accuracy and coverage."""
    lf_quality = {}
    for lf_name, lf_fn in lfs.items():
        predictions = lf_fn(validation_data)
        covered = predictions != -1  # -1 = abstain
        if covered.sum() == 0:
            lf_quality[lf_name] = {"coverage": 0, "accuracy": None, "useless": True}
            continue
        accuracy = (predictions[covered] == gold_labels[covered]).mean()
        lf_quality[lf_name] = {"coverage": float(covered.mean()), "accuracy": float(accuracy),
                                "useful": covered.mean() > 0.1 and accuracy > 0.6}
    
    return {"lfs": lf_quality, "consensus_possible": sum(1 for lf in lf_quality.values() if lf.get("useful")) >= 2}
```

## Quality Gate

- Contrastive pairs: positive similarity > negative + 0.1 margin.
- Self-supervised: pretext-downstream correlation > 0.3.
- Diffusion: prompt-image alignment > 0.25 mean.
- Autoregressive: sequences start/end with correct BOS/EOS tokens.
- Semi-supervised: high-confidence pseudo-labels > 50%.
- Weak supervision: ≥ 2 useful labeling functions.
