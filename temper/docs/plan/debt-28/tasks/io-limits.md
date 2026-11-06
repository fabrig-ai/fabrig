+++
id = "io-limits"
part = "test-limits"
kind = "migration"
needs = ["file-readers-skein"]
files = ["skein/crates/skein-io/src/limits.rs", "skein/crates/skein-fake-peers/src/face.rs", "skein/tests/playground/src/site.rs", "skein/crates/skein-io/src/tests/records.rs", "skein/tests/**", "skein/testing/**", "skein/examples/echo/service/src/tests.rs"]
effort = "medium"
minutes = 100
+++

# Test worlds start their io limits from `Limits::TEST`

## Goal

Part of skein's D033, the part that recovers the most lines. `io::Limits` is written out whole 52 times, 13 lines each, and `Limits::TEST` is reached by few: no whole literal is within 4 fields of it. So `TEST` moves to the values these literals share, and each literal in test code becomes a struct update naming only what differs. Every value stays what it is, so no test's behaviour moves: a memory pin reads the same limits.

The item's measure counts signatures and struct updates and can't fall, so it's replaced. This measure counts whole literals, in any spelling, and read 52 when the cut ran it:

```
rg -U --pcre2 --count-matches '(?<!struct )Limits \{(?=[^{}]*\bclose_timeout:)(?=[^{}]*\baccepts:)(?:(?!\.\.)[^{}])*\}' skein -g '*.rs' | awk -F: '{s+=$2} END{print s+0}'
```

Acceptance:

- The measure reads 6, one definition aside if your count shows it: the sites that stay whole, named below.
- `Limits::TEST` holds the values below, and `crate::valid(&crate::Limits::TEST)` still passes (`skein-io/src/tests/records.rs`).
- **Every field of every converted site has the value it had.** A field equal to `TEST`'s drops; a field that differs stays, spelled as it was. Your report says how you checked it.
- No memory pin moves, and no test's assertion changes.
- Net at least 300 lines fewer.

The item stays open: `client-limits` rewrites it.

## Inputs

- `skein/docs/development/debt/D033-limits-literals-in-tests.md`, whole.
- `skein/docs/foundation/testing-strategy.md`, 10.
- `skein/docs/design/io.md`, 8, whose last line is this rule.
- `temper/docs/plan/debt-28/notes/common.md`.

## Owns and may change

Owns `limits.rs`, and in `skein/tests/**`, `skein/testing/**` the unit tests of `skein/crates/**` and `skein/examples/echo/service/src/tests.rs`, only the literal of a `skein_io::Limits`. Nothing else in those files.

**These stay whole,** since their values are real configuration or are computed: `skein-ui-text/src/terminal.rs`, `skein/examples/echo/shell/src/lib.rs`, `skein/examples/ui/server/src/main.rs`, `skein/testing/skein-fake-person-browser/src/harness/capacity.rs`, `skein/tests/browser/tests/support/real.rs`, and `skein/tests/io/tests/suite/fuzzy/limits.rs`, whose every field is computed.

**These read `TEST` today** and keep their values by naming more fields: `skein-fake-peers/src/face.rs` (`sockets: 8, receive: 4096, sends: 8, backlog: 2, retry: 1 ms` join what it has), `skein/tests/playground/src/site.rs`, and `skein-io/src/tests/records.rs`'s `changed`.

A literal outside skein is not this task's. The 21 struct updates from other bases (`limits().io`, `good.io`) are already the target shape: they stay. A local `fn limits()` stays: `testing-strategy.md`, 10, asks for a setup written once, and that's one. Only the literal inside it changes.

If `TEST`'s new values fail `valid`, change only what fails, as little as it takes, and say so. If a converted site's test fails, a value was lost: fix the site, and never the assertion.

## Steps

```rust
// skein-io/src/limits.rs
pub const TEST: Self = Self {
    sockets: 2,
    names: 0,
    refusals: 1,
    intake: 32_768,
    receive: 1024,
    output: 32_768,
    sends: 2,
    accepts: 1,
    backlog: 1,
    close_timeout: Duration::from_secs(1),
    retry: Duration::from_millis(10),
};
```

These are each field's most common value across the literals, so overrides are fewest. The doc comment's citation stays.

A site reads as a sentence of what it means: `io::Limits { sockets: 1, intake: 19_000, output: 19_000, ..io::Limits::TEST }`. A site that spells `Limits {` by a local import keeps its spelling. A throwaway untracked script is the way: find each literal with the measure's query, parse its fields, compare with `TEST` by value (`32768` and `32_768` are one value, and an expression stays as written), and rewrite it. Delete the script before you push.

No new test: this moves no behaviour, and no table repeats these values.

`heavy cargo nextest run -p skein-io`; then every package whose file you changed, named, in `default`. These include the memory pins in `skein-io-world`, `skein-fake-peers`'s world and the llm connection's: they hold, since no value moved. Then `--profile fuzzy` for `skein-io-world` alone. `cargo fmt`; `(cd skein && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy` naming each package, `--all-targets -- -D warnings`; `python3 -I -B tools/citations/citations.py --all`. Commit, push.

## Report

As your text says, with the measure's reading, the net lines, and how you checked the values.

## Answers

2026-11-06. May this task edit the echo service's unit-test literal and keep the computed fuzzy literal whole, so the measure reads 6? Yes. `skein/examples/echo/service/src/tests.rs`'s `limits()` is a test literal like the others: its `io` literal converts, keeping every value, and its package joins the ones you test. `skein/tests/io/tests/suite/fuzzy/limits.rs`'s literal stays whole with all its draws in order, with no `..Limits::TEST` added, since an update that supplies no field would only beat the measure. The spec above now says so. The net cap holds.
