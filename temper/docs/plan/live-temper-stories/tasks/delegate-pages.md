+++
id = "delegate-pages"
part = "landing-story"
kind = "feature"
needs = ["held-login-client"]
files = [
  "temper/crates/temper/tests/support/forge_land.rs",
  "temper/crates/temper/tests/suite/forge_land.rs",
  "temper/docs/design/domain/testing.status.md",
]
effort = "high"
minutes = 60
+++

# The landing story reads the ended delegates whole, a page at a time

## Goal

The Claude Code landing story ran 56.6 s under fab's after-landing pair, then failed `whole
delegate set`: `task list --project 2 --parent 2 --ended --json` exited 0 with its page cut
or a cursor after it, and the live judge requires each parent's ended delegates on one uncut
page. Read them as `temper/docs/design/domain/testing.md, 2.7` now says: a page at a time, by
`--after` and the cursor ctl printed, until a page prints none, within 32 pages.

Acceptance:

- A new `EndedDelegates` (below) holds the paging; the live judge reads the chat's ended
  delegates, then the change's, through it, in a new `LivePhase::Ended`, before the closing
  reads; `check_landing` takes its delegate sets from it, and the positional `tasks_at` reads
  are gone.
- A cut page with a cursor reads on; a page cut with no cursor, a live delegate, a row with no
  task number, or a 33rd page fails the story, naming it.
- The new suite tests (below) pass; every `forge_land::` test of `temper`'s `suite` binary
  passes; clippy is clean (it compiles the `live` binary).
- `temper/docs/design/domain/testing.status.md` no longer holds this plan's 2.7 entry.
- No live story is run (notes/common.md).

## Inputs

- `temper/docs/plan/live-temper-stories/notes/common.md`.
- `temper/docs/design/domain/testing.md, 2.7`: the owned-forge landing story's bullet, its
  sentences from "The chat's ended delegates and the change's are read whole".
- `temper/docs/design/client/tools.md, 5.2`: the `task list` row and the bullet "`task list`
  reads one group a call" (`--ended` with `--after CURSOR` reads on in the ended group).
- `temper/docs/plan/live-temper-stories/inputs/request.md`, item 2.

## Owns and may change

Owns, in `temper/crates/temper/tests/`:

- `support/forge_land.rs`: the new `EndedDelegates` and its `impl`, placed beside
  `plan_ended_tasks` (~379); `LivePhase` (~1437) gains `Ended`; `LiveJudge` (~1455): the
  field `tasks_at` becomes `ended: Option<EndedDelegates>` (its initial value in `run_live`,
  ~2354); `advance`'s match (~1504-1584) gains the `Ended` arm; `read_page`'s last `else`
  (~1758-1774); `check_landing`'s delegate loop (~1821-1834). Nothing else of the file: the
  offline `plan_ended_tasks` and `ended_tasks` stay as they are.
- `suite/forge_land.rs`: the two new tests, at the end of the file.
- `temper/docs/design/domain/testing.status.md`: remove the **Not yet** entry `2.7: the
  owned-forge landing story reads each parent's ended delegates as one uncut page
  (`crates/temper/tests/support/forge_land.rs`).` whole (the `held-login-client` task left it
  so), nothing else. This is the task's one departure from the implementer's text ("Not
  yours: any `docs/design/`").

Read only: `support/github_land.rs` (`delegate_page` ~328, the fake story's paging by
`ended.rows.after.value`), `support/ctl.rs`, `live.rs` (~133 compares
`LiveEvidence::delegates` across the two runs: its type, `[Vec<u64>; 2]`, and order, each
sorted, are unchanged).

Pre-approved options:

- If the closing reads (`chat close`, `task tree --until ended`, `task list`, queued only when
  `!self.plan.first`) read cleaner in a helper beside `queue_delegates`, that is yours; keep
  their order and arguments.
- If a page's rows also carry `requester.task`, you may require it to equal the page's parent
  and say so; otherwise leave it.

## Steps

1. `support/forge_land.rs`. Sketch of the outer layer (bodies yours):

   ```rust
   /// The chat's ended delegates and its change's, read whole, a page at a time, by `--after`
   /// and the cursor each page prints (domain/testing.md, 2.7).
   pub struct EndedDelegates { /* project, the two parents, the parent being read, pages read, tasks */ }

   impl EndedDelegates {
       /// The pages read in all before the story fails.
       pub const PAGES: usize = 32;

       /// `parents` is `[chat, change]`.
       #[must_use]
       pub fn new(project: u64, parents: [u64; 2]) -> Self;

       /// The first read: `task list --project P --parent CHAT --ended --json`.
       #[must_use]
       pub fn first(&self) -> Vec<String>;

       /// Take the read of the page last asked for: the next read (the same parent's with
       /// `--after CURSOR` appended, or the change's first), or `None` once both are whole.
       /// # Errors
       /// A live delegate (`live` not empty or `live_after` set: "no live delegates"), rows
       /// missing, a row with no `chip.task`, a page `cut` with no cursor, or a read past
       /// `PAGES`.
       pub fn read(&mut self, read: &serde_json::Value) -> Result<Option<Vec<String>>, String>;

       /// Both parents' ended delegates, each sorted; once `read` has answered `None`.
       #[must_use]
       pub fn whole(self) -> [Vec<u64>; 2];
   }
   ```

   A page is `read["ended"]["rows"]`: its `rows`, `cut` and `after`, the cursor
   `after["value"]` (as `github_land.rs` reads it). The reads' words are those
   `read_page` queues today, `--after CURSOR` after `--json`.

   The judge: `read_page`'s last `else` sets `self.ended` to `EndedDelegates::new(project,
   [chat, change])`, queues its `first()` and enters `LivePhase::Ended`. The `Ended` arm calls
   `read(&last)?`: `Some(words)` is queued (`self.pending.push_back`); `None` queues the
   closing reads when `!self.plan.first`, as `read_page` does today, and enters
   `LivePhase::Closing`. `check_landing` takes `self.ended.take()` (an `Err` if absent) and
   its `whole()` in place of its loop over `calls.ends()[self.tasks_at + offset]`; what
   follows the loop is unchanged.

2. `suite/forge_land.rs`:

   ```rust
   fn delegate_read(rows: &[u64], cut: bool, after: Option<&str>) -> Value {
       // {"live": [], "live_after": null,
       //  "ended": {"rows": {"rows": [{"chip": {"task": N}}...], "cut": cut,
       //                     "after": after.map(|value| {"value": value}) or null}}}
   }

   /// domain/testing.md, 2.7: the chat's and the change's ended delegates are read whole,
   /// a page at a time, by the cursor each page prints.
   #[test]
   fn ended_delegates_are_read_whole_by_their_cursor() {
       // EndedDelegates::new(2, [2, 3]):
       //   first() == task list --project 2 --parent 2 --ended --json
       //   read([3], false, Some("3")) -> Some(... --parent 2 --ended --json --after 3)
       //   read([5], true, Some("5"))  -> Some(... --after 5)      a cut page with a cursor reads on
       //   read([], false, None)       -> Some(... --parent 3 --ended --json)
       //   read([7, 6], false, None)   -> None
       //   whole() == [vec![3, 5], vec![6, 7]]
   }

   /// domain/testing.md, 2.7: what the story refuses of a delegate page, naming it.
   #[test]
   fn an_ended_delegate_read_refuses_live_rows_a_cut_without_a_cursor_and_a_33rd_page() {
       // each on a fresh EndedDelegates::new(2, [2, 3]):
       //   a page with a live row, or with live_after set -> Err containing "no live delegates"
       //   read([4], true, None)                         -> Err naming the cut and the cursor
       //   a row without chip.task                       -> Err
       //   PAGES reads each with a cursor answer Some; the next read -> Err naming the bound
   }
   ```

3. Tests: each new test alone (`heavy cargo nextest run -p temper --test suite -E
   'test(=forge_land::NAME)'`), then `-E 'test(/^forge_land::/)'`.

4. The status entry (Owns). Then `cargo fmt`; `heavy cargo clippy -p temper --all-targets --
   -D warnings`; `heavy cargo nextest run -p temper` (the focused profile: never the live
   binaries); `python3 -I -B tools/citations/citations.py --base origin/main` after
   committing.

5. Commits: `EndedDelegates` and its tests; the judge's paging; the status entry. Push your
   branch.

## Report

As the implementer's text says, and also: the error words each refusal prints, the judge's
phases as they now run from `Pages` to `Closing`, and the `suite` counts.
