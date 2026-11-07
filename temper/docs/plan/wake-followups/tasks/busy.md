+++
id = "busy"
part = "busy"
kind = "feature"
needs = []
files = ["smith/crates/smith-domain-run/src/run/relay.rs", "smith/crates/smith-domain-run/src/tests/host.rs"]
effort = "high"
minutes = 90
+++

# wake-followups: a host call answered busy is asked again after a growing backoff

## Goal

In a live run, temper refused one task's forge reads (`read_pull`, `read_pull_files`,
`read_checks`, `read_statuses`) with `read admission busy` about 434 times with no visible
backoff. A fresh read refused `Busy` by the forge client's read admission answers the host's
Busy, and the agent's host asks the same call again after its backoff, within the caller's
deadline (`temper/docs/design/domain/forge.md`, 5.3; the engine's domain world's
`reviewer_reads_back_off_when_forge_admission_is_busy` holds that half). What asks again is
smith's relay, and it waits a fixed `host_backoff`, 50 ms, before every ask, so a host that
stays busy for a minute is asked about twelve hundred times. The relay's backoff now doubles
with each ask, to a cap (`smith/docs/design/domain/run.md`, 5.2; `notes/contracts.md`,
section 3). The failing test is smith's, the lowest tier that shows the fault
(`AGENTS.md`, 5): a host read answered busy again and again, then answered.

Acceptance:

- `notes/contracts.md`, section 3, holds in `host_recover`
  (`smith-domain-run/src/run/relay.rs`): the wait before attempt `n + 1` is `host_backoff`
  times `2^(n - 1)`, at most `HOST_BACKOFF_GROWTH` (64) times `host_backoff`, in the strict
  subset's arithmetic (saturating, no `as`); an ask that would start at or past the caller's
  deadline is not made and the call returns as today.
- `a_host_call_busy_many_times_keeps_its_name_until_it_succeeds`
  (`smith-domain-run/src/tests/host.rs`) becomes a table over its eight busy answers, each
  row the attempt answered busy and the wait before the next: `(1, 1)`, `(2, 2)`, `(3, 4)`,
  `(4, 8)`, `(5, 16)`, `(6, 32)`, `(7, 64)`, `(8, 64)` in units of `LIMITS.host_backoff`. For
  each row, a step just before the wait fires no submission, and the step at it fires one
  with the same name and the next attempt; the eighth's answer returns as today. The caller's
  deadline is set far enough that every row's ask is made (the sum is 191 units). The table
  fails on today's fixed wait at row 2 and passes after.
- A row where a busy answer's next wait would reach the caller's deadline returns
  `Returned::Busy` with no further ask, and with an unanswered ask before it
  `Returned::HostUnknown`: the existing `host_uncertainty_survives_busy_until_withdrawal_or_caller_expiry`
  and `host_withdrawal_and_timeout_retain_relay_until_terminal_answer_wins` keep holding that,
  their steps moved to the new waits where they assumed the fixed one.
- It bites: put the fixed wait back and see row 2 fail; drop the cap and see row 8 fail;
  restore.
- Delete the bullet `**Not yet:** 5.2: a host call asked again waits ...` from
  `smith/docs/design/domain/run.status.md`, a stated departure from the implementer's text
  (`notes/common.md`), with no other change there.

## Inputs

- `temper/docs/plan/wake-followups/notes/common.md`; `notes/contracts.md`, section 3.
- `smith/docs/design/domain/run.md`, 5.2 ("Asked again, never anew", the sentences this plan
  adds); `temper/docs/design/domain/forge.md`, 5.3 (an agent's fresh read refused `Busy`);
  skein's `skein/docs/design/forge-client.md`, section 5 (admission's `Busy` is final to the
  client's owner).

## Owns and may change

- Owns the files in the header.
- Readers whose steps assume the fixed wait, edited only to step to the new waits:
  `smith/tests/run/tests/suite/memory.rs` (it names `host_backoff`), and any test of a
  package that drives a busy or unanswered host call through smith's relay and advances time
  by one `host_backoff` per ask (`rg -n 'HostReply::Busy|Reply::Busy|host_backoff'` in
  `smith/`, `jig/` and `temper/` tests). Name each in the report.
- Pre-approved: if a world's story asserts how many asks a busy host met in a span, it
  follows the new count; if one would need the fixed wait to keep its meaning, stop with a
  `Question:`.
- Not yours: `host_backoff`'s value or meaning as the first wait, smith's limits and their
  printed inventory (`smith-agent-service/src/profile/effective/list.rs`), the forge client's
  admission and figures, temper's root (`forge_route/reads.rs`).
- `smith/docs/design/**` is read by tests (`.config/gate/read-by-tests.txt`): run the tests
  that read it (`rg -l 'docs/design' smith --type rust` for an `include_str!` of a design
  file) once with the design this plan landed.

## Steps

1. The table; see row 2 fail.
2. The growing wait in `host_recover`; the table passes.
3. Bite as above; delete the status bullet.
4. `cargo fmt`; `python3 -I -B smith/scripts/old_forms.py`; `heavy cargo clippy -p
   smith-domain-run --all-targets -- -D warnings`; `heavy cargo nextest run -p
   smith-domain-run -p smith-run-world`; the focused tests of `smith-domain-run`'s direct
   dependents in every tree (`cargo tree -i smith-domain-run --depth 1 -e normal`) whose tests
   drive a host call's retry (as the `rg` above finds), each named.
5. Commit (why: a host that stayed busy was asked again every 50 ms until the caller's
   deadline); push your branch.

## Report

As the implementer's text says, and also: each reader whose steps moved.
