+++
id = "first-demand"
part = "first-demand"
kind = "feature"
needs = []
files = ["skein/crates/skein-forge/src/forgejo/client.rs", "skein/crates/skein-forge/src/forgejo/client/delivery.rs", "skein/crates/skein-forge/src/forgejo/server.rs", "skein/crates/skein-forge/src/forgejo/server/writes.rs", "skein/crates/skein-forge/src/forgejo/tests/fixtures.rs", "skein/crates/skein-forge/src/github/decode/delivery.rs", "skein/crates/skein-forge/src/github/tests/fixtures.rs", "skein/crates/skein-forge-setup/src/host/connection.rs", "skein/crates/skein-fake-forge-http/src/git/client.rs", "skein/crates/skein-fake-forge-http/src/git/server.rs", "skein/crates/skein-fake-forge-forgejo/src/server.rs", "skein/crates/skein-fake-forge-forgejo/src/tests/**", "skein/crates/skein-fake-forge-github/src/git.rs", "skein/tests/forge/dialect/tests/common/forgejo.rs", "temper/crates/temper-fake-git/src/exchange.rs", "temper/crates/temper-fake-git/src/tests/net.rs", "skein/docs/design/forge.md", "skein/docs/design/forge.status.md"]
effort = "medium"
minutes = 30
+++

# A decoder's first demand is `Next`, asked by whoever drives it

## Goal

Finding 1 of `temper/docs/plan/debt-29/inputs/debt27-review.md`, its first half: skein's D040 was deleted unpaid. Forgejo's `client::first`, `client::delivery::first` and `server::body_first` ignore their decoder and return `tokenizer::Request::Next`. They go, and whoever drives a decoder asks for `Next` itself.

Three more functions have the same body, and the design's bullet names one of them, so they go too: GitHub's `decode::delivery::first`, and `skein-fake-forge-http`'s `git::client::first` and `git::server::body_first`. A removal this spec names is agreed.

Acceptance:

- `rg -n 'fn (body_)?first\(_decoder' skein temper -g '*.rs'` finds nothing (6 at the cut).
- `rg -n 'body_first|(client|delivery|documents)::first\(' skein temper -g '*.rs'` finds nothing.
- `skein/docs/design/forge.md`, 12's bullet **A decoder's first demand** is gone, and `skein/docs/design/forge.status.md`'s **Departs** sentence on `5.3, 7` is gone. The citations stay at 0.
- No assertion changes. The net is fewer lines.

## Inputs

- `temper/docs/plan/debt-29/inputs/debt27-review.md`, finding 1.
- `skein/docs/design/forge.md`, 12 (the bullet) and `skein/docs/design/forge.status.md` (the sentence).
- `temper/docs/plan/debt-29/notes/common.md`.

## Owns and may change

Owns the files in the header, for this edit alone.

- **The status's "No service does" is wrong:** `skein-forge-setup/src/host/connection.rs` calls `documents::first(decoder)`, and it ships. That arm sets `self.json_next` to `Some(tokenizer::Request::Next)` when the decoder was made, and finishes when it wasn't. Write it as the other arms read; any shape that keeps its two outcomes is yours, and `Some(_) | None` isn't one.
- **The two temper lines are the one departure from "a branch stays in its tree":** `temper-fake-git/src/exchange.rs` and `src/tests/net.rs` each pass `tokenizer::Request::Next` where they called `client::first(&decoder)` or `server::body_first(&decoder)`, and change nothing else. Both already import `tokenizer`.
- **A `use` that goes unused** by this change goes. A test module that lacks `tokenizer` imports it.
- `skein/docs/development/debt/reviews/**` keeps its old words: reviews are raw.
- `temper/docs/plan/**` isn't yours.

If a caller beyond these appears, including through fab's merge of the trunk, it gets the same edit. If a caller's `decoder` binding exists only to be passed to the removed call, drop the binding and say so in the report.

## Steps

Delete the six functions and `server.rs`'s `pub use writes::{.., body_first}` name; fix every caller the compiler names. The sites the cut saw:

| Package | Files |
|---|---|
| `skein-forge` | `forgejo/tests/fixtures.rs` (3), `github/tests/fixtures.rs` (1) |
| `skein-forge-setup` | `host/connection.rs` (1, ships) |
| `skein-fake-forge-forgejo` | `server.rs` (1, the fake), `tests/memory.rs`, `tests/migrate.rs`, `tests/git.rs` (2), `tests/fixtures.rs` (2) |
| `skein-fake-forge-github` | `git.rs` (1) |
| `skein-forge-dialect-tests` | `tests/common/forgejo.rs` (5) |
| `temper-fake-git` | `exchange.rs`, `tests/net.rs` |

Delete the design's bullet and the status's sentence, keeping every other sentence of that **Departs** entry as it is.

No new test: the removed functions held no behaviour.

`heavy cargo nextest run -p skein-forge -p skein-forge-setup -p skein-fake-forge-http -p skein-fake-forge-forgejo -p skein-fake-forge-github -p skein-forge-dialect-tests -p temper-fake-git`. `cargo fmt`; `(cd skein && python3 -I -B scripts/old_forms.py)` and the same for temper; `heavy cargo clippy` naming those packages, `--all-targets -- -D warnings`; `python3 -I -B tools/citations/citations.py --all`. Commit, push.

## Report

As your text says, with the two searches' readings and the net lines.
