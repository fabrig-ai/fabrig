+++
id = "landed-ci-kept"
part = "landed-ci"
kind = "feature"
needs = []
files = [
  "temper/crates/temper/tests/live.rs",
  "temper/crates/temper/tests/support/first_repository.rs",
  "temper/docs/design/domain/testing.status.md",
]
effort = "medium"
minutes = 45
+++

# The owned-forge landing stories read the landed change's CI the restart keeps

## Goal

Both owned-forge landing stories land their change on real models, restart the box, and
then fail at `temper/crates/temper/tests/live.rs` ~35 (`ci_shown`): after the restart they
expect `CI at its head\nNot read since the restart`, and the change's page shows `Passed`.
The page is right: since 10-27 a landed change keeps its last verdict at its head with its
record through a restart (`temper/docs/design/domain/forge.md, 8.8`), and the process
tier's restart worlds already read `Passed` on both runs
(`support/first_repository.rs`, `assert_task`). Bring the live stories to the design, through
one check both the live stories and the process worlds read, so the two cannot drift again.

Acceptance:

- `support::first_repository::assert_landed_ci(run, page)` asserts `CI at its head\nPassed`
  on the change's page; `assert_task` ends by calling it, in place of its own assertion.
- `live.rs` has no `ci_shown`; `owned_forge_landing` calls `assert_landed_ci` on both runs.
- `heavy cargo clippy -p temper --all-targets -- -D warnings` is clean (it compiles the `live`
  and `process` binaries); temper's focused tests pass; the process world
  `a_persons_plan_change_lands_on_the_owned_forge_and_is_found_after_a_restart` passes,
  reading `Passed` on its first and second runs through the new function; every
  `forge_land::` test of `temper`'s `suite` binary passes.
- `temper/docs/design/domain/testing.status.md` no longer holds this plan's 2.7 entry
  (below).
- No live story is run (`notes/common.md`); fab's after-landing pair is the proof, after the
  merge.

## Inputs

- `temper/docs/plan/live-temper-stories-2/notes/common.md`.
- `temper/docs/design/domain/forge.md, 8.8`: the **CI at its head** bullet.
- `temper/docs/design/domain/testing.md, 2.7`: the owned-forge landing story's bullet, the
  sentence beginning "The change's page says "Passed" for CI at its head on both runs".
- `temper/docs/design/client/README.md, 5.10`: the **Gates at its head** bullet (what the
  page says for CI on a landed change; read only).
- `temper/docs/plan/live-temper-stories-2/inputs/request.md`.

## Owns and may change

Owns, in `temper/crates/temper/tests/`:

- `support/first_repository.rs`: the new `assert_landed_ci`, placed just after
  `assert_task`, and `assert_task`'s last line (~758) alone, which becomes the call.
- `live.rs`: `ci_shown` and its doc comment (~31-36), removed; its one call site in
  `owned_forge_landing` (~124), which becomes the call. Nothing else of the file.
- `temper/docs/design/domain/testing.status.md`: in the **Not yet** list, remove the
  sentence `2.7: after the restart, the owned-forge landing pair expects "Not read since the
  restart" for CI at the landed change's head, not the "Passed" its record keeps
  (`crates/temper/tests/live.rs`, `ci_shown`).` whole, so the entry before it ends at
  `(`crates/temper/tests/live.rs`).` and `4.3: the shell subset;` follows. Nothing else. This
  is the task's one departure from the implementer's text ("Not yours: any `docs/design/`").

Read only: `process.rs` (its two restart worlds call `assert_task`, ~566 and ~688:
unchanged), `support/forge_land.rs` (`LiveEvidence::page`, the change page the live judge
reads).

Pre-approved options:

- If the process world refuses to start for want of `TEMPER_FORGEJO` or
  `TEMPER_FORGEJO_RUNNER`, or fails at a line before `assert_task` (a step this task cannot
  reach), read its log once, run it once more alone; then report it under `## Left` with both
  logs' paths, the machine and the failing line. Done stands on the other acceptance.
- If it fails inside `assert_landed_ci` (the page shows other than `Passed` after the
  restart), that contradicts `forge.md, 8.8`: stop, push what you have, and ask, with the page
  the log prints.

## Steps

1. `support/first_repository.rs`. Sketch (the body is one assertion):

   ```rust
   /// CI on temper's change page: passed at the landed change's head, at the
   /// landing and after a restart, which keeps it with the change's record
   /// (domain/forge.md, 8.8).
   pub fn assert_landed_ci(run: &str, page: &str) {
       assert!(page.contains("CI at its head\nPassed"), "{run}: CI at the landed change's head: {page}");
   }
   ```

   `assert_task`'s last line becomes `assert_landed_ci(run, page);`.

2. `live.rs`: delete `ci_shown`; in `owned_forge_landing`, `ci_shown(&evidence.page, first);`
   becomes `support::first_repository::assert_landed_ci(run, &evidence.page);` (`run` is the
   loop's `"first"` or `"second"`).

3. The status sentence (Owns).

4. Tests, one at a time:
   - `heavy cargo clippy -p temper --all-targets -- -D warnings`;
   - the process world: `tools/tiers/run temper process -- -E
     'test(=a_persons_plan_change_lands_on_the_owned_forge_and_is_found_after_a_restart)'`
     (it builds the tier's prerequisites, then nextest through heavy); check it selected one
     test;
   - the harness: `heavy cargo nextest run -p temper --test suite -E 'test(/^forge_land::/)'`;
   - `heavy cargo nextest run -p temper` (the focused profile: never the live binaries).

5. `cargo fmt`; from `temper/`, `python3 -I -B scripts/old_forms.py` (from the root it
   finds no `scripts/old-forms.txt`); after committing, from the root,
   `python3 -I -B tools/citations/citations.py --base origin/main`.

6. Commits: the shared check and the stories' call; the status sentence. Push your branch.

## Report

As the implementer's text says, and also: the process world's wall time and its two runs'
`CI at its head` lines from its log (`eprintln!("{run} change page: ...")`), the `suite`
count, and the focused count.
