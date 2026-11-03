+++
id = "limits-pages"
part = "pages"
kind = "feature"
needs = ["limits-jig"]
files = ["temper/crates/temper-service/src/limits/names.rs", "temper/crates/temper-service/src/limits/reference.rs", "temper/crates/temper-service/src/limits.rs", "temper/crates/temper-service/src/tests/limits.rs", "temper/crates/temper-forge-setup/src/limits.rs", "temper/crates/temper-forge-setup/src/check.rs", "temper/crates/temper-forge-setup/src/tests/limits.rs", "temper/crates/temper-box/src/limits.rs", "temper/crates/temper-box/src/tests/limits.rs", "temper/crates/temper/src/tests/limits.rs", "temper/crates/temper/src/tests.rs", "temper/crates/temper-docs/src/lib.rs", "temper/docs/help/reference/limits.md", "temper/docs/help/reference/limits-engine.md", "temper/docs/design/protocol/documentation.status.md", "temper/crates/temper-service/Cargo.toml", "Cargo.lock"]
effort = "high"
minutes = 150
+++

# limits-pages: the reference pages `limits` and `limits-engine`

## Goal

temper names every limit `application.limits.overrides` takes in a table beside its parser,
as jig does (`limits-jig`; contract C3), and renders the two reference pages, `limits`
(temper's, the setup's and the box's) and `limits-engine` (jig's), checked in under
`docs/help/reference/` and held equal to their rendering (temper's
`protocol/documentation.md`, 8 and 6). The configuring agent reads them with `read_docs`.
This task's departure from the implementer's text: it checks in the two rendered pages and
their `PAGES` entries.

Acceptance:

- `temper_service::limits::NAMES` and `find` (`limits/names.rs`), `temper_forge_setup::limits::NAMES`
  (and the host's), `temper_box::limits::NAMES`, each parser looking its name up first, as
  C3 says; lines as `limits-jig`'s, temper's words for temper's limits.
- `temper_service::limits::reference` and `engine_reference` (in `limits/reference.rs`,
  re-exported from `limits`) render C3's pages; each page within `temper_docs::PAGE_BYTES`
  (32,768).
- `temper/docs/help/reference/limits.md` and `limits-engine.md` checked in, each with its head
  (title, `Kind: reference`, a one-sentence summary), and their two `PAGES` rows in
  `temper-docs/src/lib.rs` in the table's order (`limits` and `limits-engine` after
  `host-tools`, before `settings-fields`), the array's length raised by two.
- Step tests: in the `temper` crate, `tests/limits.rs`, `the_pages_limits_are_what_the_tables_render`
  (with `SKEIN_CODEGEN_WRITE=1` rewriting the files, as `commands.rs`' tests do); in each of
  `temper-service`, `temper-forge-setup` and `temper-box`, `every_row_is_a_name_the_parser_takes`
  and `an_unknown_name_is_refused`, as `limits-jig`'s.
- temper's `docs/design/protocol/documentation.status.md`: the sentence "8: the reference
  pages `limits` and `limits-engine` (`docs/help/reference/` has `commands`, the `ctl-GROUP`
  pages, `host-tools` and `settings-fields`)." removed.
- The focused tests of `temper-service`, `temper-forge-setup`, `temper-box`, `temper-docs`
  and `temper` pass.

## Inputs

- `temper/docs/design/protocol/documentation.md`, 2.1, 6 and 8.
- `temper/docs/design/protocol/configuration.md`, section 7 (temper's limits).
- `temper/docs/plan/configuring-p3/notes/contracts.md`, C3.
- `temper/docs/plan/configuring-p3/notes/common.md`.

## Owns and may change

- Owned: the files in this spec's `files`; in `temper/crates/temper/src/tests.rs` the `mod
  limits;` line.
- `temper/crates/temper-service/src/limits.rs` is also changed by `public-reading-build` on its
  trunk: keep its hunk to the module and re-export lines.
- Options, pre-approved:
  - If `limits` passes 32,768 bytes, shorten lines first; if it still passes, split the setup's
    and the box's names into a third page, `limits-setup`, with its own row, and say so.
  - A name in a parser with no field doc and no design words: write the line from what its arm
    sets, and list it.

## Steps

1. temper's three tables and lookups, crate by crate, each with its tests.
2. The renderers; the page test; write the pages with `SKEIN_CODEGEN_WRITE=1`, then check
   them clean.
3. The `PAGES` rows; the status sentence.
4. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p
   temper-service -p temper-forge-setup -p temper-box -p temper-docs -p temper --all-targets
   -- -D warnings`; their focused tests; the citations check.
5. Commit ("temper: the reference pages limits and limits-engine (documentation.md, 8)") and
   push.

## Report

As the implementer's text says, and each page's bytes.

## Answers

2026-11-03. May temper-service depend on temper-box, so that `reference` renders the box's
rows from `temper_box::limits::NAMES`? Yes (option A): add `temper-box.workspace = true` to
`temper/crates/temper-service/Cargo.toml`'s `[dependencies]` and let Cargo.lock follow; both
are now in `files`, and this is the one dependency the spec allows beyond `common.md`'s rule.
temper-box needs only skein and temper-forge-setup, which temper-service already has, so the
edge makes no cycle and C3 keeps its shape: both renderers stay in temper-service's `limits`
module. No other dependency changes.

2026-11-03. May the task give `setup.fetch_bytes` a table row and change
`temper-forge-setup/src/check.rs` so its name is looked up first? Yes (option A). The
design names it (`configuration.md`, 7) and the page must list every override
(`documentation.md`, 8), so leaving it out would fail this task's own acceptance. In
`temper-forge-setup/src/limits.rs`, beside `fetch_bytes`, add `FETCH_NAMES`: one row with
C3's members (`name`, `unit` bytes, `line`, `value`), with `value` reading the scalar
(`fn(&u64) -> u64`). Its small value is `FETCH_BYTES`. The setup crate's row type may be
generic over what `value` reads (`Name<L>`, with `Limits`, `Host` and `u64` as `L`), or it
may be a second struct with the same members: choose whichever is plainer. `check.rs` is
now in `files`. There, `check::limits` finds the name in `FETCH_NAMES` before it calls
`limits::fetch_bytes`, and every other name still goes to `apply`. Keep that hunk to the
lookup: `limits`' signature, its `(Limits, u64)` result and `Checked` stay as they are.
`reference` renders the fetch row in the setup's section, beside `NAMES` and the host's.
The setup crate's `every_row_is_a_name_the_parser_takes` covers the fetch row too, and
reads its value from what `check::limits` returns after the override is applied. C3 in
`notes/contracts.md` says this too.

