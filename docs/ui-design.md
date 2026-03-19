# UI Design

This document describes the concrete UI design for prior-web, derived from two
sources: the Gauntlet Week 1 collaborative board project (design language and
layout patterns) and the Prior software factory architecture (domain model and
data flow).

This supersedes the exploratory notes in `ui-ideas.md` with specific layout
decisions, navigation structure, and component mappings.

## Design Principles

Prior-web is an operations tool, not a marketing site.

- Dense, monospace, content-forward (IBM Plex Mono, 11-12px base)
- Zero or minimal border-radius, no glass morphism, no backdrop blur
- No marketing copy, no hero sections, no illustrative placeholders
- Every pixel shows real data or provides real navigation
- Dark theme, warm neutrals (derived from Gauntlet palette)
- Keyboard-navigable, sortable, filterable

## Navigation Structure

Three levels of zoom:

```
Repo/Run List (card grid)            "where is everything"
  click card
    Run Workbench (trace-shell)      "what is happening in this run"
      click entity
        Inspector panel              "show me the details"

  click into room
    Room Console (board-shell)       "live conversation for this repo"
```

Top-level navigation (persistent rail or toolbar):

- Runs (default landing, card grid)
- Rooms (room list or direct room entry)
- Ops (system health, transport status)

## Two Layout Shells

The entire UI is built from two reusable layout shells, not four bespoke
surfaces. Both are derived directly from Gauntlet Week 1 patterns.

### Board Shell (from Gauntlet board view)

Used for: Room Console

Structure:

```
toolbar (48px)
  back / breadcrumb | room name | active actors | actions

left rail (52px collapsed, ~200px expanded)
  room list / repo actions (import, sync, publish)

central workspace (flex)
  transcript / message history
  message input at bottom

right rail (52px collapsed, ~360px expanded, tabbed)
  tabs: actors, linked runs, repo metadata, room history

status bar (32px)
  connection status | session identity | room metrics
```

This is the conversational, presence-aware, real-time surface. One instance per
repo room. The central area is the transcript. Actors appear in the right rail.
Linked factory runs are accessible from the right rail tabs.

### Trace Shell (from Gauntlet trace view)

Used for: Run Workbench

Structure:

```
three-column layout, full width, all monospace at 12px

col 1 (240px fixed) - run index + summary
  run list (scrollable)
    each row: repo, run id, current stage, issue progress, blockers, duration
  sort by blockers descending for triage
  click to scope col 2 to that run

col 2 (flex) - run timeline
  sticky column headers
  chronological event stream for the selected run:
    stage transitions, issue starts/completions, artifact creation,
    checkpoint merges, gate verdicts
  columns: TIME | TYPE (badge) | EVENT (with tree indent) | STATUS | ACTOR
  tree indent shows hierarchy: run > stage > issue
  sortable by any column
  filterable by entity type and status

col 3 (right panel, expandable) - entity inspector
  click any event in col 2 to open detail:
    stage record: dependencies, input/output artifacts, summary
    issue record: acceptance criteria, branch, verification requirements
    artifact record: type, version, lineage, payload (JSON viewer)
    gate record: verdict, evidence, blocking items
    checkpoint record: branch, merged issues, conflicts, head sha
  close button returns to log-only view
```

## Repo/Run List (Landing Page)

Derived from the Gauntlet board list (dashboard) pattern: a grid of cards with
just enough visual signal to orient the operator.

Layout:

```
grid: repeat(auto-fill, minmax(275px, 1fr))
responsive: 3x5 at full width, collapses to fewer columns
```

Each card represents one active factory run:

```
repo name                          status indicator
feature / brief label              (dot: green/amber/red)

stage progression
  [ Planning | Implementing | Verifying | Gated | Completed ]
  current stage highlighted, completed stages filled, future stages hollow

issue progress (compact)
  8/14 issues passed

duration or "live"
```

No phase columns, no issue squares, no lane layout, no decorative elements.
Cards answer one question: where is this run in the pipeline.

Behavior:

- Active runs sort to top, grouped by repo if multiple runs per repo
- Failed/blocked runs surface with red status indicator
- Completed runs fade or sort to bottom
- Click a card to enter the trace-shell Run Workbench for that run

## Entity Type Mapping (Trace Shell)

Each factory entity type maps to a prefix badge in the event timeline, following
the Gauntlet pattern of single-letter color-coded badges:

| Entity     | Badge | Color         | Examples                              |
|------------|-------|---------------|---------------------------------------|
| Stage      | S     | #5b9bd5 blue  | stage started, stage passed           |
| Issue      | I     | #e6a23c amber | issue created, issue running, failed  |
| Artifact   | A     | #4ad981 green | artifact created, schema validated    |
| Gate       | G     | #2ec4b6 cyan  | gate checked, verdict pass/fail       |
| Checkpoint | C     | #ff69b4 pink  | checkpoint advance, merge, conflict   |
| Room       | R     | #888888 grey  | room created, actor joined, message   |

Status badges (right column):

| Status  | Label   | Color         |
|---------|---------|---------------|
| Pending | pending | grey          |
| Ready   | ready   | grey          |
| Running | run     | #e6a23c amber |
| Passed  | pass    | #4ad981 green |
| Failed  | fail    | #ff69b4 pink  |
| Blocked | block   | #ff69b4 pink  |

## Run Timeline Hierarchy

Events in the trace-shell Col 2 use tree indent (via parent relationships) to
show structure without requiring separate navigation:

```
12:04:01.332  S  stage:interpret started                    run    orchestrator
12:04:01.440  R    room factory/42/interpret created        run    orchestrator
12:04:01.512  R    actor joined (claude-opus)               run    orchestrator
12:04:08.891  A    artifact:interpreted_problem created     pass   claude-opus
12:04:09.002  S  stage:interpret passed                     pass   orchestrator
12:04:09.015  S  stage:resolve started                      run    orchestrator
  ...
12:05:44.200  I    issue:implement-auth-handler running     run    claude-opus
12:05:58.100  A      artifact:delivery_commit created       pass   claude-opus
12:05:58.300  I    issue:implement-auth-handler passed      pass   claude-opus
12:06:01.000  C  checkpoint:advance merging                 run    orchestrator
12:06:01.500  C  checkpoint:advance ready                   pass   orchestrator
12:06:02.000  G  gate:merge_readiness checked               pass   orchestrator
```

This is a flat, scrollable, chronological log with visual nesting — not a tree
widget. The same pattern that Gauntlet uses for frame parent/child display.

## Ops Surface

System health does not need its own layout shell. It is either:

- A trace-shell instance filtered to transport/gate events instead of factory
  events (connection status, room lifecycle, transport errors)
- A summary panel accessible from the top-level nav

Contents:

- Gate/server connection status
- Active rooms (count + list)
- Active runs (count + summary)
- Recent transport errors or degraded states
- Uptime / last restart

## Repo Metadata

Repo-specific information (import status, sync state, publishable branches,
linked rooms, linked runs) does not need its own layout shell. It lives in:

- The room console right rail (tab: repo metadata) when viewing a repo room
- The run workbench inspector (col 3) when inspecting a run linked to a repo
- The card grid, where repo name is the primary label on each run card

## What This Design Removes

Compared to the current prior-web UI:

- No serif headline typography
- No marketing taglines or descriptive prose in the UI
- No illustrative/placeholder data lanes
- No rotated pinned-note card styling
- No glass morphism or backdrop blur effects
- No `clamp()` hero text sizing
- No "blueprint" visual metaphor
- No landing page with value proposition copy

The landing page is the card grid. The card grid shows real runs. If there are
no runs, it shows an empty state with an action to start one.

## Implementation Sequence

1. Replace the landing/home page with the card grid (run list)
2. Build the trace-shell layout (three columns, sizing, scroll behavior)
3. Wire Col 1 to factory:run:list + factory:run:status
4. Wire Col 2 to factory:stage:list + factory:issue:list + factory:artifact:list
   for the selected run, merged into a chronological timeline
5. Wire Col 3 inspector to entity detail views
6. Build the board-shell layout (toolbar, rails, central workspace, status bar)
7. Wire the board-shell to room:join + room:message + room:history
8. Add top-level nav (Runs / Rooms / Ops) and routing
9. Adapt Gauntlet CSS tokens (colors, spacing, typography) to prior-web
