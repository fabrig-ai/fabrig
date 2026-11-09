+++
id = "see-surface"
part = "answers"
kind = "interface"
needs = []
files = ["temper/crates/temper-engine-domain/src/route/see.rs", "temper/crates/temper-engine-domain/src/tests/see.rs", "temper/crates/temper-engine-domain/src/lib.rs"]
effort = "medium"
minutes = 60
+++

## Goal

Give temper's root the shapes `see`'s M2 addresses cross with: the
address grammar of `temper/docs/design/domain/agent.md`, 8.4.1's rows `me`
to `inbox`, each parsed to its own `Address`, and `SeeAnswer`'s new
variants and records, so that the root's reads (`see-route`) and the
renderer (`see-render`) are built against them at once. Behaviour stays
as it is: every new address still answers `see ADDRESS: not there yet.`

Acceptance:

- `Address` and `SeeAnswer` gain exactly the variants, and the records
  `SeePerson`, `SeeRead`, `SeeNotRead`, `SeeNames` and `SeeName` exist, as
  `temper/docs/plan/ui-ask-m2/notes/contracts.md`, section 2 gives them,
  with their doc comments; the public ones are exported from the crate's
  root beside `SeeAnswer` (`lib.rs`'s `pub use` of `route::see`'s items).
- `address` parses each of 8.4.1's forms to its variant: `me`, `project`,
  `goals`, `goals ended`, `tasks`, `task N`, `task N tree`, `task N
  activity`, `task N turn A.T`, `change N`, `changes`, `inbox`; N and A a
  positive `u64`, T a positive `u32`; any other word, a zero, an overflow
  or a missing part is `Address::Unknown`. `Address::NotYet` stays, parsed
  from nothing, until `see-route` removes it.
- `call` routes each new address to `SeeAnswer::NotYet`, as today, so no
  answer changes.
- The forced arms: `decision.rs`'s `host_within` admits `Person` and
  `NotRead` as it admits `Ways` and `Settings`; `route/ways/view/address.rs`
  and `route/see/forge.rs` treat the new addresses as they treat the other
  addresses that are not theirs; `temper-engine-smith/src/see.rs`'s
  `see_answer` answers `SeeAnswer::Person(_)` as `NotYet` is answered and
  `SeeAnswer::NotRead(why)` with 8.4.1's refusal, `see ADDRESS: not read:`
  then `nothing you may read in this project is there.`, `temper is busy;
  read it again.` or `this chat has no person to read as.`, an error, and
  `following` treats both as errors (no `see:` line).
- `addresses_parse_in_the_chats_project` holds a row for each form above,
  replacing its `NotYet` rows, and rows that stay unknown: `task 0`, `task
  x`, `task 41 turn 1`, `task 41 turn 0.2`, `task 41 turn 1.0`, `task 41
  turn 1.2.3`, `task 41 tree now`, `goals done`, `tasks 3`, `me now`,
  `change 18446744073709551616`, `inbox all`.

## Inputs

- `temper/docs/plan/ui-ask-m2/notes/common.md` and
  `temper/docs/plan/ui-ask-m2/notes/contracts.md`, section 2.
- `temper/docs/design/domain/agent.md`, 8.4.1: the address table's rows `me`
  to `inbox`, *A refusal* of an address read as the person.
- `temper/crates/temper-engine-domain/src/route/see.rs` (the grammar,
  `natural`, `small`, `turn_number`), its readers named below, and
  `temper/crates/temper-engine-domain/src/tests/see.rs`.

## Owns and may change

- **Owns:** `route/see.rs`'s `Address`, `SeeAnswer`, the new records and
  `address`; the parser test; `lib.rs`'s export line.
- **Forced arms only,** each edited only for that:
  `temper/crates/temper-engine-domain/src/decision.rs` (`host_within`),
  `temper/crates/temper-engine-domain/src/route/ways/view/address.rs`,
  `temper/crates/temper-engine-domain/src/route/see/forge.rs`,
  `temper/crates/temper-engine-smith/src/see.rs` (`see_answer` and
  `following`, as the acceptance says, nothing else), and any other match
  the compiler names.
- **Options, pre-approved:** `turn_number` may become a function returning
  the pair, `(u64, u32)`. If a record's field type does not derive what
  `SeeAnswer` derives (`PartialEq, Eq, Hash, Debug`), box it or wrap it and
  say so; never drop a derive from `SeeAnswer`.

## Steps

1. The variants and records; the grammar; `call`'s routing to `NotYet`.
2. The forced arms.
3. The parser rows; break one form (parse `goals ended` as `goals`) to
   see its row fail, and restore.
4. `cargo fmt`; `heavy cargo clippy -p temper-engine-domain -p
   temper-engine-smith --all-targets -- -D warnings`; `heavy cargo nextest
   run -p temper-engine-domain --lib -E 'test(/see/)'`; `heavy cargo nextest
   run -p temper-engine-smith --lib -E 'test(/see/)'`; `heavy cargo nextest
   run -p temper-engine-domain-world --test suite -E
   'test(/an_asking_chat_reads_each_address/)'`. Commit
   and push.

## Report

As the implementer's text asks.
