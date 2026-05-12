---
name: rl-alignment-data-task
description: Curate preference and reward modeling data for RLHF, DPO, GRPO, and other alignment methods. Covers preference pair construction, ranking data quality, reward model training set design, and chain-of-thought preference annotation.
recommended_skills:
  - trl
  - llm-assisted-curation
  - embedding-analysis
  - dataset-versioning
recommended_guidelines:
  - training-data-design-principles
  - dataset-creation-curation-task
  - llm-training-data-task
---

## Overview

Alignment data teaches models human values, preferences, and safety constraints. It's fundamentally different from pre-training or instruction data: you're not teaching facts, you're teaching judgment. The data must capture nuanced preferences — what makes one response better than another — and it must do so without introducing new biases or reward hacking vulnerabilities.

## Phase 1: Preference Pair Construction

### 1a. What Makes a Good Preference Pair

A preference pair is (prompt, chosen_response, rejected_response). The key properties:

| Property | Requirement |
|-------|-------|
| **Shared prompt** | Both responses answer the same prompt |
| **Meaningful gap** | The chosen response is clearly better, not marginally |
| **Honest rejection** | The rejected response is plausible but flawed, not gibberish |
| **Single-axis difference** | Ideally, responses differ on only one quality axis |
| **No ties** | When responses are equally good/bad, discard the pair |

### 1b. Preference Axes

Explicitly annotate which axis the preference is on:

| Axis | Chosen Example | Rejected Example |
|-------|-------|-------|
| **Helpfulness** | Complete, actionable answer | Vague, incomplete, or evasive |
| **Honesty** | Acknowledges uncertainty | Confidently wrong |
| **Harmlessness** | Refuses harmful request gracefully | Complies with harmful request |
| **Format** | Follows requested format | Ignores format instructions |
| **Conciseness** | Direct and clear | Verbose and rambling |
| **Reasoning** | Sound logic, correct steps | Flawed reasoning or calculation error |

### 1c. Data Sources for Preference Pairs

| Source | Quality | Scale | Bias Risk |
|-------|-------|-------|-------|
| **Human annotators** | Highest | Low ($$$) | Annotator demographics |
| **Human + LLM assistance** | High | Medium | LLM bias in suggestions |
| **LLM-as-judge (strong model)** | Medium-High | High | Strong model blind spots |
| **Synthetic rejection sampling** | Medium | Very High | Requires careful prompt design |
| **Chat comparison data** | High (organic) | Medium | Privacy, selection bias |
| **Constitutional AI** | Medium | High | Constitution author bias |

### 1d. LLM-as-Judge for Preference Data at Scale

Use `llm-assisted-curation` with vLLM/SGLang:

```python
JUDGE_PROMPT = """You are evaluating two responses to the same prompt.

Prompt: {prompt}

Response A: {response_a}
Response B: {response_b}

Which response is better? Consider: helpfulness, honesty, harmlessness.
Reply with ONLY: "A" or "B" or "TIE"
"""

# For multi-axis annotation:
MULTI_AXIS_PROMPT = """...Rate both responses on:
- helpfulness (1-5)
- honesty (1-5)
- harmlessness (1-5)
- reasoning_quality (1-5)
- conciseness (1-5)
Respond with JSON: {"response_a": {...}, "response_b": {...}, "winner": "A"/"B"/"TIE"}
"""
```

**Validation**: Have a second, different model judge independently. Discard pairs where judges disagree.

### 1e. Preference Pair Quality Filters

Remove pairs where:
- Chosen and rejected are near-duplicates (embedding cosine > 0.95).
- The rejected response is nonsensical, empty, or obviously broken.
- The prompt contains PII, toxic content, or benchmark contamination.
- Responses differ only in length but not substance (length bias).
- The chosen response is longer in > 90% of pairs — this teaches the model to be verbose.

## Phase 2: Reward Model Training Data

### 2a. Reward Model Data Design

For RLHF with a separate reward model:
- **Pair count**: 50K–500K high-quality pairs is the practical range.
- **Coverage**: cover all deployment-relevant domains and difficulty levels.
- **Calibration**: include "obviously good" and "obviously bad" anchor pairs.
- **Human baseline**: ≥ 10% of pairs should have human labels for calibration.

### 2b. Ranking Data (Multi-Response)

For methods that use rankings (not just pairs):

```
Prompt → Response 1 (best)
       → Response 2
       → Response 3
       → Response 4 (worst)
```

Ranking data is more information-dense than pairwise data but harder to annotate consistently. Use when:
- You have expert annotators who can maintain consistent ranking criteria.
- The task naturally has a clear quality gradient (math, code, factual QA).

## Phase 3: DPO-Specific Considerations

DPO trains directly on preference pairs without a separate reward model. This simplifies the pipeline but makes data quality even more critical.

### 3a. DPO Data Requirements

- **Higher quality bar than RLHF**: there's no reward model to smooth over noise.
- **Avoid "easy" pairs**: pairs where the rejected response is clearly terrible teach the model nothing.
- **Preference strength matters**: on a 1-5 scale, prefer pairs with ≥ 2-point gap.
- **Reference model matters**: the data must be off-policy relative to the reference model. If you generated responses with the same model you're training, the signal is weak.

### 3b. DPO Data Augmentation

- Generate multiple responses per prompt (vary temperature, model, system prompt).
- Rank them with a judge model.
- Create pairs from non-adjacent ranks (rank 1 vs rank 3, not rank 1 vs rank 2).
- This multiplies your effective dataset size.

## Phase 4: GRPO / Group-Based Methods

GRPO and similar methods compare groups of responses to the same prompt.

### 4a. Group Construction

- Generate 4-16 responses per prompt from the current policy.
- Score all responses (reward model, LLM judge, or verifier for math/code).
- Normalize scores within the group (z-score or rank).
- The group is the training unit — all responses must be from the same model snapshot.

### 4b. Verifiable Domains

For math, code, and formal reasoning:
- Use execution-based verification (unit tests, symbolic equality) instead of learned reward models.
- This provides ground-truth reward signal with zero reward hacking risk.
- Curate prompts with clear, verifiable correct answers.

## Phase 5: Chain-of-Thought Preference Data

For reasoning models, preferences apply to the reasoning trace, not just the final answer.

### 5a. CoT Preference Annotation

- **Correctness**: is the final answer right? (Verifiable domains only.)
- **Reasoning quality**: are the steps logical, complete, and free of errors?
- **Efficiency**: does the trace avoid unnecessary steps or dead ends?
- **Honest uncertainty**: does the model acknowledge when it's unsure?

### 5b. CoT Data Construction

- Generate CoT traces from the model.
- Score with a verifier (math/code) or LLM judge (open-ended).
- Keep traces that are correct AND well-reasoned.
- For preference pairs: choose the trace with better reasoning, even if both are correct.

## Quality Gate

Alignment data is ready when:
- Preference pairs are annotated with axis (helpfulness, honesty, harmlessness).
- LLM judges agree on ≥ 85% of pairs (inter-judge agreement).
- Length bias is measured and controlled (< 90% of chosen responses are longer).
- Reward model training set covers all deployment domains.
- Human-validated subset (≥ 10%) confirms automated labels.
- Data is versioned with provenance for every source.
- No benchmark contamination in prompts or responses.
