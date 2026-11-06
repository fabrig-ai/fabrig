# ui-walk-3: the plan

The whole client walked again now that everything has landed, as the request says: `temper/docs/plan/ui-walk-3/inputs/request.md`. No design is written: this plan compares what's built with the mock and with the second walk, and fixes what's small. Decisions and rulings are carried in `notes/decisions.md`; the walk's method in `notes/walk.md`; the rules in `notes/common.md`.

## The tasks

| # | Task | Needs | Effort, minutes | What it gives |
|---|---|---|---|---|
| 1 | `walk` | | high, 120 | `report.md`, `findings.md`: the seven steps, W3, W6 and its unfunded path |
| 2 | `additions` | | high, 120 | `additions.md`: A4 ten times, rename and the Owner menu, the two scroll bars, `chat-work`, `close-chat`, the sign-in and model-check looks, temper's own way |
| 3 | `real-login` | | medium, 40 | `real-check.md`: R1 on a real box |
| 4 | `fix-look` | 1, 2, 3 | high, 75 | the small findings in the sheets and the view, each red first |
| 5 | `fix-flow` | 4 | high, 75 | the small findings elsewhere, and the real check's |
| 6 | `walk-again` | 5 | high, 75 | the walk over what the fixes touched; the final `report.md` and `findings.md`; the list for `fabrig/ui` |

## The critical path

`walk` or `additions` (120) → `fix-look` (75) → `fix-flow` (75) → `walk-again` (75): 345 implementer-minutes, then the trunk's end check. `walk` holds a real wait of at least 31 minutes.

## What the cut decided

- **Three walks at once, not one long one:** the seven steps with W3 and W6 (`walk`); what the second walk never held (`additions`); the real command (`real-login`, which needs a different build). None waits for another. Each writes its own file, so they never meet: `additions.md` and `real-check.md` are folded in at the end.
- **The driver, the second walk's files and the decisions are committed in this directory,** since an implementer's builder can't read where they were kept, as the request allows. They leave main with the plan; neither the outward check nor any test reads them. `driver/focus.py` comes from the first walk's plan in git history, since its `README.md` names it and `additions` starts from it. The screenshots stay on the builders.
- **The fix tasks' rows don't exist yet.** A pushed spec never changes, so `fix-look` and `fix-flow` read the rows from the trunk, by owner, and each ends with a commit, even with nothing to fix.
- **Two fix tasks, chained,** since they'd meet in `temper.css`, the gallery, the goldens and the prints: the sheets and the view first, whatever's deeper second.
- **The walk once more goes only over what the fixes touched,** as the request asks; W6's 31 minutes and A4's ten repeats come back only if a fix touched them. With no fix it writes the result from what's found.
- **Finding ids don't clash across parallel writers:** `N` for `walk`, `M` for `additions`, `R2` on for `real-login`, `B` for `walk-again`; W3, W6, A2, A3, A4 and R1 keep theirs.
- **W3 goes on its own box, before any setting changes:** the fake's cue proposes against a fresh box's newest revision.
- **W6's unfunded path, default:** a second member, since a plan serves only a sole member's project; then a period set down through ctl. A state a person can't reach is itself a finding.
- **`backends.rs` has one writer in the first wave, `walk`,** since `model_document` is one long line, and two scripts would conflict. `additions` shows by the gallery what needs a new script.
- **What's owed is named, not a finding:** `chat-reading`, `sign-in-new-tab` and `model-check`'s status files say what.
- **No new browser-tier test** without measurements: fixes are red in lower tiers.
- **R1's departure from `AGENTS.md`, 3** is the pinned download alone, as the request names it. No credentials and no sign-in.

No stored format, protocol or public interface changes by this cut. A fix that changes one says so in its report, with no compatibility, and a wire change re-records its journeys in that task.

## Overlaps with the drafts beside this one

- **`walk`, `additions`, `real-login`** touch only this directory, and `walk` perhaps `temper-fake/src/backends.rs`: nothing another draft lists.
- **`fix-look`:** `chat-reading-c` and `chat-reading-b` list `temper-client-view/src/pages/{chat,chat_line,agent}.rs`, `limits.rs`, `temper.css` and the journeys' prints. It can't be avoided, since a fix goes where the finding is. The fixes come after the walks, so those plans have the time to land first, and the request's rules on `temper.css`'s markers and on regenerating prints keep a meeting small.
- **`fix-flow`:** `sign-in-new-tab` lists `temper-client-domain/src/pages/accounts/request.rs` and the client wire; it only meets them if a finding does, and its owed behaviour isn't a finding.
- **`real-journeys`' merged work** is in `temper/tools/journeys` and the shell's `journey/`: a regenerated print meets it only through the tool.
- **`configuring-walks`, `reference-walks`, `v0.1.0-ladder`** are queued with no tasks listed: unknown.
