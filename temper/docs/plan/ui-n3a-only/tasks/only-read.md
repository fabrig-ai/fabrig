+++
id = "only-read"
part = "read"
kind = "interface"
needs = []
files = ["temper/crates/temper-client-domain/src/pages/only_you.rs", "temper/crates/temper-client-domain/src/pages.rs", "temper/crates/temper-client-domain/src/pages/parts.rs", "temper/crates/temper-client-domain/src/tests/pages/only_you.rs", "temper/crates/temper-client-domain/src/tests/pages.rs", "temper/crates/temper-client-domain/src/tests/pages/parts.rs"]
effort = "high"
minutes = 70
+++

## Goal

Give the client domain Only you's read: `pages::only_you`, the pure rule
that lists each of the person's own accounts whose deciding source is a
sign-in or a program's login signed out or read as expiring, and the
`only` fact and part of Needs you's page parts
(`temper/docs/design/client/README.md`, 3.5, **Only you**, *Its read*
and *It shows*; *A page's parts*). It draws nothing.

Acceptance:

- `pages/only_you.rs` holds `Why`, `Item`, `OnlyYou`, `items` and `read`
  exactly as `notes/contracts.md`, section 1 gives them, its rule that
  section's; `pages.rs` gains `pub mod only_you;` beside `pub mod
  needs;`.
- `pages/parts.rs`'s `NeedsFacts` gains `only` and `NeedsParts` gains
  `only` (contracts, section 2): `needs` sets `only: facts.known &&
  facts.only > 0`, `needs_facts` fills `only` from `only_you::read(root)`,
  and the `Default`s give none. `list`, `done` and `decided` are as they
  were.
- `src/tests/pages/only_you.rs`, one table test of `items`, a row a
  document and the items expected: none delivered as an empty document;
  the person's sign-in granted, no item; signed out, `SignedOut`;
  expiring, `Expiring`; expiring and signed out, `SignedOut`; a program's
  login signed out, `SignedOut` with `kind` `Program`; a key signed out,
  none; a `None` source, none; the deployment's account and another
  party's, none; the playground provider's account, none; the first
  source granted and the second signed out, none; two of the person's
  accounts, both items in the document's order; and a `cut` document,
  `cut` true.
- `src/tests/pages/parts.rs`'s `needs_parts_wait_for_known_and_keep_decided_independent`
  gains its rows: `only` with `known` false shows nothing; `only` with
  `known` true and no card open shows `done` and `only`; with a card open,
  `list` and `only`. Its existing rows keep their expectations, `only`
  false.
- The rule bites: drop the playground's check once, see its row fail,
  restore; the report names it.

## Inputs

- `temper/docs/plan/ui-n3a-only/notes/common.md`;
  `temper/docs/plan/ui-n3a-only/notes/contracts.md`, sections 1 and 2.
- `temper/docs/design/client/README.md`, 3.5, **Only you** (*Its read*,
  *It shows*), *A page's parts*'s table and its paragraph; 5.3 for the
  deciding source the rail's mark reads.
- Patterns: `temper-client-view/src/accounts/words.rs:683` (`mark`, the
  same walk over the document: holder, playground, deciding source);
  `temper-client-domain/src/pages/parts.rs` (the facts and parts);
  `temper-client-domain/src/tests/pages/accounts.rs:11` (an `Accounts`
  document's literals and constants).

## Owns and may change

- **Owns:** the files in this spec's header.
- **May change:** `temper/tests/client/tests/suite/fuzzy/density.rs`'s
  `check_rules`, its `parts::NeedsParts` literal only, with `only:
  needs.known && needs.only != 0`, the compiler's arm for the new field;
  any other literal of `NeedsFacts` or `NeedsParts` the compiler names,
  its new field only.
- **Options, pre-approved:** if a fixture for an `Accounts` document
  already exists in the domain's tests (`tests/fixtures.rs`,
  `tests/playground.rs`), build the rows on it; otherwise write the
  literals in `tests/pages/only_you.rs`. If `items` needs the provider's
  `Option<u32>` compared with `playground: Option<u32>`, compare them as
  `mark` does (`playground.is_some() && account.provider == playground`).

## Steps

1. `pages/only_you.rs` and its `mod` line; the parts' fields.
2. The table and the parts' rows; break the playground check once.
3. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo
   clippy -p temper-client-domain --all-targets -- -D warnings`; `heavy
   cargo nextest run -p temper-client-domain --lib`; `heavy cargo check
   -p temper-client-view --tests`; `heavy cargo check -p
   temper-client-world --tests`; `heavy cargo nextest run -p
   temper-client-world --profile fuzzy --test suite -E
   'test(=fuzzy::density::random_facts_and_settings_keep_each_part_and_card)'`.
   Commit and push.

## Report

As the implementer's text asks; name any literal outside the files above
the compiler made you touch.
