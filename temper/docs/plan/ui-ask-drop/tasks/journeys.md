+++
id = "journeys"
part = "stock"
kind = "migration"
needs = ["stock", "client-names", "gallery", "help"]
files = ["temper/docs/design/client/journeys/**", "temper/docs/design/client/tui/journeys/**", "temper/docs/design/client/journeys.status.md", "temper/docs/design/client/tui/journeys.status.md"]
effort = "medium"
minutes = 75
+++

# journeys: the journeys' recordings are made again without template 3

## Goal

Five committed journeys recorded a stock that held template 3, the configuring chat:
their recordings embed its template row and the inventory's configuring lines, though
none walks the old Configure page. Once the stock holds no template 3, this task records
them again with `temper-client-journeys again NAME`, generation and not a walk
(`notes/common.md`), so that the gate's focused replay reads what temper now serves.

Acceptance:

- `temper/docs/design/client/journeys/{welcome-set-up,welcome-playground,project-landing,chat-draft}/`
  and `temper/docs/design/client/tui/journeys/sign-in-approved/` are recorded again; no
  recording there, decoded, holds a template 3 row, `configuring` as a template's name or
  a charter 3 the stock offered.
- Each journey's prints differ from the base's only where the stock's change shows (a
  template list, a count) or where `temper-client-journeys again` always moves (times,
  identifiers it says it renews); the report names each print's change.
- `heavy cargo nextest run -p temper-client-journeys --lib -E 'test(=tests::the_committed_journeys_are_their_print)'`
  passes.
- These status entries are deleted whole, and no other status line (a stated departure
  from the implementer's text, `notes/common.md`): in
  `temper/docs/design/client/journeys.status.md`, the **Not yet** item beginning "9: the
  recordings of `welcome-set-up`, `welcome-playground`, `project-landing` and
  `chat-draft` were made on fake boxes whose stock held template 3"; in
  `temper/docs/design/client/tui/journeys.status.md`, the bullet beginning "**Not yet:** 9:
  `sign-in-approved`'s recording was made on a fake box whose stock held template 3".
  If the first leaves its **Not yet:** with no item, the label goes too.

## Inputs

- `temper/docs/plan/ui-ask-drop/notes/common.md`: journeys' generation, the three retries
  on "a key mark caught a moving page", each on a fresh fake box, then a widened wait.
- `temper/docs/design/client/journeys.md` (how a journey is recorded and replayed) and
  `temper/docs/design/client/tui/` for the TUI's.

## Owns and may change

- Owns the five journeys' directories above: their `recording` and prints.
- `temper/tools/journeys/src/**`: only a key mark's wait, widened as `notes/common.md`
  allows after three failed retries, named in the report.
- Run from `temper/tools/journeys` as its README says
  (`cargo run -p temper-client-journeys -- again NAME`, through `heavy`, with
  `TEMPER_BUNDLE` as the tool sets it and Chromium where the builder has it). If the
  builder has no Chromium the journey's recording needs, stop and report `Question:`
  with what is missing; never hand-edit a recording.
- Any other journey whose recording the replay finds stale is recorded again the same
  way and named.

## Steps

1. Decode each of the five recordings (read-only) and confirm the template 3 rows the
   survey found; note any other journey whose recording names template 3.
2. `again` each journey, one at a time; after each, the replay test above.
3. Commit ("Record the journeys again without the configuring chat") and push.

## Report

Each journey recorded, its tries, its prints' changes; any wait widened; the replay
test's result.

## Answers

2026-11-08 (temper/ui). Neither as asked: first find which it is, from your scratch directory /tmp/temper-journey-2572037-0, its last still save, without a new walk. (1) If the goal page has its Draft return link, but to another chat than task 2, task numbers moved when the configuring chat stopped being made. Then the walk changed, and the recording isn't the product's fault. Leave chat-draft's recording and prints at the base. Run its focused replay. If that passes, add one line to the browser journeys' status: chat-draft is to be walked anew, as its task numbers moved with the configuring chat's removal. Then finish: I'll have it walked anew after landing. (2) If the goal page has no Draft return link at all, it's this plan's regression. Repair it in its owning code under a failing focused test first, then run again chat-draft once. (3) If the replay fails in case (1), stop with Question: and its first differing step. Say in the report which case it was, with the still save's evidence.
