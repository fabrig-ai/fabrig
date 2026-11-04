+++
id = "engine-fetched"
part = "row"
kind = "interface"
needs = []
files = ["temper/crates/temper-engine-domain-forge/src/boundary.rs", "temper/crates/temper-engine-domain-forge/src/domain.rs", "temper/crates/temper-engine-domain-forge/src/lib.rs", "temper/crates/temper-engine-domain-forge/src/tests.rs", "temper/crates/temper-engine-domain-forge/src/tests/connect.rs"]
effort = "high"
minutes = 35
+++

# The forge domain keeps the fetch's latest line

## Goal

The forge domain takes `Event::Fetched`, keeps the latest beside the forge's connection in memory, and puts it on the forge's row while the connection is setting up at `fetch`. The link feeds it in `link-fetched`.

Acceptance:

- `SetUpBinary`, `SetUpFetched`, `Event::Fetched` and `Row.fetched`, as `temper/docs/plan/fetch-progress-1/notes/contracts.md`, section 4, has them;
- a `Fetched` for a forge setting up at `fetch` sets the row's `fetched`, replaces the one before it and asks `Request::MarkedForges`;
- one for a forge at any other connection, or an ended forge, changes nothing and asks nothing;
- any `Event::Connection` for that forge drops it, as `Event::BoxClosed` does: gone at the next step, at `SetUpFailed`, at `Connecting`;
- a forge beside it keeps its own row;
- nothing of it reaches a store family or a record.

## Inputs

- `temper/docs/design/protocol/forge.md`, 2.8, the box's lines.
- `temper/docs/plan/fetch-progress-1/notes/contracts.md`, section 4, and `notes/common.md`.

## Owns and may change

Owns the files in the header. `Connection::SettingUp { step }` does not change; that is the contract's point.

May also change, for what the compiler requires: `crate::Row`'s literals in `temper-engine-domain-forge`, `temper-engine-domain` and `temper-service`, `fetched: None` or the field's copy; nothing else there. `temper-service/src/route.rs` lists the events it routes and ends in `unreachable!`: leave it, `link-fetched` adds the arm.

Pre-approved: the domain asks one set up at a time, so one slot for one forge is enough; a field of `ConnectionState` is as good. Your call, named in the report. `decision.rs` measures `Row` with `size_of`, so it moves by itself; if a test there pins the size, update its number and say so.

## Steps

1. `boundary.rs`: the two types beside `SetUpStep`, the event, the row's field, exported as `SetUpStep` is.
2. `domain.rs`: the event's arm, and the drop where `Event::Connection` writes `d.connections` and in `Event::BoxClosed`. `forges()` fills `Row.fetched`.
3. Tests beside the connection tests in `src/tests.rs`: `a_fetched_line_is_on_the_row_while_the_step_is_fetch`, `a_fetched_line_at_another_step_changes_nothing`, `the_next_step_a_failure_and_connecting_each_drop_it`, `a_closed_pipe_drops_it`.
4. `cargo fmt`; the old-forms check; `heavy cargo clippy -p temper-engine-domain-forge --all-targets -- -D warnings`; `heavy cargo nextest run -p temper-engine-domain-forge`; `heavy cargo check --tests -p temper-engine-domain -p temper-service`. Commit and push.

## Report

As the implementer's text has it, with where it keeps the line, and each rule broken for a new test and the failure seen.
