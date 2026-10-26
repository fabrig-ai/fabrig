# Supplement skein-01.s3: the request table retires a request not kept

Added by ruling 170, from the design answer of 2026-10-16T20:03Z (skein
`1f71424d`, `docs/design/lib.md`, 12, "Not kept"). jig 12.2 needs its
landing: 12.2's first commit moves jig's skein onto it. Read first:
skein's `AGENTS.md`, its gate `scripts/check.sh`, `lib.md`, 12; this
plan's README, "Rules of this plan".

Scope: `lib.md`, 12: "a request whose record the asker's store did not
write is retired by its key (`not_kept`), whatever its state: parked,
ready, in flight or retrying ... It is never sent; an answer for its
attempt changes nothing. Its erase is emitted, as any retired request's,
since an earlier write may hold its record. Its owner value is handed
back, as a final answer's, with no progress ... A key that names no
request, or a retired one, hands back nothing."
Needs: nothing (skein's `main`).
Files: `crates/skein-lib/src/requests.rs`: `pub fn not_kept(&mut self,
key: RequestKey) -> Option<O>`, beside `answered` (`:268-:305` at
`1f71424`). It finds the entry by `self.index` (`:177`). A key not held,
or `State::Retired`, gives `None`. Otherwise it sets `state =
State::Retired { erase: true }`, `save = false`, `changed = false` (as
`answered`'s `Final` arm, `:288-:291`) and returns `Some(owner)`. The
module doc's table (`:9-:23`) gains `| parked, ready, in flight,
retrying | not kept | retired | owner value, erase |`. Its guard on
`State::Retired` is an exhaustive `match`, as `answered`'s: skein's
`clippy.toml` disallows `core::matches` (`:86`), in tests too, and
`wildcard_enum_match_arm` is denied.
Test shape, in `crates/skein-lib/src/tests/requests.rs`:
- `not_kept_retires_a_parked_request_and_erases_it`: link down, `ask`,
  take its `Save` (`first_sent` `None`). `not_kept(key)` is
  `Some(owner(1))`, then `None`, and `progress()` is `None`. The next
  take gives `Erase(key)` alone. After `link(true)` and
  `confirmed(Some(7))`, a take gives nothing. `reclaim()` leaves
  `is_empty()` and `bytes()` 0.
- `not_kept_makes_a_held_attempt_stale`: `live()` (`:199`), `ask`,
  `sent` (`:207`) gives the attempt. `not_kept(key)` is `Some`, then
  `answered(.., attempt, Final)` is `Stale`, and a take gives
  `Erase(key)`. A second request answered `Final` gives `None` from
  `not_kept`.
- In the first test, a second parked request whose `Save` is not taken
  yet: `not_kept` of its key is `Some`, and the next take gives its
  `Erase` and no `Save` (this pins `save = false`).
- Their `Save` and `Erase` checks are exhaustive `match`es, as
  `final_hands_back_owner_once_and_erases_before_reclaiming`'s.
Proof: skein's gate.
Don't: change `answered`, `RequestOut`, `RequestAnswered`,
`RequestProgress` or `tests/lib/src/request_table.rs`; name a downstream
repository (`git -C <worktree> diff origin/main...HEAD | grep -i -w -E
'^\+.*(jig|temper|smith)'` prints nothing).
Size: at most 70 lines (ruling 171: the sketch, two tests, is 57).
Done when: `git -C ~/src/rust/skein log origin/main --all-match
--grep='^Plan: client-api$' --grep='^Step: skein-01.s3$' --format=%H -1`
prints its landing.
