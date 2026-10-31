# component-kit-3: the counts that may only fall

The counts of the draft's section 4 (`skein/docs/design/draft/component-kit.md`),
as forms of `skein/scripts/old_forms.py`, counted over skein's tracked Rust
files with comments removed, as the existing forms are. The `counts` task
adds them; each later task lowers them with `--write` for what it removes.

| Form | Matches | Outside | Baseline | Target |
|---|---|---|---|---|
| `io_resume` | `\b(?:skein_)?io::resume\s*\(` | `crates/skein-io/` | 39 | 0 |
| `earliest` | `\bfn\s+(?:earliest\|earlier)\s*\([^)]*\bTime\b` | `crates/skein-lib/` | 3 | 0 |
| `Lifecycle` | `\benum\s+Lifecycle\b` | `crates/skein-lib/` | 3 | 1 |
| `GROWTH_FLOOR` | `\bconst\s+GROWTH_FLOOR\b` | (everywhere) | 5 | 1 |
| `pop_loop` | `for _ in 0\.\.[^{;]*\{\s*(?:drop\(\|let _ = )?[\w.]+\.pop\(\)\)?;\s*\}` | (everywhere) | 21 | 0 |
| `hex_decimal` | `\bfn\s+(?:hex\|decimal)\b` | `crates/skein-lib/` | 26 | 9 |
| `forwarded_host` | `\bimpl\s+(?:\w+::)*Host\s+for\s+Process\b` | (everywhere) | 11 | 2 |

In the table, `\|` is the table's escape of a regex `|`. "Outside" is a
path from `skein/` whose files the form does not count.

The baselines are the cut's own count at `e7164f82d` (main before the
first task). The `counts` task's script count is the recorded baseline;
where it differs from this table, its report says why.

What stays at each target, and why:

- `Lifecycle`: `tests/io/src/processes.rs`'s `Lifecycle { Running, Exited,
  Closed }` is a scripted child's, not a component's (programming-model.md,
  5.2).
- `GROWTH_FLOOR`: lib's one growth rule keeps the one floor.
- `hex_decimal`: four writers that already call lib's `Decimal::of` and
  only adapt its digits to where they go (`skein-fake-forge-domain/src/actions.rs`,
  `skein-fake-forge-github/src/rates.rs`, `skein-forge-setup/src/github/app.rs`,
  `skein-ui/src/phrases.rs`); and five functions that are not copies of lib's helpers:
  `skein-fake-forge-github/src/pages.rs` (random hex), `skein-forge-setup/src/reconcile.rs`
  (a secret written as hex), `skein-json/src/string.rs` (a `\u` escape's
  accumulator), `skein-fake-llm-protocol/src/provider/inflate.rs` (a test's
  hex text) and `tests/json/src/reference.rs` (the reference parser, an
  independent oracle that shares nothing with the code it checks). Each
  may still fall if its body becomes a call of lib's helper.
- `forwarded_host`: `tests/io/src/processes.rs` and `tests/io/src/usage.rs`
  host io themselves; they forward nothing.
- `io_resume`: zero. Every host's io stage is `skein_io::stage::{up, down}`.

The plan's other figures (production and test lines of the crates in scope,
before and after) are fab's code report's.
