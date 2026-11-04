+++
id = "making-run"
part = "making-service"
kind = "feature"
needs = ["making-root", "copy-machine", "copy-words", "fake-migrate"]
files = ["temper/crates/temper-service/src/setup.rs", "temper/crates/temper-service/src/setup/lifecycle.rs", "temper/crates/temper-service/src/setup/making.rs", "temper/crates/temper-service/src/setup/runtime.rs", "temper/crates/temper-service/src/family.rs", "temper/crates/temper-service/src/route.rs", "temper/crates/temper-service/src/tests/setup_making.rs", "temper/crates/temper-service/src/tests/family.rs", "temper/crates/temper-forge-setup/src/limits.rs"]
effort = "high"
minutes = 75
+++

# The service runs what the connector holds

## Goal

A request from a client reaches the connector, its row runs its plan one writing run at a time, and the
plan's end comes back, as `temper/docs/design/protocol/setup.md`, 6.9 says (contracts T5). This is the
plan's meeting point of temper and skein.

Acceptance, in a new `temper-service/src/tests/setup_making.rs`, on `setup_create.rs`'s fake host:

- `a_copy_runs_on_a_connected_forge_and_is_made`; `an_empty_making_runs_create_with_the_starter`.
- `a_row_waits_for_the_forges_writing_run`, and `one_row_runs_at_a_time`.
- `a_row_waits_for_its_forge_to_connect`, and `an_excluded_forge_keeps_its_rows`.
- `copying_is_answered_from_the_row`: `Read` gives none, then the byte after the mark; `Write` is answered
  `Written` only after `MakingCopied`.
- `copying_never_reaches_the_records`.
- `said_words_reach_the_row_on_the_stop`; `taken_empty_and_timed_out_are_typed`.
- `a_busy_root_does_not_lose_an_end`: `root::Request::Busy` gives a `Making*` event back; the arm that
  drops `FirstMade` (`route.rs`, 111) is not its arm.
- `a_restart_runs_the_row_again`, and `a_made_row_asks_its_adoption_once_connected`.
- `the_label_is_the_records_or_the_modes`: `first.label`, or `temper-host` on owned, `docker` on managed.
- `family.rs`: `MakeRepository` decodes; its `host` is filled as `Adopt`'s; each outcome encodes. In
  `src/tests/family.rs`.
- `heavy cargo nextest run -p temper-service -p temper-forge-setup` pass; `heavy cargo check --tests -p
  temper -p temper-one-process-world -p temper-real-loop-world` passes.

## Inputs

- `temper/docs/plan/repository-copies-1/notes/common.md`; `notes/contracts.md`, S2 and T5.
- `temper/docs/design/protocol/setup.md`, 6.7 and 6.9.
- skein's `skein/docs/design/forge-setup.md`, 4.10 and 6.3.

## Owns and may change

- The files of the header. Put the new code in `setup/making.rs`: `debt-11` and `debt-10` are in
  `setup.rs`, `setup/runtime.rs` and `route.rs`; keep your lines there few.
- **The pull:** `lifecycle::advance`, 821, past its `busy` skip: `connected_in_root` (231), then, when the
  connector's `next_making` has a row, `Event::MakingStarted`, then `run_plan` (`setup.rs`, 758) with a
  `Run { kind: Kind::Making, internal: true, .. }`, the row's key held on the `Run` beside it, `Kind`
  being `Copy`. The limits come from `Runtime`
  (`runtime.rs`, 195), never `Local`, which `stopped_connection` removes. `check_copy` and `check_create`
  are asserted, as `create_local` (1120) does; a row's fields were checked at admission.
- **A pulled adoption:** `making_due`, then `Event::MakingAdopt`, in the same place.
- **`Kind::Making`'s arms** (`setup/lifecycle.rs`): `kind` (260), `done` (342), `stopped` (451). `Stage` to `MakeStep`,
  `Why` to `MakeFailure`: `Taken`, `Empty`, `TimedOut`, `Refused(status)`, anything else `Other`, with
  words: what `Event::Said` said, held on the `Run`; else the words the setup gives that `Why` (`public.rs`,
  601).
- **`Item::Copying`:** `read_item` and its write answer it here, before the item reaches
  `temper-protocol-records`. `copy-surface`'s "does not keep" arm is replaced.
- **`Request::MakingCopied`** answers `Written`, by the run's key.
- **`copy_deadline`:** `temper-forge-setup/src/limits.rs`, as a `small` constant, if `copy-surface` left it
  in `plan()`'s literal.
- **`temper-service/src/family.rs`:** the arms in `to_root` (54), `answer` (586) and the row (`forges`, 784,
  `ForgeRowParts` at 826). `making-wire`'s refusal of the request is replaced. Its address is parsed with
  `source::parse`, and `address` gives the kept form.
- Remove "6.9: ..." from `temper/docs/design/protocol/setup.status.md`, and "4.6, 8: ..." from
  `temper/docs/design/client/README.status.md`.
- Pre-approved:
  - A run that stops `Busy` here is a bug of the pull's: assert in a test it never happens, and treat it
    as `Waiting` if it does.
  - `Lifecycle::forget` (138) needs nothing, unless you keep a queue.

## Steps

1. `family.rs`. 2. `setup/making.rs`: the pull and the plans. 3. `Copying` and `Said`. 4. The ends. 5. The
tests, each biting: answer `Written` at once, and start a row on a busy forge, each once. 6. The
implementer's checks; commit and push.

## Report

As the implementer's text asks; what a forge's other writing runs do while a long copy lasts.
