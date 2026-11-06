+++
id = "queue-merges"
part = "queue-merges"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-engine-domain-forge/src/public.rs",
  "temper/crates/temper-engine-domain-forge/src/public/landings.rs",
  "temper/crates/temper-engine-domain-forge/src/public/merges.rs",
  "temper/crates/temper-engine-domain-forge/src/tests/public.rs",
  "temper/tests/engine/forge/src/public.rs",
  "temper/tests/engine/forge/tests/suite/public.rs",
  "temper/docs/design/client/forge.status.md",
]
effort = "high"
minutes = 50
+++

# A stock queue's merges read as public landings

## Goal

A stock queue's changes land with no landing by turn, so `make_landings` lists nothing for them and a stranger reads an
empty Landings on a project whose work is landing. Make the forge's *landings* list each public queue merge after the
landings by turn, from the landing log, and make its *landing* answer through the merge's change. The wire is unchanged.

Acceptance:

- **The list:** `PUBLIC_LANDINGS` holds, after the public landings by turn, each public queue merge newest first by the
  log's `at`, in the form `notes/contracts.md` gives, all within `l.public_landings_landings`.
- **The landing:** `PUBLIC_LANDING` through a merge's change answers the `PublicLanding` that note gives. A change with a
  landing row answers as today, public or unknown.
- **The rule (`forge.md`, 11.2):** a logged landing is a public queue merge only when it has at least one change and every
  one of them passes `public_change` and has `change.way == Way::Queue`. Otherwise it is absent and its *landing* unknown:
  a change not handed, a change of another project, a change by turn, a logged landing with no changes.
- **An ended change still shows.** A handed task's `PublicTask.goal` may be `None` for a task that has ended (the root's
  `public_goal` reads live tasks), and jig's handing it is what makes it public (`jig/docs/design/domain/people.md`,
  13.7). The rule never reads `goal`. A test holds it, since `public_queues` does read `goal` and is the nearest copy.
- **Nothing else crosses:** no log's branch, title or pull request's number reaches an answer's bytes.
- **No wire change:** `temper-client-protocol-wire` is untouched; `public_bounds` and its golden still pass.

## Inputs

- `temper/docs/plan/ui-public-history/notes/common.md` and `notes/contracts.md`.
- `temper/docs/design/client/forge.md`, 11.1 (the *landings* and *landing* rows), 11.2 (*A public queue merge*), 11.3, 11.5.
- `temper/docs/design/domain/after-landing.md`, section 10 (*The landing log*). It says a change's close takes its row;
  today's `end_change` keeps the row, so `public_change` passes for a closed change. If you find a production path that
  removes a landed change's row, say so in the report and leave the rule as above.
- `jig/docs/design/domain/people.md`, 13.3, 13.7 and 13.9.

## Owns and may change

- **Owns:** `temper-engine-domain-forge`'s `src/public.rs`, `src/public/landings.rs`, the new `src/public/merges.rs` and
  `src/tests/public.rs`; the world's `temper/tests/engine/forge/src/public.rs` and `tests/suite/public.rs`.
- **May change, and how:**
  - `temper/docs/design/client/forge.status.md`: **a departure from the implementer's text, which closes `docs/design/` to
    you:** when your acceptance holds, remove this task's `**Not yet:** 11.1, 11.2` entry and put line 4 back to
    `Unlisted sections, including public documents (11), have no known gap.` Nothing else in `docs/design/`.
  - `src/tests/landing_log.rs` and `src/tests/verification.rs`: raising a helper's visibility (`add_change`, `merged`,
    `configured_domain`) if a test of yours reuses it, nothing else.
- **Not yours:** `temper-client-view`, its words and `browser_public.rs` (`board-words` owns them);
  `temper-client-protocol-wire`.
- **Pre-approved:**
  - If the world's turn fixture (`public::world`) can't reach a queue merge, build the story on `World::new` and
    `changes::reviewed_change` or `World::merge`, as `tests/suite/settle.rs` drives one, and name the helper you add to
    `temper/tests/engine/forge/src/public.rs`.
  - If an existing test's exact expected record trips because its fixture now logs a merge with a queue-way change, give
    that fixture's change `Way::Turn` or expect the line, whichever keeps what it checks, and name it in the report.
  - A change with a stale way, `Way::Queue` under a project since moved to turns, may then read as both a landing's line
    and a merge's. Leave it; add no dedupe.

## Steps

1. **`src/public/merges.rs`,** a new module beside `landings.rs` (cite `temper/docs/design/client/forge.md`, 11.2). The
   outer layer, in the strict subset:

   ```rust
   impl Domain {
       /// The project's public queue merges, newest first (a map's order, as `changes_history` keys its own).
       pub(super) fn queue_merges(
           &self,
           ask: &PublicAsk<'_>,
           limits: &Limits,
       ) -> Option<Map<(Reverse<Wall>, Repository, Commit), &LoggedLanding>>;
       /// Every change handed, of the project and by the queue, and at least one.
       fn public_merge(&self, ask: &PublicAsk<'_>, logged: &LoggedLanding) -> bool;
       pub(super) fn merge_line(&self, logged: &LoggedLanding, names: &mut Names) -> Option<w::PublicLandingLine>;
       /// The merge `change` is in, read from its repository's log; `None` is unknown.
       pub(super) fn make_merge(
           &self,
           ask: &PublicAsk<'_>,
           change: &ChangeRow,
           names: &mut Names,
       ) -> Option<w::PublicLanding>;
   }
   ```

   `queue_merges` walks `self.landing_logs` as `views/changes.rs`, `changes_history`, does, with capacity
   `limits.repositories * limits.landings_kept` by checked arithmetic. Mind the visibility of `LoggedLanding` and
   `skein_forge::api::{Commit, Repository}`: other names and a type alias are yours.

2. **`make_landings`** (`public/landings.rs`): after the turn landings' loop, push `merge_line` for each of `queue_merges`
   while `landings.room() > 0`.

3. **`PUBLIC_LANDING`** (`public.rs`):

   ```rust
   let row = self.public_change(ask, ask.task?)?;
   let record = match self.newest_landing(row.task) {
       Some(landing) => {
           if !self.public_landing(ask, landing) {
               return None;
           }
           self.make_landing(landing, &mut names)?
       }
       None => self.make_merge(ask, row, &mut names)?,
   };
   ```

4. **Unit tests** in `src/tests/public.rs`, writing `d.landing_logs` directly where that's the shortest, each expected record
   built independently in the file's manner:
   - `a_queue_merge_is_a_landed_line_newest_first`: two merges at different `at`; `turn` none; handed with `goal: None`.
   - `a_queue_merge_reads_as_its_landing`: through its change, and through either of two changes logged at one commit,
     whose line has two members and one `landed`.
   - `a_logged_landing_not_wholly_public_is_absent_and_unknown`: one row of the rule each (not handed; another project; by
     turn; no changes).
   - `landings_by_turn_stand_before_queue_merges_within_the_bound`: more merges than room leaves the newest.
   - **Extend `forbidden_sources_never_reach_the_bytes`** with a logged landing whose branch and title hold markers and a
     pull number, and assert those bytes absent.

5. **The world**, `tests/suite/public.rs`: `a_queue_merge_read_through_a_real_merge`: a stock-queue change merged through the
   fake forge, then `PUBLIC_LANDINGS`' line, `PUBLIC_LANDING`'s member and `PUBLIC_CHANGE`'s `merge` all name one commit;
   with the task not handed, the line is gone and `world.public(.., w::PUBLIC_LANDING, ..)` is `None`.

6. **Each new test bites:** break its rule once by hand (drop the `way` check; drop the handed check; push before the turn
   landings) and see the test fail for that reason; the report names them.

7. **Checks:** `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-engine-domain-forge -p
   temper-engine-forge-world --all-targets -- -D warnings`; `heavy cargo nextest run -p temper-engine-domain-forge -p
   temper-engine-forge-world`. This changes what a shared projection returns, so also run its readers:
   `heavy cargo nextest run -p temper-engine-domain` and `heavy cargo check -p temper-one-process-world --tests`; that
   world's walk stays unrun.

8. `python3 -I -B tools/citations/citations.py --base origin/main`, then commit and push.

## Report

As the implementer's text asks. Name: the rules you broke and what failed; whether any production path removes a landed
change's row; each file outside `Owns` you touched.
