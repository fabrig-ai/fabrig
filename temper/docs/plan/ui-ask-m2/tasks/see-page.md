+++
id = "see-page"
part = "page"
kind = "feature"
needs = ["see-render-runs", "see-route"]
files = ["temper/crates/temper-engine-smith/src/see.rs", "temper/crates/temper-engine-smith/src/see/**", "temper/crates/temper-engine-smith/src/reference.rs", "temper/crates/temper-engine-smith/src/calls.rs", "temper/crates/temper-engine-smith/src/lib.rs", "temper/crates/temper-engine-smith/src/tests/**", "temper/crates/temper/src/tests/commands.rs", "temper/crates/temper-docs/src/lib.rs", "temper/docs/help/reference/see.md", "temper/docs/help/reference/host-tools.md", "temper/crates/temper-engine-domain/src/route/see.rs", "temper/crates/temper-engine-domain/src/decision.rs", "temper/docs/design/protocol/documentation.status.md", "temper/docs/design/domain/agent.status.md"]
effort = "medium"
minutes = 100
+++

## Goal

Give `see` its reference page and one table of its addresses
(`temper/docs/design/domain/agent.md`, 8.4.1, *Its declaration* and *On
its page*; `temper/docs/design/protocol/documentation.md`, section 8's row
`see`): a row an address, its form, its line and the addresses one level
down, which renders `docs/help/reference/see.md` and gives the unknown
address's line and each answer's `see:` line; and the declaration's new
description. `SeeAnswer::NotYet`, which nothing delivers once `see-route`
has landed, goes.

Acceptance:

- `temper-engine-smith` holds one table of `see`'s addresses, in
  `ask.txt`'s order (`me` first, `more TOKEN` last, the settings' members
  each a row): its form as the instructions' line names it, a line on what
  it answers (8.4.1's *Answers*, shortened to at most 160 bytes), and its
  following addresses (8.4.1's `see:` column). `following` takes each
  answer's `see:` line from it, and `ADDRESSES` is held equal to the forms
  the instructions' line joins, by a step test where it stays a `&str`.
- `temper_engine_smith::see_reference(out: &mut Writer) -> Result<(),
  Overflow>` renders the page `see`: `# Ask temper's addresses`, `Kind:
  reference`, a paragraph on what `see` is and how `more` reads on, then a
  section a row, `## FORM`, its line, `Next: ` and its following
  addresses. `temper/docs/help/reference/see.md` is that rendering,
  registered in `temper-docs`' `PAGES` (name `see`, kind reference, a title
  and summary), and `temper`'s step test
  `the_page_see_is_what_the_addresses_render`, beside
  `the_page_host_tools_is_what_the_declarations_render`, holds it equal,
  writing it under `SKEIN_CODEGEN_WRITE=1`.
- `see`'s declaration's description is 8.4.1's, "Read temper by address,
  as your person, in your project: its work, changes, settings, proposals,
  pages and files. Each answer ends with what to read next." (154 bytes);
  `host-tools.md` regenerated; `asks_first_request_fits_both_shapes`
  (`temper-service`'s `tests/ask_bound.rs`) still holds 16,384 bytes.
- `SeeAnswer::NotYet` and its arms (`temper-engine-domain`'s
  `route/see.rs` and `decision.rs`; `temper-engine-smith`'s `see.rs` and
  its test rows) are gone.
- `temper/docs/design/protocol/documentation.status.md`: the **Not yet**
  entry on 8 is deleted; `temper/docs/design/domain/agent.status.md`: the
  **Not yet** entry *Its declaration* and *On its page* is deleted.

## Inputs

- `temper/docs/plan/ui-ask-m2/notes/common.md`.
- `temper/docs/design/domain/agent.md`, 8.4.1 (the address table, *Its
  declaration*, *On its page*, *Unknown*);
  `temper/docs/design/protocol/documentation.md`, section 8 and section 6's
  `temper` row; `temper/docs/design/draft/agent-docs.md`, 8.1.
- `temper/crates/temper-engine-smith/src/reference.rs` (the host-tools
  page), `src/calls.rs` (`SEE`, `DESCRIPTION_BYTES`), `src/see.rs`
  (`ADDRESSES`, the constants and `following`);
  `temper/crates/temper/src/tests/commands.rs`;
  `temper/crates/temper-docs/src/lib.rs` (`PAGES`, `MAX_PAGES`);
  `temper/crates/temper-service/src/stock/ask.txt` (the line of addresses,
  which this task does not change).

## Owns and may change

- **Owns:** the files above, for this: the table and its uses, the
  page's renderer and its step test, the page and its registration, the
  description, the variant's removal, the two status entries.
- **Generated:** `host-tools.md` and `see.md`, by their step tests under
  `SKEIN_CODEGEN_WRITE=1`, never by hand.
- **Options, pre-approved:** if `temper-docs`' bounds or index test needs
  the page named elsewhere (the `index` page's reference list), add its
  line there and say so. If a `see:` line of today's differs from 8.4.1's
  column, take the column's and name the change.

## Steps

1. The table, `following` and `ADDRESSES` from it; the existing see
   tests pass unchanged but for rows whose `see:` line the column changed.
2. The renderer, the page, its registration and its step test.
3. The description; regenerate `host-tools.md`; the bound test.
4. `SeeAnswer::NotYet` removed.
5. The status entries.
6. `cargo fmt`; `heavy cargo clippy -p temper-engine-smith -p
   temper-engine-domain -p temper-docs -p temper --all-targets -- -D
   warnings`; `heavy cargo nextest run -p temper-engine-smith --lib`;
   `heavy cargo nextest run -p temper-docs`; `heavy cargo nextest run -p
   temper --lib -E 'test(/page|reference|help/)'`; `heavy cargo nextest run
   -p temper-service --lib -E 'test(/ask_bound|stock/)'`; `heavy cargo
   nextest run -p temper-engine-domain --lib -E 'test(/see/)'`. Commit and
   push.

## Report

As the implementer's text asks; the page's bytes and the first request's
figures on both shapes.
