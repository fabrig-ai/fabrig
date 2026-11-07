+++
id = "facts-line"
part = "line"
kind = "interface"
needs = ["facts-record"]
files = ["jig/crates/jig-domain-core/src/documents/accounts.rs", "jig/crates/jig-domain-core/src/documents/deployment.rs", "jig/crates/jig-domain-core/src/tests/documents/accounts_document.rs", "jig/crates/jig-protocol-engine-clients/src/translate/accounts.rs", "jig/crates/jig-protocol-engine-clients/src/translate/deployment.rs", "jig/crates/jig-protocol-engine-clients/src/tests/accounts.rs", "jig/crates/jig-client-protocol-wire/schema/client-v19-part1.schema", "jig/crates/jig-client-protocol-wire/src/generated/v19.rs", "jig/crates/jig-client-protocol-wire/golden/**", "jig/crates/jig-client-protocol-link/src/translate/accounts.rs", "jig/crates/jig-client-protocol-link/src/tests/owner_accounts.rs", "jig/crates/jig-client-domain/src/contract/accounts.rs", "jig/testing/jig-client-test-records/src/**", "temper/docs/design/client/journeys/**"]
effort = "high"
minutes = 170
+++

# An account's row carries its facts, each kind's audience applied

## Goal

The party's `accounts` document and the deployment's carry each account's facts, from the store's record to the client's contract. Only what this build registers goes, and a holder's kinds go to its holder alone. Nothing writes a fact yet but a restore, so no page changes.

Acceptance:

- Section 6 of the contracts note, exactly.
- `jig-domain-core`, `tests/documents/accounts_document.rs`, a table over a restored account's facts:
  - each registered kind reaches its holder's line, whole, with its label, value, `seen`, source and origin;
  - a fact of an unregistered kind, origin or window state reaches no line, and stays in `Domain::account_facts`;
  - `fact_lines(.., holder_reads: false)` leaves kinds 3, 4 and 7 and drops 1, 2, 5 and 6.
- The deployment's row: a reader who may decide on the deployment's accounts gets every registered kind; any other reader with a deployment role gets the account's kinds alone. A row of the same table or its sibling.
- `jig-protocol-engine-clients`, `account_line_terms_reach_the_wire`'s test or a row beside it: facts reach the wire for both rows, each value shape.
- `jig-client-protocol-link`: facts reach `c::AccountLine` and `c::Account`; a share or a window's `used` past 10000 is a `w::Problem`; `charge_account`'s room counts each fact's label and text, so that a line past the room is refused as today's are.
- `c::AccountLine::email` and `c::Account::email`: a table: an email fact gives its bytes; none, or a kind 1 fact whose value isn't text, gives `None`.
- A test in the lowest crate that sees both pins `c::FACT_EMAIL == accounts::KIND_EMAIL` and `c::FACT_NAME == accounts::KIND_NAME`; if no crate does, a row in `jig/tests/client-protocol`.
- `jig/tests/client-wire`'s fuzzy `check!(AccountLine, bytes)` reads records with a full facts list: `jig_client_test_records::account_line_record()` fills one, and the deployment's likewise.
- The client wire changes, so journey recordings go stale. Run `heavy cargo run -p temper-client-journeys`. For each journey it names stale, run `heavy cargo run -p temper-client-journeys -- again NAME`, as `common.md` says, then the focused replay checks. Read the print's diff: this task changes no page, so a difference beyond times and ports is a red.
- The focused and fuzzy tests of every package named above, and `temper-client-view`, `temper-client-domain`, `temper-client-tools` and `temper-client-shell-terminal`, pass, and `heavy cargo check --workspace --tests`.

## Inputs

- `jig/docs/design/client/domain/contract.md`, 5.1, *the party's accounts* and the paragraph on an account's row's facts.
- `jig/docs/design/domain/accounts.md`, 3.7.
- `temper/docs/design/client/journeys.md`, 8.1.
- `temper/docs/plan/account-email/notes/contracts.md`, 3 and 6.
- `temper/docs/plan/account-email/notes/common.md`.

## Owns and may change

Owns the header's files. Readers, each edited only for the field:

- `d::AccountLine` and `d::Account` literals: `jig-domain-core`'s and `jig-protocol-engine-clients`' tests, `temper/crates/temper-service/src/tests/clients_unwatch.rs`.
- `c::AccountLine` and `c::Account` literals gain an empty `facts`: `jig-client-protocol-link/src/tests/owner_accounts.rs`; `temper-client-view/src/tests/{accounts,frame,fixtures,choice}.rs`; `temper-client-domain/src/tests/pages/{paste,accounts,connect_model}.rs` and `tests/playground.rs`; `temper-client-tools/src/tests/fixtures/accounts.rs`; `temper-client-shell-terminal/src/tests/fixtures/chat.rs`; `temper/tools/gallery/src/worlds.rs`; and any other the compiler names.
- `w::Limits` literals: `jig/testing/jig-client-test-records/src/lib.rs` and any other the compiler names. These lists are capped by the schema at 24, so `jig-client-protocol-link/src/limits.rs`'s `narrow` needs no line.

Options, each pre-approved:

- `List` has no empty constant in a literal's reach: use `List::with_capacity(0)`.
- A fact is whole: the document's `text_bytes` doesn't cut it, and neither `cut` flag counts it. Don't route it through `small::text` or `DocumentText`.
- `holder_reads` for the deployment's row is the check `routing/party/key_routes.rs`'s `admitted` makes for `Holder::Deployment`. If that check needs a request's context it hasn't got, take its people's rule alone, and name it in the report.
- A heap or decoded-bound test the wider lines trip takes the new figure, stated in the report with its figure before.

## Steps

```rust
// documents/accounts.rs
/// An account's facts as this reader may see them (client/domain/contract.md, 5.1).
pub(super) fn fact_lines(core: &Core, account: u32, holder_reads: bool) -> List<accounts::AccountFact>;
```

The party's `accounts` calls it with `true`: the document holds only the party's own. Regenerate the wire with `common.md`'s command. Translate on each side as `sources` and `uses` are, a span by its `Duration`, a wall time by `Wall`. `jig-client-domain` names no jig-domain crate: it has its own vocabulary, as for its other kinds.

Each new test breaks its rule once by hand. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py` and `temper/scripts/old_forms.py`; `heavy cargo clippy` on the changed packages `--all-targets -- -D warnings`; `heavy cargo nextest run` on them and their direct dependents, one command at a time. Commit and push.

## Report

As the implementer's text says, with the journeys recorded again and what each print's diff held.

## Answers

2026-11-07 (temper/ui). Accept: generated ids that renumber, with their label references still paired, are recording noise, as times and ports are. Markup, text and attributes being identical otherwise is what matters. Finish.

2026-11-07 (temper/ui). Yes, as answered at 07:40Z: this amends the acceptance. It now reads: a difference beyond times, ports and generated ids is a red. Your read-only comparison already shows nothing else differs. So the journey acceptance is met: don't stop on it again, and finish the task's remaining acceptance. Done, if that's the last.
