+++
id = "box-sign-in"
part = "scope"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-service/src/startup/application.rs",
  "temper/crates/temper-service/src/startup/problems.rs",
  "temper/crates/temper-service/src/tests/forge_sign_in.rs",
  "temper/crates/temper-service/src/tests/members.rs",
  "temper/docs/design/protocol/sign-in.status.md",
]
effort = "medium"
minutes = 30
+++

# A box's sign-in refuses `application.sign_in` beside it

## Goal

A box is its one person's, and temper is about to count that person's Forgejo account as theirs (`temper/docs/design/draft/forge-identity.md`, 9). A Google member signed in beside a box's sign-in could land on that account by name. So the start refuses `application.machine.owned.sign_in` that isn't `null` beside an `application.sign_in` that isn't `null`, at the first one's path, in the words "a box's sign-in takes application.sign_in null". Nothing refuses it today.

This task departs from the implementer's text in one way: one status entry is yours to remove (acceptance 3), because the design already says what this task builds.

Acceptance:

1. A document with both set is refused, its lines holding `application.machine.owned.sign_in: a box's sign-in takes application.sign_in null`; with either `null` the start goes on as today.
2. The single-case test `machine_owned_sign_in_must_name_a_provider_client_at_its_path` (`temper/crates/temper-service/src/tests/forge_sign_in.rs`) becomes a table of `(owned sign_in, application.sign_in, expected line)` rows holding today's "unknown reference" row and the new refusal; rename it for what it now holds (`machine_owned_sign_in_is_refused_at_its_path`). It bites: drop the new check once, see the new row fail.
3. The entry *9.9: the start doesn't refuse `machine.owned.sign_in` beside an `application.sign_in` that isn't `null`.* is gone from `temper/docs/design/protocol/sign-in.status.md`.
4. Focused tests of `temper-service` pass (`heavy cargo nextest run -p temper-service`), and `heavy cargo check -p temper-init --tests` passes; clippy clean for `temper-service`.

## Inputs

- `temper/docs/plan/ui-forge-identity/notes/common.md`.
- `temper/docs/design/protocol/sign-in.md`, 9.9, the bullets *Refused too, at the start, at `machine.owned.sign_in`'s path*: the second binds.

## Owns and may change

**Owned:** the `files` above.

**Readers and neighbours:**

- `temper/crates/temper-service/src/tests/members.rs`, `every_row_is_a_member_its_decoder_takes`: its "box sign-in decoder example" sets `application.machine.owned.sign_in` on the expanded document, which holds an `application.sign_in`. That example now also sets `application.sign_in` to `null` (by `fixtures::with` or its `serde_json` edit), and stays a decoder example of the box's member. Nothing else in that file changes.
- Any other test or fixture the new refusal trips (a document with both set): set `application.sign_in` `null` there, and name it in the report. `temper-init`'s box document writes `application.sign_in` `null` (sign-in.md, 9.9), so it shouldn't trip.

**Options, each with its default:**

- The problem: a new unit variant of `startup::problems::What`, default name `BoxSignIn`, its words in `words` and its arm in every exhaustive `match` on `What` the compiler names. If an existing variant already says exactly this, use it and say so.
- The check sits beside the existing `machine.owned.sign_in` check in `application.rs` (the `if let Some(machine) = &machine && let Some(owned) = &machine.owned && let Some(client) = &owned.sign_in` block), reading `sign_in.is_some()`. Default: push it in that block, after the client lookup.

## Steps

1. The variant, its words, the check.
2. The table test (acceptance 2), building each row's document with `super::stock::generated()` and `fixtures::with`, as the test does today. For the refused row, a Google sign-in as `sign_in_members_login_refuses_the_bot_without_any_forge_entries` builds it (providers, owners and `application.sign_in`), with an `owned` whose `sign_in` names a client the provider has, so that the only line is the new one; assert the line is present.
3. The `members.rs` example (above), the status entry.
4. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-service --all-targets -- -D warnings`; `heavy cargo nextest run -p temper-service`; `heavy cargo check -p temper-init --tests`. Commit ("temper-service: a box's sign-in refuses application.sign_in beside it, as a box is its one person's"), push.

## Report

As the implementer's text says, with the rule broken for the bite and the failure seen.
