# ui-p3b-reaches: plan

Milestone P3b of `temper/docs/design/draft/whole-story.execution.md`
(ranked nice): How work runs' What reaches you, as
`temper/docs/design/draft/surfaces.md`, section 7, and the whole story's
`how.html` draw it, on N1's classes and Away. Its design is
`temper/docs/design/client/README.md`, 5.13, *What reaches you*, *Its
parts* and *Its rows*, and 3.5, *Away*'s *On How work runs*, with
`browser.md`, 2.5, *How work runs' look*, and 7.2's states, all written by
this cut. `notes/common.md` holds the rules every task follows;
`notes/contracts.md` the shapes that cross tasks.

## What was decided

- **Three columns of words, from kind 8 alone:** "Agents decide, then
  tell you", "Asks you" and "Always waits for you", each a box with who
  acts' chip, a line a subject and a small line; a column with no line
  isn't drawn, so You approve's table shows no "Agents decide". The lines
  are `how.html`'s words, one per kind, a set "a change to NAME", an item
  with words its subject's line "under “WORDS”". `other`, which always
  waits and no mockup lists, is listed too ("anything an agent can’t
  place"): the page draws what the engine holds.
- **What Away changes** is the "Asks you" box's small line: the subjects
  that ask even when away stand first, then those Away may decide, and
  the line says "they are decided…", "They ask even while you’re away."
  or "all but OBJECTS are decided…", naming the exceptions as Stop asking
  names subjects. No count in words ("the last two"), which a table of up
  to 32 subjects can't say well.
- **No control of kind 8 on the page:** What reaches you takes the place
  of *What needs you* and *While you're away, agents may decide*, whose
  checks an owner ticked. A class changes by proposal, as N1 built it:
  Ask temper's ("Ask temper to change this"), or a need's Stop asking,
  which an owner may apply (the request: "Changing a class goes by
  proposal, as N1 built it"). The mockup draws no control there. The
  checks' code (`forge::classes::{asking, turned}`, `WaysRow::Classes`)
  leaves with them.
- **Two rows stay, under the columns:** kind 6's "Questions a goal can
  answer from its plan", which is what reaches the person of a goal's
  questions, and "Away until", the person's own choice of the spell's
  end, the only place "I turn it off" is chosen. Neither is a class.
- **The pairs and team mockups' other lines** (a landing rule's approvals,
  "a change's questions", an agent's own repairs) are no class: an
  approval is the path's stop and Rules by path's (P3c), the question
  route the section's row. Recorded as a departure in
  `surfaces.status.md`, 7 and 13.
- **Its place:** after the path, before Roles, as all three mockups order
  it; "Limits" focuses its heading now.
- **The head's right** says "on PROJECT; each project has its own" when
  the person has two projects or more, else "on PROJECT".
- **One model, two pure functions:** `pages::how::parts` gains `reaches`
  (kind 8 read) and `several`; `pages::how::reaches` lays the table's
  subjects in the settings order the proposal card already uses. The web
  and the terminal share them; the TUI draws no How work runs.
- **Before the read answers,** and after a lasting refusal, the section
  holds its head and its rows, no columns: no stock drawn as if it were
  the project's.
- **The gallery:** `how-reaches` (the at-scale table of `how.html`, a
  path set asking even when away) and `night-how-reaches` take the place
  of `settings-away` and `settings-away-you-approve`, whose section goes,
  so the over-budget gallery prints no more states than before; `how` and
  `how-approve` hold their columns, `how-approve` reading You approve's
  table.
- **Shared files with `ui-n3a-only`** (cut beside this one): each task
  names its insertion anchors in the How work runs neighbourhoods, and the
  view's new look slots reuse the reserved run 681 to 685 so that neither
  plan appends at `look::NAMES`' end; the words take ids 5630 to 5651.

## Parts and tasks

| Part | Task | Needs | Crates | Minutes |
|---|---|---|---|---|
| `model` | `reaches-model` | | `temper-client-domain` | 50 |
| `fuzz` | `reaches-fuzz` | `reaches-model` | `temper-client-world` (`fuzzy/how.rs`) | 60 |
| `page` | `reaches-page` | `reaches-model` | `temper-client-view`, `temper.css`, the terminal's golden; the checks' removal in `temper-client-domain` and the gallery | 180 |
| `page` | `reaches-gallery` | `reaches-page` | `temper-client-gallery` | 90 |
| `story` | `reaches-story` | `reaches-page` | `temper-client-world` (`away.rs`) | 50 |

Released in the first push: `reaches-model` and `reaches-fuzz`; in the
second, `reaches-page`, `reaches-gallery` and `reaches-story`.
`reaches-fuzz` and `reaches-story` both write `temper-client-world`, on
files of their own (`fuzzy/how.rs`, `away.rs`), and seldom at once:
`reaches-story` waits for `reaches-page`.

## Critical path

`reaches-model` (50) → `reaches-page` (about 180) → `reaches-gallery`
(about 90), about 5 hours 20 minutes of implementer time; `reaches-fuzz`
runs beside `reaches-page`, and `reaches-story` beside `reaches-gallery`.

## Who hears first

- **`temper-client-domain`, for the TUI's owner (temper/tui):**
  `pages::how::Facts` and `Parts` each gain two fields (`classes`,
  `several`; `reaches`, `several`), and `pages::how` gains `Line`,
  `Reaches` and `reaches` (`notes/contracts.md`, section 1). No existing
  fact or part changes meaning. `forge::classes`, Settings' checks'
  helper, goes with the checks (`reaches-page`).
- **`temper-engine-domain` and `forge_route.rs`:** no change.

## Overlaps

- **`ui-n3a-only`** (Only you, cut beside this one) writes
  `temper-client-view`'s `words.rs`, `look.rs`, `temper.css` and the
  gallery's `samples.rs`, `tests.rs` and `names.txt`, and the design's
  `README.md`, 3.5, `README.status.md`, `browser.md`, 7.2, and
  `surfaces.status.md`. This plan keeps to How work runs' neighbourhoods
  in each: words after `HowRoleInstructions`, look slots 681 to 685, CSS
  under `/* ==== ui: how ==== */`, gallery states beside `how`; in the
  design, 3.5's *Away* and 5.13, and `surfaces.status.md`'s 7 lag and a
  departure after `ui-p3f-design`'s. Unavoidable: `browser.md`, 7.2 loses
  its `settings-away` rows among the Away rows, near Needs you's, and
  `samples.rs` loses `settings-away`'s two samples (`reaches-page`).
- **Drafts beside this one** (`worker-link-quiet`, `resume-fits-room`)
  write `temper-service` and jig's worker crates: no file in common.
