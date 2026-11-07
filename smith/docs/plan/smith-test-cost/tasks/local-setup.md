+++
id = "local-setup"
part = "local-setup"
kind = "feature"
needs = ["test-shapes"]
files = [
  "smith/tests/local-process/src/process.rs",
  "smith/tests/local-process/src/process/configuration.rs",
]
effort = "medium"
minutes = 35
+++

# The local process world parses its launch's document once

## Goal

`make_local` parses the same generated document twice on every traced or interactive launch: `configuration_for_launch` calls `config::from_value`, which derives the whole profile, and `product_header` then calls it again, overwrites the result's `service` and reads only its `profile`, `declarations` and `memory`. A fixture many tests share should be cheap to build (`skein/docs/foundation/testing-strategy.md`, 8). Parse once. This is the fixture's waste alone: no product crate changes, and no story loses anything.

Acceptance:

1. One `make_local` calls `smith_agent_shell::config::from_value` once and `agent_document` once, in both placements, traced or not.
2. `product_header` still calls `Local::trace_header` with the same profile, declarations and final service, so the header it adopts is the one it was.
3. No file outside `files` changes. `make_agent` still parses the written file through `config::read`: that's the product's path, and a spawned story keeps it.
4. The focused tests of `smith-local-process-world` pass.

## Inputs

- `smith/docs/plan/smith-test-cost/notes/common.md`.
- `smith/docs/plan/smith-test-cost/inputs/request.md`, 3.
- `skein/docs/foundation/testing-strategy.md`, 8.

## Owns and may change

- **Owns:** this spec's `files`.
- **Readers, read at the cut:** `configuration_for_launch` is `pub(super)` with one caller, `make_local`. `product_header` is private with one caller. `agent_document` is public and read by `tests/hosts` and `tests/end-to-end`: its signature and meaning stay.
- **Option:** if the parsed document's `program` or `program_endpoints` isn't `None` or empty for a launch, carry them too. `make_agent` builds the same struct literal with `None` and an empty set, and that's the shape to follow if they are.
- **Not this task, read at the cut:**
  - `profile::effective::header::name`, 9% of a median story. It already writes through `Writer::put`, a bulk copy, and its endpoint scan runs over one or two entries. It runs three times because `trace_header` needs the header's own names to settle the sink's limits. Reworking that fixed point changes a product's startup for a startup cost, and it isn't real waste. No product change.
  - A parse shared across a story's worlds. About 18 of the 64 stories loop over both placements, and each would save one parse of two. But `agent::Config` isn't `Clone` (`lower_configuration_with` rebuilds it by hand) and the document varies by launch, so a cache keyed on it is more fixture than that saving.

## Steps

1. **`configuration.rs`:** `configuration_for_launch` takes the launch's document and also returns what the header reads:

   ```rust
   /// What the product's header reads of the launch's document, beside its final service.
   pub(super) struct Parsed {
       pub(super) memory: /* Configuration's field type */,
       pub(super) profile: /* Configuration's field type */,
       pub(super) declarations: /* Configuration's field type */,
   }

   pub(super) fn configuration_for_launch(
       root: kernel::Fd,
       launch: &Launch,
       document: serde_json::Value,
   ) -> (service::Config, agent::Config, Parsed)
   ```

2. **`process.rs`, `make_local`:** build `agent_document(&launch)` once, write it for a spawned launch (keep its bytes and its file), and hand it on. `product_header` takes `Parsed` and builds its `smith_agent_shell::config::Configuration` as `make_agent` does, with `service: lower` and `trace: None`, then calls `Local::trace_header` as now.

   `Parsed` is only needed on the traced or interactive branch. Let an unused value simply drop.

3. No new test: this moves nothing a story sees, and a story that needs the header still reads it. Then `cargo fmt`, `python3 -I -B smith/scripts/old_forms.py`, `heavy cargo clippy -p smith-local-process-world --all-targets -- -D warnings` and `heavy cargo nextest run -p smith-local-process-world`. No public item changes, so no dependent needs a check. Commit and push.

## Report

Per the implementer's text. Say that `from_value` runs once per `make_local`, and that no product crate changed. Any timing you took is a reading only.
