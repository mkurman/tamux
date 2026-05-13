---
name: cultural-linguistic-data-task
description: Curate multilingual and multicultural datasets with cultural context awareness — translation quality vs cultural preservation, language-specific bias detection, cross-cultural concept alignment, and localized evaluation validity.
recommended_skills: [multilingual-data-task, bias-audit, embedding-analysis, llm-assisted-curation]
recommended_guidelines: [specialized-modality-data-task, dataset-creation-curation-task]
---

## Overview

Translation is not enough. A dataset that works in English may fail in Japanese, Arabic, or Swahili — not because of translation quality but because concepts, politeness norms, and evaluation criteria differ. This guideline covers cultural and linguistic data patterns that general multilingual guidelines miss.

## Phase 1: Cultural Context Validation

```python
def audit_cultural_translation(source_texts, target_lang, translation_pairs, native_speakers=2):
    """Is the translation culturally appropriate, not just linguistically correct?"""
    issues = []
    for source, translated in translation_pairs:
        # Check for cultural concept mismatch
        cultural_markers = _extract_cultural_markers(source, source_lang="en")
        for marker in cultural_markers:
            if marker["type"] == "idiom" and not _has_equivalent(marker, target_lang):
                issues.append({"text": source[:100], "marker": marker, 
                               "issue": "untranslatable_idiom"})
            elif marker["type"] == "cultural_reference" and not _is_known(marker, target_lang):
                issues.append({"text": source[:100], "marker": marker,
                               "issue": "culturally_opaque_reference"})
            elif marker["type"] == "politeness_marker" and _politeness_mismatch(marker, target_lang):
                issues.append({"text": source[:100], "marker": marker,
                               "issue": "politeness_level_mismatch"})
    
    return {"n_issues": len(issues), "issue_rate": len(issues) / max(len(translation_pairs), 1),
            "acceptable": len(issues) / max(len(translation_pairs), 1) < 0.1}
```

## Phase 2: Language-Specific Bias Patterns

```python
LANGUAGE_BIAS_PATTERNS = {
    "gendered_languages": ["fr", "es", "de", "ar", "he", "pl"],  # grammatical gender
    "honorific_languages": ["ja", "ko", "th", "vi"],  # requires social hierarchy encoding
    "pro_drop_languages": ["ja", "zh", "ko", "es", "it"],  # subject can be omitted
}

def detect_language_specific_bias(model, test_data, language):
    patterns = []
    if language in LANGUAGE_BIAS_PATTERNS["gendered_languages"]:
        patterns.append(_test_grammatical_gender_bias(model, test_data, language))
    if language in LANGUAGE_BIAS_PATTERNS["honorific_languages"]:
        patterns.append(_test_honorific_consistency(model, test_data, language))
    return patterns
```

## Phase 3: Cross-Cultural Concept Alignment

```python
def measure_concept_alignment(concept_pairs, embeddings_model):
    """
    Do concepts map across cultures? "Freedom" in English ≈ "自由" in Japanese?
    But "privacy" in English ≠ exact equivalent in many cultures.
    """
    alignments = {}
    for source_concept, target_concepts in concept_pairs.items():
        source_emb = embeddings_model.encode(source_concept)
        target_embs = {tc: embeddings_model.encode(tc) for tc in target_concepts}
        similarities = {tc: float(np.dot(source_emb, temb) / (np.linalg.norm(source_emb)*np.linalg.norm(temb)))
                        for tc, temb in target_embs.items()}
        alignments[source_concept] = {"similarities": similarities,
                                       "best_match": max(similarities, key=similarities.get),
                                       "alignment_quality": "STRONG" if max(similarities.values()) > 0.85
                                       else "MODERATE" if max(similarities.values()) > 0.7 else "WEAK"}
    return alignments
```

## Phase 4: Localized Evaluation Validity

```python
def validate_localized_benchmark(original_benchmark, translated_benchmark, bilingual_evaluators=3):
    """
    Does the translated benchmark measure the same thing?
    MMLU in Chinese should test the same knowledge, not just be a translation.
    """
    # Check: do bilingual evaluators get similar scores on both versions?
    # If they score 90% on original but 70% on translated → translation lost information
    
    # Check: are culturally-specific questions appropriately adapted?
    # "Who was the 16th US president?" → valid in English
    # In Japanese: should this be a US history question, or adapted to Japanese history?
    
    # Check: are evaluation criteria culturally appropriate?
    # "Is this polite?" → politeness norms differ across cultures
    pass
```

## Quality Gate

- Cultural translation issue rate < 10%.
- Language-specific bias patterns documented for all target languages.
- Cross-cultural concept alignment measured for key concepts.
- Localized benchmarks validated with bilingual evaluators (≥ 3).
- At least one native speaker reviewed outputs per target language.
