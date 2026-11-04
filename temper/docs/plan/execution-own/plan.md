# execution-own: the plan

From `inputs/request.md` (survey/agents-refs, step 11 of 12, approved by the user): temper's drafts
say what, not who runs it. Documents only, two direct tasks. `notes/common.md` holds the rules both
name: the departures from the implementer's text, what is never touched, and which words go and
which are the product's own.

## Parts and tasks

1. **notes-and-example** (`temper/docs/design/draft/notes.md, 4`), slice 1.
   - `tasks/notes-and-example.md`, high, ~40 minutes, needs nothing. `notes.md` loses sections 2 and
     3, and every N item keeps its choice with a reason of its own. The sentences that cite the
     evidence in `reference-landing.md`, `reference-case.md` and `reference-code-production.md` are
     fixed. `dashboards.md`'s example becomes an unnamed example project's status page.
2. **execution-lines** (`temper/docs/design/draft/reference-case.execution.md, 2`), slice 2.
   - `tasks/execution-lines.md`, medium, ~30 minutes, needs `notes-and-example`. Both execution
     files lose who and where, the two status lines lose their name, and the six entries leave
     `tests/boundary/outward.txt`.

**Critical path:** notes-and-example, then execution-lines, about 70 implementer minutes.

## Why a chain

An entry can leave `tests/boundary/outward.txt` only once its file holds no names, and the six
entries are adjacent lines, so two tasks each removing their own would conflict. One task takes
them all, last. A count may fall below its entry, so `notes-and-example` stays green without
touching the file. A third, thin task for the inventory would let the two run side by side, but the
request asks for one or two tasks, and the wait is short.

## What was decided

- **`notes.md` keeps its numbers:** 1, 4, 5, 6 (`DESIGN.md`, 4). No later section moves, so only
  citations of 2.x need fixing. Those are three sentences in `reference-landing.md` (one of them
  section 9's opening) and one in `reference-case.md`. The header sentences of
  `reference-landing.md` and `reference-code-production.md` that say "evidence" change with them.
- **Three sentences use "the evidence" or "the run" without citing `notes.md`:**
  `reference-code-production.md`, section 17, twice, and `reference-landing.md`, section 12. They
  would dangle once the evidence goes, so they are fixed too.
- **N2 takes in the rules section 2.1 held,** which no other part of `notes.md` states.
- **N11 keeps "about 170" wakes,** stated at the reference's size (N21): it is the cost the choice
  weighs, not a record of a run.
- **`reference-case.execution.md`, section 2 stays, shortened, not deleted:**
  `temper/docs/plan/reference-walks/plan.toml` names it as its `happy_path`, and
  `configuring.execution.md` cites it. Its walk is there.
- **Every `Old code` bullet and the old-branches paragraph go.** The branches are in repositories
  on a lab machine, so a name alone leads a reader nowhere. It is not what a milestone delivers, and
  the lab keeps its copy.
- **`status-page.md` stays as the example's file name,** so `tools/citations/allow.txt`'s dashboards
  entry stays: the draft reads the document by its path (`dashboards.md`, 6.1).
- **`notes.md`'s `status.md` entry leaves `allow.txt`:** its only target was in section 3. The request
  did not name it.
- **`dashboards.md` has two counted lines the request's list leaves out:** `hetzner3` at l.236 and
  the status host at l.422. The first spec names both.
- **The example's page uses the draft's words,** so "missions", "the director's post" and
  "implementer runs" go from the mapping table, not only the names the check counts.

## Overlaps

- **Could not be avoided: `temper/docs/design/client/browser.status.md` and
  `temper/docs/design/playground.status.md`, with `playground-browser`.** The request needs one line
  in each, and that trunk changes the same lines: it removes the name from `browser.status.md`, and
  rewords the line in `playground.status.md`, keeping the name. They come last in this plan, and
  `execution-lines` pre-approves the case where `playground-browser` lands first. If this plan lands
  first, `playground-browser`'s merge with main conflicts on those two lines, and its resolution has
  to drop the name, or the outward check goes red there, the entry being gone.
- **None other:** no trunk open at this cut changes any other file of this plan.

## Left for the user to know

- `notes.md` has no N18 and no N24, though four documents say "N1 to N24". The request says to keep
  the numbers, so the range stays as written.
- `temper/docs/development/`'s three entries stay in the inventory, as the request's bounds say.
