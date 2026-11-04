+++
id = "smith-primitives"
part = "smith-items"
kind = "migration"
needs = ["skein-concat"]
files = ["smith/crates/**/*.rs", "smith/benchmarks/src/agent/smith/observe.rs", "smith/benchmarks/src/suite.rs", "smith/tests/**/*.rs", "smith/docs/development/debt/D016-retyped-primitives.md", "smith/docs/development/debt/README.md"]
effort = "medium"
minutes = 75
+++

# smith: pay D016, the primitives retyped beside skein's

## Goal

Pay `smith/docs/development/debt/D016-retyped-primitives.md`: smith converts native lengths with
`skein_lib::len_u64` wherever the conversion cannot fail, and has no private `earliest`, `earlier` or
byte `join` beside skein's. Three sweeps, one commit each. All of it is mechanical and no behaviour
changes on a supported target.

Acceptance:

- The item's **Measure** reads 0: `git ls-files 'smith/*.rs' | grep -v generated | xargs grep -oE
  'u64::try_from\([a-z_.()]*len\(\)\)' | wc -l` (90 at base).
- `rg -n 'u64::try_from\(.*len\(\)' smith -g '*.rs' -g '!**/generated/**'` holds only sites whose
  conversion can fail for a reason other than the target's width, each named in the report. It reads
  105 at base: the measure misses longer receivers.
- `fn earliest` in `smith-agent-service/src/service/stages.rs`, `fn earlier` in
  `smith-inline-agent/src/domain.rs` and `fn join` in `smith-protocol-program/src/mcp/json.rs` are gone.
- The item's file and its README row are gone.

## Inputs

- `temper/docs/plan/debt-7/notes/common.md`, `AGENTS.md`, `smith/AGENTS.md`.
- `smith/docs/development/debt/D016-retyped-primitives.md`.
- `skein/docs/design/lib.md`, 6 (`bytes::concat`) and 8 (`earliest`).
- `skein/crates/skein-lib/src/length.rs`; `skein/crates/skein-lib/src/time.rs`, `earliest`, whose doc
  gives the form: `next = earliest(next, Some(deadline))`.

## Owns and may change

- Owns: every site below. Any smith `.rs` file may change, each only for these three forms.
- **Exempt, not changed:** `u64::try_from(size_of::<T>())`, which converts no length (D016 is about
  lengths); `smith-local-protocol/src/plain.rs`'s `join`, `smith-domain-tools/src/path.rs`'s `join` and
  `joined`, and `smith-protocol-program/src/backend/replies.rs`'s `joined`, which are bounded and answer
  `None`, not byte joins of `parts`; the `generated/` trees.
- **A helper that loses its only failure:** where `.ok()?` on a length was a function's only way to
  answer `None`, drop the `Option` from that private function and the `?` at its callers, if the function
  is private to its crate and the change stays under about ten lines; otherwise keep the signature and
  answer `Some(len_u64(..))`. Checked arithmetic around the conversion stays: `checked_add`,
  `checked_mul` and their `?`.
- `match u64::try_from(x.len()) { Ok(n) => .., Err(_) => .. }`: the `Err` arm goes, and any failure path
  only it reached goes with it, in that function.
- `unwrap_or(u64::MAX)`, `.expect(..)` and `.ok() == Some(n)` on a length: the conversion alone, so
  `len_u64(x.len()) == n` for the last.
- A public signature of a smith crate does not change. If a site needs one changed, keep the signature
  and answer `Some(..)`.
- **A failure path a test drives:** if a test reaches a removed arm, stop and ask. The item says none can
  be taken.

## Steps

1. **Lengths.** The 90 sites by file are the measure's output without `wc`, and the top of that list is
   `smith-domain-session/src/session/compaction.rs` (9), `llm.rs` (6), then `smith-protocol-llm` and
   `smith-domain-tools`. A crate that names `skein_lib` needs no new dependency: all of them do,
   `smith/benchmarks` included.
2. **Earliest.** In `stages.rs`, `next_deadline` and `next_policy_deadline` take the fold's form, starting
   from the first deadline. A step is the strict subset (`smith/AGENTS.md`): no closure, no `fold`.

   ```rust
   let mut next = match face::domain(frame) { Some(domain) => domain.next_deadline(), None => None };
   next = earliest(next, service.llm.next_deadline());
   ...
   next
   ```

   In `smith-inline-agent/src/domain.rs`, `next = Some(earlier(next, at))` becomes
   `next = earliest(next, Some(at))`.
3. **Concatenation.** Every `json::join` and `mcp::json::join`, tests included, becomes
   `skein_lib::bytes::concat`; `json.rs`'s own calls too. Delete `fn join`, and a `Writer` import it
   leaves unused.
4. `cargo fmt`; `python3 -I -B smith/scripts/old_forms.py`; `heavy cargo clippy` naming every package
   changed, `--all-targets -- -D warnings`; `heavy cargo nextest run` naming the same packages and
   their direct dependents in smith. This changes no meaning, so it runs no reader outside smith.
5. Delete the item and its row; the citations check. Commit and push.

## Report

The common note's items. No new test, so no break. The two counts, the exemptions you met, and each
helper that lost its `Option`. Net lines.

## Answers

2026-11-04. May the 15 conversions in `smith/tests/**` and `smith/benchmarks/src/suite.rs` change, or are they
exempt? **They change; `files` now lists them.** They are not exempt: each widens a native `usize`, and the broad
search of the acceptance must hold. Every package there names `skein_lib`, so no manifest changes. Each is the
conversion alone: `u64::try_from(<expr>).expect(..)` becomes `skein_lib::len_u64(<expr>)`, its `.expect` going with
it, `<expr>` as it stands, no assertion and no value moved. This departs from `notes/common.md`'s "a test may change
only where it names a moved item" for these lines only. A `size_of` inside a longer length sum, as in
`fuzzy/listing.rs` and `recorded.rs`, is one of them: the exemption is for `u64::try_from(size_of::<T>())` alone.
The packages they sit in join step 4's clippy and nextest lists. Keep the pushed commit, then the two other sweeps.
