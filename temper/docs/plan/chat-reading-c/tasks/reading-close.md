+++
id = "reading-close"
part = "close"
kind = "docs"
needs = ["reading-worlds", "reading-browser"]
files = [
  "temper/docs/design/client/README.md",
  "temper/docs/design/client/README.status.md",
  "temper/docs/design/client/journeys/**",
  "temper/docs/design/client/journeys.md",
  "temper/docs/design/client/browser.md",
  "temper/docs/design/client/mockups/STATUS.md",
  "temper/docs/design/draft/chat-reading.status.md",
  "temper/docs/design/STATUS.md",
  "temper/tools/gallery/**",
]
effort = "medium"
minutes = 40
+++

# Design agrees with what's built, and the journeys that moved are named

## Goal

Reprint what the plan moved, say which journey moved for fabrig/ui, and make the design say what's true. **Departure from the implementer's text:** this task edits the design files in its header, and `look-round-6` having landed is what lets it: 5.15 went in as new text so as not to meet that plan's hunks, and the lines it leaves stale are edited here.

**Your base must hold `look-round-6`:** this plan stands on it, and `fabrig/ui` has priority. Check first: `rg -n 'State::ToLogEnd' temper/crates/temper-client-view/src/pages/chat.rs`. If that finds nothing, change nothing and report `Question: look-round-6 has not reached my base; wait for the catch-up?`, recommending yes.

Acceptance:

- **Prints:** `heavy cargo run -p temper-client-journeys`, the gallery and the goldens are reprinted, and the diff is explained by this plan: a page's "Older" gone, a log's states, a terminal's line.
- **Journeys:** a recording whose walk pressed "Older", or whose exchange with the engine moved since a chat no longer reads a second page by itself, is stale (`temper/docs/design/client/journeys.md`, section 8). Recording one again is a user's walk or `again NAME`, and neither is done here unless `again` holds: try it for each. Your report names each journey that moved, what moved, and whether it holds now, **for fabrig/ui**. `chat-draft/21.html` and `22.html` hold "Older" today.
- **`temper/docs/design/client/README.md`:** 3.5's *Reading back*, *"Loading earlier messages…"* and *"Older"*, the table row for `Task { task }`, 5.4's window of 16 lines, 5.9, 5.11's *"Older,"*, and section 10's and section 11's mentions, each now says what 5.15 says, or cites it. Nothing renumbered.
- **`temper/docs/design/client/README.status.md`:** the line beginning `5.15:` is gone.
- **`temper/docs/design/draft/chat-reading.status.md`:** section 5's *Design lags* entry is gone, and a *Design departs* entry for section 5 holds, by the plan `chat-reading-c`: *Keys, in the terminal* (no keys: lines, with Enter); *A window bounded by size* (`lines` kept for jig's reference UI and tools, past what temper's bytes hold); *In every browser* (the tier runs Chromium alone: the rest is read). `STATUS.md` follows, as `DESIGN.md` says.
- `python3 -I -B tools/citations/citations.py --base origin/main` is clean.

## Inputs

- `temper/docs/design/client/README.md`, 5.15.
- `temper/docs/design/client/journeys.md`, sections 5, 8 and 9.
- `DESIGN.md`, whole, as whoever writes design does.
- The plan's `temper/docs/plan/chat-reading-c/plan.md`, *What didn't hold in the request*.
- `temper/docs/plan/chat-reading-c/notes/common.md`.

## Owns and may change

Owns the files in the header. `jig/docs/design/client/domain/view.md` and `temper/docs/help/` stay.

Options, each decided:

- **A word of 5.15 that differs from what was built** is fixed to what was built, and named. A rule that differs is a red: fix what's wrong in its lowest tier if it's one hunk; otherwise a `Question:`.
- **A stale recording `again` can't hold** stays stale, named: fabrig/ui has priority over it, and no person walks it here.
- **`temper/tools/gallery/names.txt`:** no state is added. A sample that showed "Older" reprints without it.

Not this task: new behaviour.

## Steps

1. Reprint; read the diff by `git diff --stat`, then the hunks in question.
2. `again NAME` where it applies.
3. The design's words, as above; `chat-reading.status.md`.
4. `python3 -I -B tools/citations/citations.py --base origin/main`; `heavy cargo nextest run -p temper-client-journeys -p temper-client-gallery`. Commit and push.

## Report

As the implementer's text asks, and **the journeys that moved, for fabrig/ui**, each with what moved. Then what the user reads on a real chat: this plan's `walk` is false, as the user reads it.

## Rebase (2026-11-06, step merge, foreman chat-reading-c/reading-close/foreman-1)

reconcile reading-close with main’s newer client design and regenerate journey prints
The merge failed on documentation and generated `chat-draft` HTML; this is a branch conflict, not an infrastructure failure.
Rebase onto current main, preserving its unrelated changes throughout the conflicting files.
In `temper/docs/design/client/README.md`, retain Pause/Resume/Close from `17327a3918` and composer-focus rules from `cbb15d02a5`; apply this task’s edge-reading corrections within those newer paragraphs.
Preserve agent Latest from `68dc9aeb23` and chat-reading-b’s design. Reconcile this branch’s stale “agent has no pill” statements, including section 5.15, with the combined implementation.
In `README.status.md`, remove only the task’s completed 5.15 gap; retain main’s other findings and completed removals.
In `draft/chat-reading.status.md`, preserve main’s section 2 departures and section 7 wording, remove section 5’s Design lags entry, and retain this branch’s three section 5 Design departs entries.
Merge `temper/docs/design/STATUS.md` accordingly, preserving main’s other status updates.
In `client/journeys.md` and `client/mockups/STATUS.md`, retain main’s newer journey/setup and model-check wording while applying the no-Older, edge-reading corrections. Keep main’s numbered-list status citation.
Main refreshed `chat-draft` in `cdfa781cb4`, added setup-based recording in `8a332171b3`, and refreshed it again in `37558b97dd`; preserve those recording/setup sources.
Merge `journeys/chat-draft/README.md`: keep main’s setup-based provenance and this task’s description of edge loading and the returned chat’s measured window.
Resolve conflicting `chat-draft/08.html`–`18.html`, `21.html`, and `22.html` by regeneration, not hand merging (`client/journeys.md`, 5.2); regenerate the automatically merged prints too.
Run `heavy cargo run -p temper-client-journeys`; for each stale fake journey, try `again NAME` as specified. Report any refusal and each journey’s changed behavior for fabrig/ui.
Regenerate any gallery or golden outputs made stale by the combined code using their existing tools; these remain this task’s responsibility.
Review the automatically merged `client/browser.md` for agreement with both changes.
Run the spec’s citation check and focused journey/gallery tests through heavy, then commit and push under the new rebase branch name.
