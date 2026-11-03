# debt-1: the plan

From `inputs/request.md` (fabrig/debt, 2026-11-03): four small debt items, one per tree, each
paid on its own branch so that its **Paid when** holds. The four read none of each other's
code, so each is one part, one slice and one task, all released at once with this cut's first
push. `notes/common.md` holds the standing rules every task names.

## Before the tasks

This cut wrote the design each task follows, pushed with the specs:

- **skein, `examples.md` 3.4:** the echo connection's states gain AdmittingEnded,
  AnsweringEnded and Shut, and Closing and Shut hold what the domain still holds (its call
  out, the session owed `Gone`, or nothing) in place of the `ended`, `closed` and `call`
  flags; a new "The layer's stop" gives its two machines, one per lifecycle
  (programming-model.md, 5.4): the shutdown (Running, Asked, Told) and the connections' stop
  (Serving, Draining, Aborting), in place of `stopping` and `aborting`. `examples.status.md`
  records it as **Not yet**.
- **smith, `domain/session.md` 3:** the opener names in its opening the one call it may have
  withheld, and the session restores that call alone as withheld. `session.status.md` and
  smith's `STATUS.md` record the gap the debt item found unrecorded.
- **temper, `protocol/setup.md` 5.3:** reconcile's limits name no `setup.orgs`; temper gives no
  org and gives skein's orgs bound as 1, the least the kit takes (skein refuses 0). The three
  status entries D006 names already record the code's side.

## Parts and tasks

1. **skein-echo** (skein D024; `skein/docs/design/examples.md, 3.4`), slice 1.
   - `tasks/skein-echo-states.md`, high, ~75 minutes, needs nothing: `layer.rs` and `conn.rs`
     follow 3.4's states; no test file changes; the status's 3.4 words, the item and its row go.
2. **smith-withheld** (smith D004; `smith/docs/design/domain/session.md, 3`), slice 2.
   - `tasks/smith-session-withheld.md`, high, ~120 minutes, needs nothing: `Opening` gains
     `withheld: Option<Box<[u8]>>`, which the run's root always sets to its `wait`; restore
     matches it, never `b"wait"`; `Event::AnsweredAndYield` folds into
     `Event::Answered { .., after: After }`, `After` made public; the session world, the root,
     an agent-service test and one jig channel test take the literals' fallout.
3. **temper-reconcile** (temper D006; `temper/docs/design/protocol/setup.md, 5.3`), slice 3.
   - `tasks/temper-reconcile-gone.md`, high, ~120 minutes, needs nothing: the verb, its help
     row and the command table's `transition` field; the shell's `reconcile` and `exchange`
     modules and their tests; `setup.orgs`, `Host.orgs`, `Entry.orgs` and the orgs places;
     the two help pages by their tests; the three status entries, the item and its row.
4. **jig-lints** (jig D019, clauses 2 and 3; `AGENTS.md, 2`), slice 4, and **a question**
   (`d019-shared-table`).
   - `tasks/jig-lint-blocks.md`, high, ~150 minutes, needs nothing: the 40 manifests write one
     block with the replay bans on; six `jig/testing/` kits get `tests/`'s bans in their own
     `clippy.toml` instead of allowing the lints; redundant `clippy.toml` files go and the rest
     say truly why they differ; D019 stays open, retitled, for its first clause.

## Critical path

All four start at once and none waits on another: the path is the longest task,
`jig-lint-blocks` (~150 implementer-minutes, up to twice that with the buffer);
`temper-reconcile-gone` and `smith-session-withheld` (~120 each) and `skein-echo-states` (~75)
run beside it.

## Overlaps

Checked against the brief's `Drafts beside this one`, 2026-11-03:

- `skein-echo-states`, `smith-session-withheld` and `jig-lint-blocks` touch no file a draft
  beside this one is changing. `status-lines` is a cut in flight whose files are unknown; it
  may meet this cut's edits of smith's `STATUS.md` and the status files above.
- `temper-reconcile-gone` cannot avoid fabrig-scale's unmerged tasks: one line of
  `temper-service/src/limits.rs` (`brief-room`), one test's override in
  `temper-service/src/tests/startup.rs` (`init-standard`, `stock-scale`), small hunks in
  `temper/tests/suite/startup.rs` and `temper/tests/support/owned.rs` (`init-standard`,
  `reference-trim`, `stock-scale`), and one sentence each of `protocol/configuration.status.md`
  (`brief-room`, `init-standard`, `reference-trim`, `stock-scale`) and
  `protocol/setup.status.md` (`init-standard`, `measure-box`). The item names those entries
  and files, so the overlap is the item's.
- This cut's own row in `temper/docs/design/protocol/setup.md` 5.3 sits beside
  `fabrig-scale/smith-calls`'s edit of the same document, merged on its trunk.

## Calls made

- **jig D019 is paid in part, and asked.** Cargo shares one lint table among members, the
  root's `[workspace.lints]`, all or nothing (the survey ran cargo 1.95's `metadata` on a
  scratch workspace: overrides beside `workspace = true` are refused, and named tables do not
  exist), and that table is the step code's; under it the 40 crates would trip about ten
  thousand sites. So the first clause cannot hold within the request's limit (manifests,
  `clippy.toml` files and scoped expects). The task pays the other two clauses, which every
  answer needs, writes the one form a later table or check would hold, and keeps the item
  open; the question `d019-shared-table` gives the user the options, (a) recommended.
- **jig's six ordinary-Rust kits under `jig/testing/`** allowed the bans only because they
  inherit the step-code `clippy.toml`; under `tests/`'s list they use none (the survey). Each
  takes `tests/`'s bans in a `clippy.toml` of its own: a step kit beside them relies on
  `jig/clippy.toml`, so one `jig/testing/clippy.toml` cannot serve. None of the 40 uses wall
  clock or threads outside the one scoped expect already there.
- **smith D004 reads "a tool name" as an opener's.** `session/batching.rs`'s `b"job"` names
  the session's own tool, decoded by `smith-protocol-llm`, and stays.
- **The run names `wait` always,** not only when its charter grants it now, so a transcript
  parked under an earlier charter restores as today (the request: behaviour unchanged).
- **Nothing in D004 is stored or on a wire:** `record::Opening` and `Event` are in-process
  values the root builds; `smith-transcript`'s `Opening` is another type. So the request's
  stop-and-ask does not apply.
- **The echo's stop is two machines,** not one enum of their product
  (programming-model.md, 5.4: one enum per lifecycle); the closing connection's binding below
  is Closing or Shut, and its binding above what it holds. Shut ignores late stream events as
  Closing does, so behaviour is unchanged.
- **temper's `transition` field and `exchange` module go with the verb:** the field's only
  `true` row was `forge reconcile`, and its doc names it; `exchange.rs` uses
  `reconcile::Settings` alone. `Entry.orgs`, which only the shell's reconcile read, goes with
  the places; `temper-service`'s setup check stops naming `setup.orgs` for places that are not
  orgs.
- The request cites `skein/docs/design/programming-model.md`; the file is
  `skein/docs/foundation/programming-model.md`, which the specs cite.

## Noticed, outside this plan

- `temper/docs/design/draft/github.status.md`'s Design lags entry for 12.2 says `setup.md`
  keeps the standalone `temper forge reconcile` (6.5); `setup.md` names no such verb. That
  draft's status is its designer's.
