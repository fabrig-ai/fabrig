+++
id = "sv-skein-retire"
part = "rf"
kind = "interface"
needs = []
files = ["skein/crates/skein-forge-client/**"]
effort = "high"
minutes = 70
+++

## Goal

skein's forge client retires one forge on its owner's word, without disturbing the others
(the answer to `fe-remove`'s question, 2026-11-01): `Event::Retire { forge: u16 }` drops the
forge's writer and settles every write the client holds on it, so that an owner removing a
forge (temper's `RemoveForge`, draft `temper/docs/design/draft/playground-reset.md`,
section 4) can erase its rows knowing the client will never again speak of that forge's
entries. skein only: nothing here names temper.

Why it is skein's: the client's writers and outbox are private (`src/domain.rs`, `Domain`),
and `Withdraw { entry }` changes nothing for an entry with an attempt
(`skein/docs/design/forge-client.md`, 7.2), so a sent or uncertain write on a forge that
can no longer answer would back off for ever (`src/outbox/retry.rs`, an uncertain entry's
`backoff`), and an owner that erased its row beside it would then meet the client's
`Progress`, `Outcome` or `Settled { left }` for an entry it no longer holds.

Acceptance:
- `skein_forge_client::Event` gains `Retire { forge: u16 }`; its doc comment says what this
  spec says.
- In the step that takes it:
  - the forge's writer is dropped from `Domain::writers`, so a later `Make` on it is refused
    `Failed(Forbidden)` as on a forge with no writer (7.2, admission); the client emits no
    `Erase` for it (the writer's durable row is the owner's to erase);
  - each outbox entry whose repository is on the forge, in entry-number order, whatever its
    phase, attempt or uncertainty, with its alarm cancelled and its queued calls cancelled
    (`calls::cancel_entry`): when no call of it is out, it settles at once, one
    `Request::Outcome { entry, task, outcome: Outcome::Failed(Error::NotConnected) }`, and
    the client holds it no more; when a call of it is out, it is marked retired and settles
    so at that call's one terminal, whatever the answer (no echo, no recovery, no further
    call), as a withdrawn entry with a call out settles at its answer today;
  - no `Progress` is emitted for a retired entry, then or later.
- Afterwards nothing the client emits names a retired entry: not `Progress`, not a second
  `Outcome`, not `Settled { left }`; a stop (7.8) and `SettleOutbox` treat it as gone; the
  lanes it held free at once (or at its terminal, when a call was out), so other forges'
  entries, calls, alarms, keeps and budgets are untouched.
- `Retire` of a forge with no writer and no entry changes nothing and says nothing; it is
  taken before and after restore ends (it acts on what the client holds then).
- Live resources and cached repository records of the forge are not this event's: the owner
  drops them by its next `Keep` without the forge's watches, as today.
- Unit tests in `src/tests.rs` (or a new `src/tests/retire.rs` if the crate's layout takes
  one), each by name:
  `retiring_a_forge_settles_its_unsent_and_uncertain_entries_at_once`;
  `a_retired_entry_with_a_call_out_settles_at_its_answer_whatever_it_is`;
  `a_retired_forge_has_no_writer_and_refuses_a_make`;
  `retiring_one_forge_leaves_another_forges_entries_and_calls`;
  `nothing_names_a_retired_entry_afterwards` (a later `SettleOutbox`, a stop, and the
  retired entry's former alarm firing);
  `retiring_an_unknown_forge_says_nothing`.
- Every existing test of the crate passes.

## Inputs

- `skein/docs/design/forge-client.md`, sections 2, 7.1, 7.2, 7.4, 7.8, 8.
- `skein/crates/skein-forge-client/src/outbox.rs` (`withdraw`, `finish`, the `withdrawn`
  flag) and `src/calls.rs` (`cancel_entry`, `answered`): the model is `Withdraw`'s, widened
  to entries with an attempt and to a whole forge.
- `skein/docs/foundation/programming-model.md`, section 1.
- `temper/docs/plan/playground-reset-r6/notes/common.md`.
- `skein/AGENTS.md`.

## Owns and may change

- Owns: `skein/crates/skein-forge-client/` (all of it).
- May change, call sites only: any skein or temper file the compiler names because
  `Event` gains a variant (a reader matching `skein_forge_client::Event` exhaustively): add
  the arm that forwards or ignores it, nothing else, each listed in the report.
- Not `skein/docs/design/`: the report proposes forge-client.md's new text (7.2 gains
  "Retirement" beside "Withdrawal"), and the lead writes it.

## Steps

1. `Event::Retire` and its step, as above.
2. The tests above.
3. Run, one at a time: `heavy cargo nextest run -p skein-forge-client --lib`; `heavy cargo
   check --workspace --all-targets`; `cargo fmt --all`; `heavy cargo clippy -p
   skein-forge-client --all-targets -- -D warnings`; `python3 -I -B scripts/old_forms.py`
   from `skein/`.
4. Commit, push your branch.

## Report

- forge-client.md 7.2's proposed "Retirement" paragraph.
- Every file outside the owned crate you edited, one line each.
