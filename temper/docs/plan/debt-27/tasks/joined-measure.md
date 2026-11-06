+++
id = "joined-measure"
part = "joined-measure"
kind = "feature"
needs = []
files = ["skein/crates/skein-lib/src/length.rs", "skein/crates/skein-lib/src/tests/length.rs", "skein/crates/skein-http/src/client/request.rs", "skein/crates/skein-http/src/server/response.rs", "skein/crates/skein-http/src/sse/writer.rs", "skein/crates/skein-fake-forge-github/src/**", "skein/crates/skein-fake-forge-forgejo/src/setup.rs", "skein/crates/skein-fake-forge-forgejo/src/tests/migrate.rs", "skein/crates/skein-fake-forge-domain/src/actions.rs", "skein/crates/skein-fake-oauth/src/tests.rs", "skein/crates/skein-oauth-accounts/src/paste_tests.rs", "skein/crates/skein-oauth-issuer/src/documents.rs", "skein/crates/skein-forge-connection/src/tests/reads.rs", "skein/tests/forge/machine/tests/suite/hand/github/github_writes_heap.rs", "skein/tests/forge/machine/tests/suite/hand/github/puller.rs", "skein/tests/forge/machine/tests/suite/hand/forgejo/reads.rs", "skein/docs/design/lib.status.md", "skein/docs/development/debt/D007-joined-and-measure.md"]
effort = "high"
minutes = 55
+++

# One concatenation and one length measure in skein-lib

## Goal

Pay skein's D007. `skein_lib::bytes::concat` is the one concatenation: the copies of `joined` go onto it, each overflow answer with them. `skein_lib::length::Measure` is new, as `skein/docs/design/lib.md`, 6, says, and skein-http's three go onto it.

Acceptance:

- `rg -n 'fn joined\b|struct Measure\b' skein -g '*.rs'` names two lines: `skein-lib/src/length.rs`'s `Measure`, the one the item asks for, and `skein-forge/src/forgejo/server/files.rs`'s `joined`.
- `files.rs`'s `joined` is **exempted, not changed**: it puts a slash between a path's segments, which is another job than a concatenation of parts, and the request exempts code that uses the same word for something else.
- No test changes what it asserts. Only the `.unwrap()` or `.expect()` on a removed `Option` goes.
- `lib.status.md`'s "6: `length::Measure`, while skein-http's three writers each keep their own." sentence is gone.
- `skein/docs/development/debt/D007-joined-and-measure.md` is deleted.
- Net lines fall, generated code and tests counted apart.

## Inputs

- `skein/docs/development/debt/D007-joined-and-measure.md`, whole.
- `skein/docs/design/lib.md`, 6.
- `temper/docs/plan/debt-27/notes/common.md`.

## Owns and may change

Owns the files in the header. `length::Measure` is an addition; `service::joined` is `pub(crate)`, so nothing outside `skein-fake-forge-github` reads it. This plan's `utf8-one` edits `skein-oauth-issuer/src/documents.rs` in another function: spell the call there as `skein_lib::bytes::concat`, so that the `use` lines don't meet.

Left alone, and reported: `join(parts, max)` in `skein-fake-forge-github/src/pages.rs` and `skein-forge/src/github/setup.rs`, whose bound is their job; `skein-fake-llm-domain/src/pieces.rs`'s `join`, over its own `Piece`.

Options, each pre-approved:

- A function left returning `Option` or `Result` with no way left to `None` or `Err`: if it's private and has few callers, drop the wrapper and fix those callers; otherwise keep the signature, under a scoped `#[expect(clippy::unnecessary_wraps, reason = "...")]` if clippy asks.
- An error variant left unbuilt in `skein-fake-forge-github`, which is private to the fake, goes. In `skein-oauth-issuer`, `Error` is public: leave it, and report it.

## Steps

**`skein-lib/src/length.rs`:**

```rust
/// A length added up with checks (skein/docs/design/lib.md, 6): past a `usize` it stays `None`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Measure(Option<usize>);

impl Measure {
    pub const ZERO: Measure = Measure(Some(0));
    pub fn add(&mut self, len: usize);
    #[must_use]
    pub const fn total(self) -> Option<usize>;
    #[must_use]
    pub fn total_u32(self) -> Option<u32>;
}
```

`skein_lib::length::Measure` is its path; no re-export at the crate's root. One table test in `tests/length.rs`, over rows of parts, `total` and `total_u32`: no parts, `[1, 2]`, a sum one past `u32::MAX`, a sum past `usize::MAX`, and more parts after that one. Break `checked_add` and the `u32` conversion by hand in turn, and see the table go red.

**skein-http's three.** `client/request.rs` reads `total()`, `response.rs` and `sse/writer.rs` read `total_u32()`. `sse/writer.rs`'s `field` method becomes a private function beside `field`, named as you choose. The answers don't move: a part past a `u32` or a sum past one was `None` in `Option<u32>`, and is `None` from `total_u32`.

**`joined`:**

| Where | What goes |
|---|---|
| `skein-fake-forge-forgejo/src/setup.rs`, `src/tests/migrate.rs`; `skein-fake-oauth/src/tests.rs`; `skein-oauth-accounts/src/paste_tests.rs`; `skein-forge-connection/src/tests/reads.rs`; `hand/github/github_writes_heap.rs`, `hand/github/puller.rs`, `hand/forgejo/reads.rs` | `fn joined(parts)`; each call is `concat(&[..])` |
| `skein-fake-forge-github/src/service.rs`, and its 28 callers in that crate | `joined(first, last) -> Option`; each is `concat(&[first, last])`. Chains flatten: `writes.rs`'s three-step prefixes become one `concat`. The `else` arms go, since only a sum past a `usize` reached them |
| `skein-fake-forge-domain/src/actions.rs` | `joined(context, parts)`; its one call is `concat(&[b"Run ", context])`. `append` stays |
| `skein-oauth-issuer/src/documents.rs` | `joined(left, right) -> Result`; its calls are `skein_lib::bytes::concat(&[left, b".", right])`, and their `?` goes |

Tests: `heavy cargo nextest run -p skein-lib --lib`, `-p skein-http`, `-p skein-fake-forge-github`, `-p skein-fake-forge-forgejo`, `-p skein-fake-forge-domain`, `-p skein-fake-oauth`, `-p skein-oauth-accounts`, `-p skein-oauth-issuer`, `-p skein-forge-connection`, and the world in `skein/tests/forge/machine` by its manifest's name. Dependents of `skein-lib` read an addition: their tests don't run.

`cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`, `--write` if a count falls; `heavy cargo clippy` naming each changed package, `--all-targets -- -D warnings`; `python3 -I -B tools/citations/citations.py --all`; `python3 -I -B tools/debt/debt.py`. Commit, push.

## Report

As your text says: what you broke and what failed, the search's two lines, the net lines, and what you left under `## Left`.
