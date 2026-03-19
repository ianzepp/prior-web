# UI Ideas

This is a rough product/UI direction note, not a committed design spec.

The main conclusion: Prior probably does not want one unified UI. The architecture
already splits naturally into multiple operator surfaces, and forcing them all
into one dashboard creates noise quickly.

## Core View

Prior is at least three systems at once:

- a transport/session system
- a room-based interaction system
- a persisted factory/control-plane system

Those are related, but they are not the same interaction model.

The likely mistake is trying to collapse them into one homepage or one
"mission control" screen.

## Recommended Product Surfaces

### 1. Run Workbench

Primary object:

- one factory run

This should likely be the primary application surface.

Why:

- the software factory is persisted and lifecycle-driven
- runs are the main orchestration unit
- most operator questions are run-shaped:
  - what state is this run in?
  - what is blocked?
  - what should happen next?
  - did verification pass?
  - what do the gates say?

Likely navigation inside a run:

- Summary
- Stages
- Issues
- Artifacts
- Verify
- Gates
- Review

Design precedent:

- closer to Linear, GitHub Actions, Jenkins, or a workflow console
- not chat-first

### 2. Room Console

Primary object:

- one room

Why:

- rooms are live execution contexts
- repo intake and user/general rooms are naturally conversational
- room history, active actors, and message send are better treated as a clean
  workspace than as dashboard widgets

Likely contents:

- room selector
- transcript/history
- active actors
- message input
- optional room metadata / linked run / linked repo context

Design precedent:

- closer to Slack/Discord/ChatGPT
- but with stronger room identity and operator metadata

### 3. Repo Console

Primary object:

- one repo

Why:

- repos are durable anchors across runs and rooms
- provider-facing actions belong here
- import/sync/publish should not be buried inside run or room screens

Likely contents:

- repo summary
- import/sync status
- linked rooms
- related runs
- publishable branches / checkpoints

Design precedent:

- closer to a repository admin console than a chat app

### 4. Ops Monitor

Primary object:

- the system itself

Why:

- transport health and live session visibility matter
- but they are operational concerns, not the main product surface

Likely contents:

- gate/server health
- active rooms
- active runs
- recent transport/runtime events
- failures and degraded states

Design precedent:

- closer to a lightweight ops dashboard
- not the core user-facing homepage

### 5. Artifact / Lineage Inspector

Primary object:

- one artifact or one run's artifact graph

Why:

- Prior is compiler-shaped
- typed artifacts are one of the system's main advantages
- users need to inspect lowering outputs and evidence directly

Likely contents:

- artifact list
- artifact detail view
- producer / consumer links
- lineage graph
- payload viewer

Design precedent:

- part document inspector
- part graph/lineage browser

## What Is Probably Wrong

These directions are tempting but likely wrong for this architecture:

### One Giant Mission Control Homepage

Why wrong:

- too many conceptual layers at once
- transport, rooms, repos, runs, and artifacts all compete for attention
- turns into noise fast

### Chat-First for Everything

Why wrong:

- rooms matter, but the factory is not fundamentally a chat product
- stages, artifacts, gates, checkpoints, and review bundles are first-class

### Kanban-Only

Why wrong:

- issues and stages matter, but artifact lineage and gate evidence matter too
- kanban alone under-models the system

### IDE Clone as the Primary Surface

Why wrong:

- useful later for issue execution or repo editing
- but not the main control-plane surface

## If There Is One Primary UI

If Prior has one primary application shell, it should probably be:

- a run-centric control surface

Top-level navigation:

- Runs
- Rooms
- Repos
- Ops

Default landing:

- Runs

Inside a run:

- Summary
- Stages
- Issues
- Artifacts
- Verify
- Gates
- Review

This maps more cleanly to the current backend model than a blended dashboard.

## Website Framework Implication

The website state model should support multiple frontends cleanly instead of
trying to make one layout do everything.

The current likely framework targets are:

1. Run Workbench
2. Room Console
3. Repo Console
4. Ops Monitor

The website-side model should therefore center on:

- factory snapshots
- room session interactions
- repo inventory and actions
- operational health

and not on one unified page model.

## Near-Term Recommendation

Prior-web should continue building substrate first:

- typed factory client
- typed room/repo client
- snapshot aggregation
- command/result model
- polling/sync invalidation model

Then build the four surfaces above on top of that substrate rather than
continuing to iterate on one noisy dashboard.
