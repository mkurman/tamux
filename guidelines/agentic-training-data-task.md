---
name: agentic-training-data-task
description: Curate datasets for training AI agents — trajectories, tool-use demonstrations, environment interactions, and multi-turn reasoning traces. Covers trajectory dedup, reward signal extraction, safety filtering, and environment diversity.
recommended_skills:
  - llm-assisted-curation
  - embedding-analysis
  - dataset-versioning
  - benchmark-contamination-scan
recommended_guidelines:
  - training-data-design-principles
  - rl-alignment-data-task
  - data-contamination-task
---

## Overview

Agent data is fundamentally different from static text or images. It's sequential, interactive, tool-mediated, and carries reward signals. A trajectory is not just text — it's a causal chain of observations, thoughts, actions, and outcomes.

## Phase 1: Trajectory Data Structure

### The Canonical Trajectory

```python
@dataclass
class TrajectoryStep:
    observation: dict        # what the agent sees (text, image, code, API response)
    thought: str            # internal reasoning (if available)
    action: dict            # what the agent did (tool call, API request, message)
    reward: float           # immediate reward signal
    done: bool              # is the episode complete?
    timestamp: float        # wall clock or step number

@dataclass
class Trajectory:
    id: str
    environment: str        # "web_browser", "code_interpreter", "human_chat"
    task: str               # "book a flight", "debug this code", "answer question"
    steps: List[TrajectoryStep]
    total_reward: float
    success: bool           # did the task complete successfully?
    metadata: dict          # model, temperature, tools available, etc.
```

### Data Sources

| Source | Quality | Scale | Reward Signal |
|-------|-------|-------|-------|
| **Human demonstrations** | Highest | Low | Implicit (human succeeds/fails) |
| **Model rollouts (production)** | High | High | User feedback, task completion |
| **Synthetic rollouts** | Medium | Very High | Verifiable (math, code) or LLM judge |
| **Environment simulation** | Medium-High | Unlimited | Ground truth from simulator |
| **Web trajectories** (Mind2Web, WebArena) | Medium | Medium | Task completion |

## Phase 2: Trajectory Quality Filtering

### What Makes a Good Trajectory

| Criterion | Check | Remove If |
|-------|-------|-------|
| **Task completed** | Overly reliant on recoveries | Failed after 10+ steps — teaches futility |
| **Efficient** | Minimum steps to completion | Used 50 steps for a 3-step task — teaches inefficiency |
| **Tool-call valid** | All tool calls have valid syntax | Malformed function calls — teaches broken behavior |
| **No hallucinated tools** | Tools exist in the environment | Called `search_web` when only `search_code` available |
| **Observation-action causality** | Actions follow from observations | Ignored error messages and kept retrying same thing |
| **No reward hacking** | Reward comes from task, not from exploiting the reward function | Found a way to spam points without completing the task |

```python
def audit_trajectory(traj):
    issues = []
    
    # Task completion check
    if not traj.success and len(traj.steps) > 10:
        issues.append("long_failure")
    
    # Efficiency check
    if traj.success and len(traj.steps) > 5 * _min_steps_for_task(traj.task):
        issues.append("inefficient")
    
    # Repeated action check (stuck loop)
    actions = [s.action for s in traj.steps]
    for i in range(len(actions) - 3):
        if actions[i:i+3] == actions[i+1:i+4]:
            issues.append("stuck_loop")
            break
    
    # Tool hallucination
    available_tools = traj.metadata.get("tools", [])
    for step in traj.steps:
        if step.action.get("tool") not in available_tools:
            issues.append(f"hallucinated_tool:{step.action.get('tool')}")
    
    # Ignored errors
    for i, step in enumerate(traj.steps):
        if "error" in str(step.observation).lower():
            if i + 1 < len(traj.steps):
                next_action = traj.steps[i + 1].action
                if next_action == step.action:  # retried identical action
                    issues.append("ignored_error")
    
    return issues
```

## Phase 3: Reward Signal Extraction

### From Trajectories to Training Signal

```python
def extract_reward_signal(trajectory):
    """
    Different reward types for different training objectives.
    """
    signals = {
        "trajectory_id": trajectory.id,
        "success": trajectory.success,
        "total_reward": trajectory.total_reward,
    }
    
    # Outcome reward: did it work? (sparse)
    signals["outcome"] = 1.0 if trajectory.success else 0.0
    
    # Process reward: was each step reasonable? (dense)
    step_rewards = []
    for step in trajectory.steps:
        step_r = 0.0
        if step.reward:
            step_r += step.reward
        if "error" not in str(step.observation).lower():
            step_r += 0.1  # bonus for error-free steps
        step_rewards.append(step_r)
    signals["process"] = step_rewards
    
    # Tool-use quality: were tool calls correct?
    signals["tool_quality"] = _score_tool_calls(trajectory)
    
    # Efficiency reward: shorter is better (for successful trajectories)
    if trajectory.success:
        optimal_steps = _min_steps_for_task(trajectory.task)
        signals["efficiency"] = max(0, 1 - len(trajectory.steps) / (5 * optimal_steps))
    
    return signals
```

## Phase 4: Multi-Turn Conversation Data

### What Distinguishes Agent Chat from Static Chat

| Aspect | Static Instruction | Agent Conversation |
|-------|-------|-------|
| Turns | Message → Response | Message → Tool Call → Observation → Thought → Response |
| Context | Fixed | Growing (observations accumulate) |
| Failure modes | Hallucination, refusal | Infinite loops, tool misuse, context overflow |
| Evaluation | Single response quality | Trajectory-level success |

### Filtering Multi-Turn Data

- **Truncated conversations**: Remove trajectories that end mid-action (API timeout).
- **Context explosion**: Remove trajectories > 100K tokens (practically untrainable).
- **Delegation loops**: Agent calls another agent that calls back → infinite recursion.
- **Human takeovers**: Flag trajectories where a human intervened (different data distribution).

## Phase 5: Environment Diversity

```python
def measure_environment_coverage(trajectories, embedding_model):
    """
    Does the training data cover diverse environments?
    """
    # Embed each trajectory's task description
    task_texts = [t.task for t in trajectories]
    embeddings = embedding_model.encode(task_texts)
    
    # Cluster to find coverage gaps
    from sklearn.cluster import KMeans
    
    k = min(20, len(trajectories) // 10)
    clusters = KMeans(n_clusters=k, random_state=42).fit_predict(embeddings)
    
    # Entropy of cluster distribution
    _, counts = np.unique(clusters, return_counts=True)
    entropy = -np.sum((counts / len(clusters)) * np.log(counts / len(clusters) + 1e-10))
    max_entropy = np.log(k)
    
    return {
        "n_clusters": k,
        "entropy": entropy,
        "normalized_entropy": entropy / max_entropy,
        "underrepresented_clusters": np.where(counts < len(trajectories) / (k * 3))[0].tolist(),
        "adequate_coverage": entropy / max_entropy > 0.7,
    }
```

## Phase 6: Safety Filtering

| Issue | Detection | Action |
|-------|-------|-------|
| Agent executes dangerous command | Code audit: `rm -rf`, `sudo`, `eval` | Remove trajectory |
| Agent accesses unauthorized URLs | Domain allowlist check | Remove or flag |
| Agent produces harmful content | Toxicity classifier on outputs | Remove |
| Agent impersonates human | Check for "I am", personal details | Flag |
| Agent tries to escape sandbox | "disable safety", "ignore previous" | Remove and harden environment |
| PII in observations | Regex + NER scan | Redact or remove |

## Quality Gate

- Trajectories audited for stuck loops, tool hallucinations, and ignored errors.
- Failed trajectories > 10 steps removed (teach futility).
- Reward signals extracted for outcome, process, tool quality, and efficiency.
- Environment diversity measured (normalized entropy > 0.7).
- Safety filters applied (dangerous commands, unauthorized access, PII).
- Synthetic trajectories flagged separately from human demonstrations.
