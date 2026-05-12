---
name: auditing-goal-artifacts
description: Use when reviewing recent tamux goal run outputs, closure markers, ledgers, or evidence bundles to judge whether completion is credible or to identify remaining uncertainty.
---

# Auditing Goal Artifacts

## Overview

Goal status alone is not enough. Trust the execution artifacts, verification evidence, and sampled bundles over the top-line run label.

**Core principle:** distinguish run-state hygiene from real closure evidence.

## When to Use

Use this skill when:
- the operator asks to review the latest goal results,
- a goal run says `completed`, `paused`, or `failed` and you need to judge whether the result is actually credible,
- you need to inspect closure markers, ledgers, or reviewer-backed reports,
- or you need to explain what is truly finished versus what is only documented as finished.

Do not use this skill when:
- the main task is starting, pausing, resuming, or cancelling a goal run,
- the main task is implementing code rather than auditing evidence,
- or a simple status lookup is enough and no artifact credibility question exists.

## Quick Reference

1. Identify the newest relevant goal run and note status, current step, and latest events.
2. Inspect the execution inventory directory before reading large payloads.
3. Read completion markers, final reports, and ledgers first.
4. Sample 2-3 concrete evidence bundles instead of trusting only summaries.
5. Separate three claims clearly:
   - workflow status,
   - artifact/evidence completeness,
   - fresh implementation versus re-verification of existing work.
6. End with trusted conclusions, weak points, and the best next audit action.

## Implementation

### 1) Start with goal metadata
- Use goal-run listing/details to identify the most recent run and its final or current state.
- Note repeated requeues, approval loops, final-review churn, or pauses separately from the substantive result.

### 2) Inspect inventory structure first
- List the execution directory.
- Look for:
  - `step-*-complete.md`
  - final closure reports
  - pass/block ledgers
  - per-item artifact directories
  - superseded blocker notes

### 3) Read the highest-signal summary artifacts
Read bounded windows from:
- final closure report
- latest step completion marker
- main pass ledger
- reviewer notes

Do not rely on one file alone.

### 4) Sample real evidence bundles
Pick a few representative items, especially:
- anything marked `skip-verified`,
- anything previously blocked,
- or anything whose closure seems surprising.

For each sampled item, check whether these files exist and are non-empty:
- `implementation-summary.md`
- `verification-log.md`
- `review-notes.md`
- `completion-marker.md`

### 5) Check consistency, not just presence
Look for:
- matching counts between report and ledger,
- reviewer notes that actually say something substantive,
- verification logs with concrete commands or exit codes,
- superseded blocked artifacts that are explicitly superseded,
- and whether the report admits when no new code changes were needed.

### 6) Report with the right distinction
Always state whether the run appears to be:
- a true new implementation success,
- a proof/closure pass over already-existing implementation,
- or still uncertain despite a nominal pass.

Treat final-review or pause-state issues as workflow friction unless they undermine the evidence itself.

## Common Mistakes

- Trusting the goal run status without reading artifacts.
- Confusing `pass` with fresh implementation.
- Reading only one completion marker and stopping.
- Treating file presence alone as proof of quality.
- Dumping huge payloads instead of bounded reads and representative samples.
