# SynthLabs Workflows

> Route SynthLabs Reasoning Generator work through the right built-in tamux skill. This is a workflow guide for operating an external app with existing tamux tools, not a dedicated SynthLabs tool reference.

## Agent Rules

- Use `synthlabs-setup` before any work that depends on a local SynthLabs instance.
- Use `synthlabs-generation` for backend-first session and generation workflows.
- Use `synthlabs-curation` for autoscore, rewrite, remove-items, and job polling.
- Use `synthlabs-ui-operator` for verifier review, data preview, DEEP mode, and visual-only workflows.

## Workflow Routing

| If you need to... | Load this skill |
| --- | --- |
| Find a SynthLabs checkout, install dependencies, start the app, or verify `/health` | `synthlabs-setup` |
| Create/list sessions or run backend-first generation flows | `synthlabs-generation` |
| Start or monitor autoscore, rewrite, remove-items, or migrate-reasoning jobs | `synthlabs-curation` |
| Drive verifier review, data preview, settings, or DEEP mode in the browser | `synthlabs-ui-operator` |

## Reference

- `synthlabs-setup` owns local instance health, port probing, and environment guidance.
- `synthlabs-generation` owns repeatable backend workflows and long-running generation orchestration.
- `synthlabs-curation` owns verification loops around job-based cleanup and quality-control flows.
- `synthlabs-ui-operator` owns visual-only or UI-led tasks that should run through tamux browser tooling.
- Treat this as a content-level integration: use existing tamux terminals, tasks, goals, and browser capabilities rather than assuming dedicated SynthLabs tools exist.