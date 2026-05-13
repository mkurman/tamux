---
name: multilingual-data-task
description: Curate multilingual datasets — language coverage auditing, script/encoding validation, translation quality gates, tokenizer fertility scoring, and code-switching handling.
recommended_skills:
  - hf-datasets
  - embedding-analysis
  - llm-assisted-curation
recommended_guidelines:
  - training-data-design-principles
  - llm-training-data-task
---

## Overview

Multilingual datasets break in ways monolingual ones don't: script mixing, encoding corruption, language misidentification, and tokenizer fertility disparities. Curation means validating that every language in your dataset is actually present, correctly encoded, and adequately represented.

## Language Audit

### Detection and Validation

```python
import langdetect
import fasttext

# fasttext is more accurate than langdetect for short texts
model = fasttext.load_model("lid.176.bin")

def detect_language(text, min_chars=20):
    if len(text) < min_chars:
        return "unknown"
    pred = model.predict(text.replace("\n", " "), k=1)
    lang = pred[0][0].replace("__label__", "")
    confidence = pred[1][0]
    return lang if confidence > 0.7 else "unknown"
```

### Language Coverage Matrix

| Language | # Examples | % of Total | Script | Tokenizer Fertility |
|------|-------|-------|-------|-------|
| English (en) | 1M | 40% | Latin | 1.2 |
| Chinese (zh) | 250K | 10% | Han | 2.8 |
| Arabic (ar) | 100K | 4% | Arabic | 2.1 |
| ... | | | | |

### Red Flags

- Language detected ≠ language tag on record (mismatch > 5% for any language).
- Script mismatch: language normally in Script A but found in Script B.
- Code-switching undetected: text contains > 30% tokens in another language without being tagged.
- Encoding corruption: Unicode replacement characters (�), mojibake, double-encoded UTF-8.

## Script and Encoding

| Issue | Detection | Action |
|------|-------|-------|
| Replacement chars (U+FFFD) | `"�" in text` | Flag; attempt recovery or remove |
| Control characters in text | Regex `[\x00-\x08\x0b\x0c\x0e-\x1f]` | Strip |
| Mixed scripts (unexpected) | Script distribution per language | Flag if > 10% non-target script |
| RTL/LTR mixing | Unicode bidi control chars | Preserve if intentional; flag otherwise |
| Normalization | NFC vs NFD | Normalize to NFC |

## Translation Quality

When using translated data as training data:

- **Back-translation consistency**: Translate to pivot language and back. BLEU < 0.3 with original → poor quality.
- **LLM quality assessment**: Rate translation fluency + adequacy on 1-5 scale.
- **Source language identification**: Is the source text from a native speaker or already a translation?

## Tokenizer Fertility

```python
# Fertility = tokens per word
# High fertility → tokenizer is inefficient for this language
# Common issue: Latin-script tokenizers on CJK, Arabic, Hindi
from transformers import AutoTokenizer

tokenizer = AutoTokenizer.from_pretrained("bert-base-uncased")
texts = ["Hello world", "你好世界", "مرحبا بالعالم"]
for text in texts:
    tokens = tokenizer.encode(text)
    words = len(text.split()) or len(text)
    fertility = len(tokens) / words
    print(f"{text}: fertility={fertility:.2f}")
```

## Quality Gate

- Language distribution measured and documented.
- Language-tag accuracy > 95% for all languages with > 10K examples.
- Script encoding validated — no mojibake, no replacement chars.
- Tokenizer fertility reported per language; high-fertility languages flagged for subword vocabulary review.
- Code-switching documented if present.
