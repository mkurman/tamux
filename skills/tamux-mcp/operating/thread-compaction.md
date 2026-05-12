---
name: thread-compaction
description: >-
  Compresses long agent conversation history, file edits, and tool execution loops into a deterministic, high-signal markdown save state. Trigger this when approaching token limits, caught in an error loop, or preparing to pause/resume the agent.
---

# Thread Compaction Protocol

## Objective
Your goal is to summarize the current execution state into a strict, highly compressed markdown format. You must strip all conversational filler, prioritize semantic outcomes over raw JSON tool outputs, and explicitly document failed paths to prevent looping.

## Instructions

### Step 1: Analyze Current State
Review your recent execution history, including the initial prompt, the tools you have called, and the files you have modified.

### Step 2: Extract Key Information
- **Primary Objective:** Identify the core, overarching goal the user originally requested.
- **Phases:** Determine what macro-step was just finished, what is happening right now, and what remains to be done.
- **Working Environment:** Note the active directory, any uncommitted modified files, and necessary context/reference files.
- **Acquired Knowledge:** Document discovered facts and constraints (e.g., specific framework versions, database credential formats, missing dependencies).
- **Dead Ends (CRITICAL):** You *must* list failed attempts, blocked tool calls, and resolved errors so you do not repeat them when you resume.
- **Recent Actions:** Summarize the last 3-5 tool calls in plain text.

### Step 3: Define the Next Step
Formulate a strict, single-action instruction detailing exactly what needs to happen immediately after this compaction is generated.

### Step 4: Generate the Checkpoint
Output exactly one markdown block using the schema below. Do not deviate from this structure or add conversational filler outside of it.

## Output Format Schema

# 🤖 Agent Context: State Checkpoint [Turn/Timestamp]

## 🎯 Primary Objective
> [1-2 sentences strictly defining the end goal. Do not alter this unless instructed by the user.]

## 🗺️ Execution Map
* **✅ Completed Phase:** [Brief summary of the macro-step just finished]
* **⏳ Current Phase:** [What you are currently trying to achieve]
* **⏭️ Pending Phases:** [List of major steps remaining]

## 📁 Working Environment State
* **Active Directory:** `/path/to/current/workdir`
* **Files Modified (Uncommitted/Pending):**
    * `[filepath]` - ([Brief description of edit])
* **Read-Only Context Files:**
    * `[filepath]` - ([Why it is relevant])

## 🧠 Acquired Knowledge & Constraints
* [Fact/Constraint 1]
* [Fact/Constraint 2]

## 🚫 Dead Ends & Resolved Errors
* **Failed:** [What you tried and why it failed] 
    * *Resolution:* [How to avoid this failure going forward]

## 🛠️ Recent Action Summary (Last 3-5 Turns)
1.  `[tool_name]([brief input])` -> [Result]
2.  `[tool_name]([brief input])` -> [Result]

## 🎯 Immediate Next Step
[Strict, single-action instruction to resume execution, e.g., "Debug the ReferenceError in src/index.js line 12"]