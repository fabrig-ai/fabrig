# D007: `temper-docs` keeps a page kind no page has

- **Kind:** dead code
- **Where:** `temper-docs/src/lib.rs`, `temper-docs/src/tests.rs`, `temper-docs/src/tests/rules.rs`, `temper-docs/src/tests/checks.rs`, `temper-docs/src/tests/paging_search.rs`
- **Seen:** 2026-11-02, in `temper/docs/design/protocol/documentation.status.md` (its Departs entry for 2.2 and 3): "a code task removes it"
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** low (one kind fewer; the voice and placeholder rules exempt nothing)
- **Size:** S
- **Status:** open
- **Paid when:** `temper_docs::Kind` has no `Guide`, no test builds a guide fixture, and the status entry is gone.

## What

The page kinds are those of `temper/docs/design/protocol/documentation.md`,
2.2; `temper-docs` still declares `Kind::Guide` (`src/lib.rs`, its name
and its page bound), with no page of that kind under `temper/docs/help/`.
Its tests exempt guides from the voice and placeholder rules
(`src/tests/rules.rs`, `src/tests/checks.rs`) and use guide fixtures for
paging (`src/tests/paging_search.rs`); `src/tests.rs` lists six kinds
and six "transitional" guide names (`GUIDES`), which a test checks every
guide against. The help's 55 pages are concepts, references, walks, one
recipe and the index (`grep -rh '^Kind:' temper/docs/help`).

## Why it matters

A rule's exemption for a kind nobody writes is a hole a new page could
fall through.

## What pays it back

Remove the variant, its arms and the exemptions; move the paging tests'
fixtures to a kind that exists. The help tests then cover the change
(`temper/AGENTS.md`).
