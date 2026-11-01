+++
id = "rm-box-remove"
part = "forge-removal"
kind = "feature"
needs = ["fr-group-file", "kept-acts", "box-pipe"]
files = [
  "temper/crates/temper-service/src/route/box_link.rs",
  "temper/crates/temper-service/src/forge/table.rs",
  "temper/crates/temper-service/src/tests/box_remove.rs",
]
effort = "high"
minutes = 40
+++

## Goal

Serve's box link sends a kept `BoxRemove` as the pipe's `remove FORGE`
and hears its answer (contracts, section 3): `removed FORGE` erases the
kept act; `failed FORGE WHY` keeps it, to go again at the next start
(`notes/progress-box.md`'s contract problem, accepted by the lead). The
box side is `box-pipe`'s.

Acceptance:

- `forge::table::box_remove(engine, forge)` (the stub `fr-surface` made,
  reached by the root's `Request::BoxRemove` from `kept-acts`) asks the
  link `remove FORGE`. Not under the box (no link), it answers at once
  as removed: there is no group to remove.
- The link keeps its one-ask-at-a-time rule (setup.md 7.1): a `remove`
  while another ask is in flight is held and sent once that ask answers,
  in order, as a held start is; more than one held is a queue bounded by
  `forge.forges`.
- `removed FORGE` answers the ask: `forge::Event::BoxRemoved { forge,
  removed: true }` to the connector (`root::Event::Forge`); `failed
  FORGE WHY` answers `removed: false` and names the why in a
  `Out::Named` log line; the pipe's end with a `remove` in flight answers
  `removed: false`.
- A `start FORGE` already in flight for the forge is not cancelled by
  a later `remove`: the remove waits for its answer, then goes.
- `removed` and the new `failed` words parse with the link's existing
  decoder (`decoded`): canonical decimal only, as `started`.

## Inputs

- `temper/docs/plan/playground-reset-r4/notes/common.md` and
  `temper/docs/plan/playground-reset-r4/notes/contracts.md` (section 3).
- `temper/docs/plan/playground-reset-r4/notes/progress-box.md` (the
  `failed` answers to `remove`), and the merged `box-pipe` report (the
  exact words).
- `temper/docs/design/protocol/setup.md`, 7.1.
- The merged `fr-group-file` (the link's ask states) and `kept-acts`
  reports.

## Owns and may change

New test files are declared from the module they test, at its end
(`#[cfg(test)] #[path = "tests/NAME.rs"] mod tests;`, the path relative to
that module, as `temper-protocol-store/src/records.rs` declares `holds`),
never in a crate's `src/tests.rs`: tasks running at once would all edit it.
Where this spec says "its `mod` line in `tests.rs`", read that.

Owned: `route/box_link.rs`, `forge/table.rs`'s `box_remove` alone, the
new `tests/box_remove.rs` (its `mod` line in `tests.rs`).

Neighbours: `route.rs`'s `box_link::drain` handling of the new `Out`
variant, if it needs one: that arm only.

## Steps

1. `box_link.rs`: `Ask::Remove(u16)`, `pub(crate) fn remove(&mut self,
   forge: u16, out: &mut Queue<Out>)`, `fn remove_line(forge) -> Box<[u8]>`
   (`remove N\n`), `Reply::Removed`, `Out::Removed { forge, removed:
   bool }` drained to `forge::Event::BoxRemoved`.
2. `table::box_remove`'s body.
3. **Tests**, `tests/box_remove.rs`: `a_kept_box_remove_writes_remove_and_removed_erases`
   (with the service fixtures and the box link under a piped fixture),
   `a_failed_remove_keeps_the_act`,
   `a_remove_waits_for_the_ask_in_flight`,
   `the_pipes_end_answers_not_removed`,
   `without_the_box_a_remove_is_removed_at_once`;
   the link's own unit tests in `box_link.rs`' test module, if it has
   one, for the line forms.
4. `cargo fmt`; `heavy cargo clippy -p temper-service --all-targets --
   -D warnings`; `temper-service`'s focused tests; `heavy cargo check
   --tests -p temper -p temper-one-process-world`. Commit and push.

## Report

fab's sections, and `## Interfaces`: `Link::remove`, the new `Out` and
`Reply` variants, and the `failed` words handled.
