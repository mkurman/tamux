---
name: specialized-modality-data-task
description: Curate datasets for embeddings (sentence/retrieval), NER (named entity recognition), and audio (TTS/speech-to-text). Covers triplet mining for embeddings, BIO tagging quality and inter-annotator agreement for NER, and transcription accuracy, speaker diarization, and alignment for audio.
recommended_skills:
  - embedding-analysis
  - hf-datasets
  - whisper
  - speechbrain
  - openmed
  - medcat
  - llm-assisted-curation
recommended_guidelines:
  - training-data-design-principles
  - dataset-creation-curation-task
---

## Overview

Embeddings, NER, and audio datasets each have domain-specific quality challenges that general-purpose curation guidelines don't cover. Embedding datasets depend on contrastive structure. NER datasets demand precise token-level annotation. Audio datasets must handle alignment, speaker identity, and acoustic quality.

---

## Part A: Embedding Datasets

Embedding models learn to map inputs to a vector space where similar items are close and dissimilar items are far. The data must explicitly encode these relationships.

### A1. Dataset Formats

| Format | Structure | Use Case |
|-------|-------|-------|
| **Pairs** | (anchor, positive) — these should be close | Sentence similarity, paraphrase |
| **Triplets** | (anchor, positive, negative) — anchor closer to positive than negative | Metric learning, retrieval |
| **Labeled pairs** | (text_a, text_b, similarity_score 0-1) | STS benchmarks |
| **Query-Document** | (query, relevant_doc, irrelevant_docs) | Information retrieval |

### A2. Triplet Mining

Bad triplets teach nothing. The negative must be hard enough to challenge the model but clearly distinguishable.

```python
# Hard negative mining strategy
def mine_triplets(embeddings, labels, margin=0.2):
    """
    For each anchor, find:
    - easiest positive (hardest to pull close) → challenging
    - hardest negative (easiest to confuse) → informative
    
    Skip triplets where negative is already farther than positive + margin.
    """
    from scipy.spatial.distance import cdist
    dists = cdist(embeddings, embeddings, metric='cosine')

    triplets = []
    for i in range(len(embeddings)):
        pos_mask = (labels == labels[i]) & (np.arange(len(labels)) != i)
        neg_mask = labels != labels[i]

        if not pos_mask.any() or not neg_mask.any():
            continue

        hardest_positive = np.argmax(dists[i][pos_mask])  # farthest same-class
        hardest_negative = np.argmin(dists[i][neg_mask])  # closest diff-class

        if dists[i][hardest_negative] > dists[i][hardest_positive] + margin:
            triplets.append((i, hardest_positive, hardest_negative))

    return triplets
```

### A3. Data Augmentation for Embeddings

- **Back-translation**: Translate to another language and back. Preserves meaning, changes phrasing.
- **Span deletion / reordering**: Drop or shuffle non-critical spans.
- **Synonym substitution**: Replace words with synonyms using WordNet or an LLM.
- **Paraphrase generation**: Use an LLM to generate paraphrases.

**Critical**: Validate that augmentation preserves the intended similarity relationship. A back-translated "The cat sat on the mat" → "The feline was seated upon the rug" is a good positive. "The cat sat on the mat" → "The dog ran in the park" is a negative.

### A4. Retrieval-Specific Considerations

- **Query diversity**: Cover informational, navigational, transactional, and comparative queries.
- **Document chunking**: Chunk strategy (fixed, sentence, semantic) must match deployment.
- **Hard negatives from BM25**: Use lexical retrieval to find hard negatives that embedding models might confuse.
- **Cross-encoder distillation**: Use a cross-encoder to score pairs, use those scores as training labels for the bi-encoder.

### A5. Quality Metrics

- **Triplet validity**: % of triplets where anchor-positive distance < anchor-negative distance by > margin.
- **Pair consistency**: On human-labeled similarity pairs, check that embedding cosine correlates with human scores (Spearman ρ > 0.7).
- **Coverage**: Embedding space coverage via k-means; measure cluster entropy.

---

## Part B: NER Datasets

Named Entity Recognition requires token-level annotation with exact span boundaries. Fuzzy matching doesn't work — a one-token offset is a wrong prediction.

### B1. Annotation Format

Use BIO / BIOES tagging:

```
Token       Label
John        B-PER
Smith       I-PER
works       O
at          O
Google      B-ORG
in          O
Mountain    B-LOC
View        I-LOC
.           O
```

### B2. Annotation Quality

| Issue | Detection | Fix |
|-------|-------|-------|
| **B- without I-** | Single-token entity has B- only (valid) vs. missing I- after B- | Check transitions |
| **I- without B-** | Invalid sequence | Fix or remove |
| **Boundary mismatch** | "John Smith" vs. "John" + "Smith" (two entities vs. one) | Define boundary rules explicitly |
| **Overlapping entities** | Same token in two entity types | Choose one or redesign schema |
| **Inconsistent labeling** | "Apple" tagged as ORG in one place, MISC in another | Harmonize |

### B3. Inter-Annotator Agreement

For NER, use entity-level agreement, not token-level:

```python
def entity_f1(annotator_a, annotator_b):
    """F1 on exact entity spans (type + boundaries must match)."""
    entities_a = set(extract_entities(annotator_a))
    entities_b = set(extract_entities(annotator_b))
    tp = len(entities_a & entities_b)
    precision = tp / len(entities_a) if entities_a else 0
    recall = tp / len(entities_b) if entities_b else 0
    return 2 * precision * recall / (precision + recall) if (precision + recall) else 0
```

Target: entity-level F1 > 0.85 between annotators. Below 0.8, revise annotation guidelines.

### B4. Entity Schema Design

- Keep the tagset small and clear. 5-10 entity types is more sustainable than 50.
- Define entity boundaries precisely: Is "New York City" one LOC or three? Write the rule.
- Include an "IGNORE" or "O" class for non-entity tokens.
- For nested entities ("[University of [California]]"), decide: flat or nested schema?

### B5. Domain Adaptation

NER models degrade sharply on out-of-domain text. When building a domain-specific NER dataset:
- Source text from the target domain, not generic newswire.
- Include domain entities even if they're rare in general text (drug names, legal citations, product SKUs).
- Use `medcat` or `openmed` for biomedical NER; they provide domain-specific entity linkers.

### B6. LLM-Assisted NER Annotation

Use `llm-assisted-curation` with SGLang structured output:

```python
NER_PROMPT = """Extract all named entities from the following text.
Return a JSON array of entities with: text, start_char, end_char, type.
Valid types: PERSON, ORG, LOC, DATE, PRODUCT.

Text: {text}
Entities (JSON array):"""

# Validate with SGLang schema constraint:
schema = {
    "type": "array",
    "items": {
        "type": "object",
        "properties": {
            "text": {"type": "string"},
            "start_char": {"type": "integer"},
            "end_char": {"type": "integer"},
            "type": {"enum": ["PERSON", "ORG", "LOC", "DATE", "PRODUCT"]},
        },
        "required": ["text", "start_char", "end_char", "type"],
    },
}
```

**Always human-validate** LLM-generated NER annotations on a stratified sample before trusting them.

---

## Part C: Audio Datasets (TTS / STT)

### C1. Speech-to-Text (STT) Data

#### Audio Quality Requirements

| Issue | Detection | Action |
|-------|-------|-------|
| Clipping / distortion | Peak amplitude analysis | Flag or remove |
| Background noise | SNR estimation | Flag for denoising or removal |
| Low sample rate | < 16 kHz | Upsample or remove |
| Silence / non-speech | VAD (Voice Activity Detection) | Trim or remove |
| Multiple speakers (unlabeled) | Diarization check | Label or separate |
| Codec artifacts | Spectral analysis | Remove if severe |

#### Transcription Quality

- **Word Error Rate (WER)** between transcription and a reference: target < 5% for training data.
- **Forced alignment**: Align transcript words to audio timestamps. Misaligned data produces models that hallucinate or skip words.
- **Punctuation and casing**: Decide early whether transcripts include punctuation. Be consistent.
- **Disfluencies**: Decide whether to include "um", "uh", false starts. Removing them produces cleaner output but less natural models.

#### STT Data Augmentation

- **Speed perturbation**: 0.9x, 1.0x, 1.1x playback speed.
- **Background noise mixing**: Add ambient noise from MUSAN or similar.
- **Room impulse response (RIR)**: Simulate different acoustic environments.
- **SpecAugment**: Time and frequency masking on spectrograms.

Use `whisper` for baseline transcription, `speechbrain` for custom STT training.

### C2. Text-to-Speech (TTS) Data

#### Speaker Requirements

- **Single speaker (voice cloning)**: 1-10 hours of clean, consistent audio.
- **Multi-speaker**: 10-100+ speakers, balanced by gender, age, accent.
- **Speaker consistency**: Same speaker should have consistent pitch, pace, and style across recordings.

#### Transcript Quality for TTS

TTS is more sensitive to transcript quality than STT:
- **Exact pronunciation match**: The transcript must match what's spoken EXACTLY. "gonna" vs. "going to" matters.
- **Phonetic coverage**: Ensure all phonemes in the target language appear in the dataset.
- **Prosody diversity**: Include questions, exclamations, statements, lists, and long sentences.
- **Numbers and abbreviations**: Decide on expansion rules. "2024" → "twenty twenty-four" or "two thousand twenty-four"?

#### TTS Data Validation

- **Forced alignment** with Montreal Forced Aligner (MFA) or similar.
- **MOS (Mean Opinion Score)** on a held-out set. Target > 4.0 for production.
- **Speaker embedding consistency**: Same speaker's embeddings should cluster tightly.

### C3. Speaker Diarization Data

For multi-speaker audio:
- **Overlap handling**: Label overlapping speech regions. Most diarization models struggle here.
- **Speaker turn boundaries**: Accurate to within 100ms.
- **Minimum segment length**: 1-2 seconds for trainable segments.

## Quality Gate

- **Embeddings**: Triplet validity > 95%, human similarity correlation > 0.7.
- **NER**: Entity-level inter-annotator F1 > 0.85, no invalid BIO transitions.
- **STT**: WER < 5% on training data, forced alignment complete.
- **TTS**: Forced alignment verified, phoneme coverage complete, MOS > 4.0 on held-out set.
- All datasets are versioned with provenance.
