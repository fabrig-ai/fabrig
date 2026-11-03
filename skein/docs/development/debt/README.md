# skein's tech debt

What skein's code and tests owe, one file per item. Read it with
`skein/docs/foundation/programming-model.md` and
`skein/docs/foundation/testing-strategy.md`, whose rules most items
measure the code against.

## 1. The rule

- **One file per item,** `Dnnn-<slug>.md`. IDs start at `D001`, are
  skein's own, and are never reused.
- **The branch that pays an item deletes its file** and its row in the
  index below; git keeps the history.
- **An item we choose to keep** stays, with status `kept` and its
  reason, so a later review does not find it again.
- **Dated review reports** are kept raw under `reviews/`, as
  `YYYY-MM-DD-<scope>.md`. Items cite them by file; reviews add items
  here.

## 2. What counts

Debt is what the code or tests owe: dead or duplicate code, coupling,
missing or hidden primitives, needless complexity, naming, hand-copied
lints, test infrastructure copied around, workarounds left in place,
files too large to read. Not here:

- **a design gap** (the code lags or departs from a design document):
  it goes in that document's status file. When an item is also a
  departure, the item covers the code's side and cites the design;
- **a bug** (wrong behaviour): it is fixed, or tracked where bugs are;
- **a feature not built yet.**

## 3. An entry

```md
# D007: <title, a short claim>

- **Kind:** dead code | duplication | coupling | primitives | complexity | naming | tests | lints | other
- **Where:** <paths; one that starts with a crate's name is under skein/crates/, others are from the repository root>
- **Seen:** <date> at `<commit>`, in <source, with its part> (one line per source)
- **Checked:** <date> at `<commit>`
- **Value:** high | medium | low (what paying it buys)
- **Size:** S (one task) | M (one plan of a few tasks) | L (several plans)
- **Status:** open | planned (<plan>) | kept (<reason>)
- **Paid when:** <a concrete, checkable condition>
- **Measure:** <optional: a command run at the repository's root, and its value when checked>

## What

## Why it matters

## What pays it back
```

A **Measure** is given only where a count tracks the debt; it runs at
the repository's root. **Paid when** is always given; for qualitative
debt it says what a reviewer checks.

## 4. Index

| ID | title | kind | value | size | status |
|---|---|---|---|---|---|
| [D001](D001-json-whole-buffer-reads.md) | Whole-buffer JSON reads are hand-driven, with no slice-fed driver | primitives | high | M | open |
| [D002](D002-index-and-length-widening.md) | Index and length conversions are written by hand beside lib's helpers | primitives | medium | M | open |
| [D003](D003-env-rewraps.md) | `Env` is rebuilt by hand where `Env::with` does it | primitives | medium | S | open |
| [D004](D004-utf8-checks.md) | UTF-8 is checked by a hand loop in codec and by `from_utf8` under expects | duplication | low | S | open |
| [D005](D005-civil-calendars.md) | The civil calendar is written three times | duplication | medium | S | open |
| [D006](D006-lru-tables.md) | Least-recently-used tables are hand-rolled in both forge dialects | primitives | medium | S | open |
| [D007](D007-joined-and-measure.md) | Byte joining and measuring helpers are copied per file | duplication | low | S | open |
| [D008](D008-http-head-parsing.md) | HTTP client and server parse message heads separately | duplication | medium | S | open |
| [D009](D009-oauth-credentials-raw.md) | The oauth crates hold credentials as raw bytes, not `Secret` | primitives | medium | M | open |
| [D010](D010-github-on-forgejo-modules.md) | GitHub's dialect stands on Forgejo's modules | coupling | high | M | open |
| [D011](D011-fake-github-on-forgejo-fake.md) | The fake GitHub stands on the fake Forgejo, which is named as the neutral fake | coupling | medium | M | open |
| [D012](D012-anthropic-on-openai.md) | Anthropic's dialect stands on `openai/`, with a mirror vocabulary | coupling | medium | M | open |
| [D013](D013-forge-connection-parallel-lists.md) | forge-connection keeps forges in parallel lists and dialects as runtime tags | complexity | high | M | open |
| [D014](D014-forge-setup-vocabulary.md) | forge-setup's shared vocabulary lives in Forgejo's reconcile, and `Reply` mixes dialects | coupling | medium | M | open |
| [D015](D015-flat-forge-enums.md) | Flat forge request, answer and node enums are re-matched in every module | complexity | medium | L | open |
| [D016](D016-transport-four-copies.md) | The socket, TLS and HTTP transport is written four times | primitives | high | M | open |
| [D017](D017-kv-store-states.md) | skein-kv's store keeps its lifecycle in flags, with the subset switched off | complexity | high | M | open |
| [D018](D018-io-pipe-and-file-states.md) | io's pipe, signals and file machines keep state in flags and literals | complexity | medium | M | open |
| [D019](D019-channel-machine-flags.md) | The channel machine keeps its lifecycle in eleven bools | complexity | medium | S | open |
| [D020](D020-forge-connection-states.md) | forge-connection's connections keep their state in flags beside a thin tag | complexity | medium | M | open |
| [D021](D021-forgejo-plan-states.md) | Forgejo's plans keep a `Stage` beside bags of fields, and measure answers twice | complexity | medium | M | open |
| [D022](D022-oauth-states.md) | The oauth client and accounts keep lifecycles in flags and parallel lists | complexity | medium | M | open |
| [D023](D023-llm-client-states.md) | skein-llm's client and llm-connection's call use data-less states | complexity | medium | M | open |
| [D024](D024-echo-template-flags.md) | The echo example, the template applications copy, keeps its stop in flags | complexity | high | S | open |
| [D025](D025-request-table-in-lib.md) | lib's `RequestTable` is a component with one consumer | coupling | low | S | open |
| [D026](D026-crate-tests-directories.md) | Three crates keep a `tests/` directory under `crates/` | tests | medium | S | open |
| [D027](D027-step-test-layouts.md) | Step tests follow several layouts, some through `#[path]` | tests | low | M | open |
| [D028](D028-worlds-as-kits.md) | World crates are used as kits by other test crates | tests | medium | M | open |
| [D029](D029-fake-github-http-faces.md) | Four HTTP faces for the fake GitHub service sit beside its own server | tests | high | M | open |
| [D030](D030-forge-connection-harness-tests.md) | forge-connection's step tests are mostly a machine world of their own | tests | medium | M | open |
| [D031](D031-forge-setup-two-worlds.md) | The forge setup world is written twice, once without the host | tests | medium | M | open |
| [D032](D032-world-harness-copies.md) | Machine-world harness pieces are copied between the protocol worlds | tests | medium | M | open |
| [D033](D033-limits-literals-in-tests.md) | Tests write out whole `Limits` and `Config` literals instead of one constructor | tests | low | M | open |
| [D034](D034-codec-double-generation.md) | tests/codec generates its fixtures twice, and skein-codec compiles one of them | tests | medium | S | open |
| [D035](D035-codegen-diff-and-emit.md) | The schema diff is written twice, and codegen edits its own output as text | duplication | medium | M | open |
| [D036](D036-lint-blocks-by-hand.md) | Lint configuration is copied by hand into 48 manifests | lints | medium | S | open |
| [D037](D037-one-word-several-things.md) | One name stands for several things in skein's kits and crates | naming | low | S | open |
| [D038](D038-comment-words.md) | Comments carry emphasis words, rulings and history | naming | low | S | open |
| [D039](D039-forge-client-section-citations.md) | forge-client's module headers cite the wrong design sections | naming | low | S | open |
| [D040](D040-dead-public-api.md) | Public items with no caller, or only a test caller | dead code | low | S | open |
| [D041](D041-workspace-duplicates.md) | The workspace lists nine skein crates twice and keeps an unused dependency | other | low | S | open |
| [D042](D042-shell-backend-copies.md) | skein-shell's Linux and Darwin backends copy their POSIX parts | duplication | low | S | open |
| [D043](D043-browser-kit-json-and-tables.md) | The browser kit keeps its own JSON decoder and `Vec` tables | primitives | low | M | open |
| [D044](D044-io-token-tags.md) | Hand bit tags work around io's and FileIo's colliding operation tokens | other | medium | S | open |
| [D045](D045-shell-keeper-syscalls.md) | skein-shell's public API carries the harness's syscalls | coupling | low | S | open |
| [D046](D046-ui-limits-and-accounting.md) | skein-ui offers no `Limits` constructor and hides its node accounting | primitives | medium | S | open |
| [D047](D047-role-sets-as-slices.md) | Role sets are written as slice `contains` checks | primitives | low | S | open |
| [D048](D048-oauth-accounts-world-copies.md) | The oauth-accounts world doubles its owner harness and switches on stories | tests | low | M | open |
