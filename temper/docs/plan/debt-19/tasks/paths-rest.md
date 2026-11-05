+++
id = "paths-rest"
part = "smith-paths"
kind = "migration"
needs = ["paths-bench"]
files = ["smith/crates/smith-agent-shell/**", "smith/crates/smith-agent-service/tests/**", "smith/crates/smith-channel/src/lib.rs", "smith/crates/smith-charter/src/lib.rs", "smith/crates/smith-domain/src/tests.rs", "smith/crates/smith-domain-run/src/prompt.rs", "smith/crates/smith-host-protocol/src/component.rs", "smith/crates/smith-host-protocol/src/component/**", "smith/crates/smith-local-domain/src/**", "smith/crates/smith-local-protocol/src/lib.rs", "smith/crates/smith-local-protocol/src/format.rs", "smith/crates/smith-local-protocol/tests/**", "smith/crates/smith-local-service/src/credentials/tests/**", "smith/crates/smith-protocol-program/src/backend/**", "smith/crates/smith-transcript/src/lib.rs", "smith/tests/**", "smith/scripts/old-forms.txt", "smith/docs/development/debt/D015-path-modules.md", "smith/docs/design/testing.md", "smith/docs/design/domain/host.status.md"]
effort = "medium"
minutes = 60
+++

# smith: every remaining `#[path]` is generated code's, with its reason

## Goal

The last step of `smith/docs/development/debt/D015-path-modules.md`: **Paid when** holds, and the item goes. 40 attributes go. Six stay,
each a plain layout with its reason beside it, because the schema compiler writes those paths, in every tree.

Acceptance:

- `rg -n '#\s*\[\s*path' smith -g '*.rs'` lists exactly the six of step 5, each under a comment saying why.
- `rg -n '#\s*\[\s*path\s*=\s*"[^"]*tests/' smith -g '*.rs'` finds nothing: no production target includes a file under `tests/`.
- `grep '^path_attribute ' smith/scripts/old-forms.txt` reads 6.
- No test changes its name or its binary: `heavy cargo nextest list` for each package you changed shows the same tests as at your base,
  but for what the spec names. No assertion changes.
- The item is deleted.

## Inputs

- `temper/docs/plan/debt-19/notes/common.md`
- `smith/docs/development/debt/D015-path-modules.md`
- `skein/docs/design/codec.md`, 11 (a patch owner's `patches/`)

## Owns and may change

- **Owns:** the files in `files`.
- **Design files, the moved file's path only:** `smith/docs/design/testing.md` (about line 396, `smith-local-protocol/tests/support/disk.rs`) and
  `smith/docs/design/domain/host.status.md` (about line 16, `smith-local-domain/src/chat/machine.rs`). This departs from the implementer's text
  because both cite a file you move. Tests read `smith/docs/design/**`; run them.
- **The new line in manifests:** `smith-agent-shell`'s and `smith-agent-process-world`'s `Cargo.toml`, as steps 3 and 4 say.
- **If a target of a move already exists,** or a moved file is read by a target this spec doesn't name, take the nearest ordinary module path and say
  so in the report.

## Steps

A module keeps its name in every step, so a test's name holds. Each move is a `git mv`.

1. **Remove the attribute; the module's name already leads there.** `smith/tests/session/tests/suite/recorded.rs`, `smith/tests/codecs/tests/support/events.rs`,
   `smith/tests/run/tests/suite/memory.rs`, `smith/tests/agent-process/tests/suite/memory.rs`, `smith/tests/host/tests/suite/memory.rs` and `simulation.rs`,
   `smith/tests/protocol-llm/tests/suite/prefix.rs`, `smith/tests/channel/tests/suite/stories.rs` and `composed.rs`, `smith/crates/smith-domain/src/tests.rs` (2),
   `smith/crates/smith-domain-run/src/prompt.rs`.
2. **Move into the parent's directory, then remove it:**

   | Declared in | Module | From | To |
   |---|---|---|---|
   | `smith/tests/session/src/world.rs` | `window_referee` | `src/window_referee.rs` | `src/world/window_referee.rs` |
   | `smith/tests/end-to-end/live_configuration.rs` | `live_tests` | `support/live_tests.rs` | `live_tests.rs`, beside it |
   | `smith/crates/smith-host-protocol/src/component.rs` | `grant_tests` | `component/tests.rs` | `component/grant_tests.rs` |
   | `smith/crates/smith-protocol-program/src/backend/files.rs` | `tests` | `backend/files_tests.rs` | `backend/files/tests.rs` |
   | `.../backend/stream.rs` | `tests` | `backend/stream_tests.rs` | `backend/stream/tests.rs` |
   | `.../backend/policy.rs` | `tests` | `backend/policy_tests.rs` | `backend/policy/tests.rs` |
   | `.../backend/compact.rs` | `compact_tests` | `backend/compact_tests.rs` | `backend/compact/compact_tests.rs` |
   | `smith/crates/smith-local-domain/src/domain.rs` | `chat_machine`, `delivery`, `reconcile`, `parent_tests`, `program_check_tests`, `child_backlog_tests` | `chat/machine.rs`, `delivery/machine.rs`, `reconcile.rs`, `*_tests.rs` | `domain/<module>.rs` |
   | `smith/crates/smith-local-service/src/credentials/tests/private.rs` | `shared_tests` | `tests/lock.rs` | `tests/private/shared_tests.rs` |
   | `smith/crates/smith-agent-service/tests/suite/fuzzy/profile.rs` | `profile_declarations` | `tests/support/declarations.rs` | `tests/suite/fuzzy/profile/profile_declarations.rs` |
   | `smith/crates/smith-local-protocol/tests/suite/main.rs` | `disk` | `tests/support/disk.rs` | `tests/suite/disk.rs` |
   | `smith/tests/fake-claude/tests/suite/main.rs` | `support` | `tests/support/` | `tests/suite/support/` |
   | `smith/tests/codecs/tests/suite/main.rs` | `support` | `tests/support/` | `tests/suite/support/` |
   | `smith/tests/agent-process/tests/suite/main.rs` | `direct_opening`, `opening` | `tests/support/` | `tests/suite/` |

3. **One owner for the product document.** `smith/crates/smith-agent-shell/tests/support/product.rs` moves to `src/fixture.rs`, behind
   `#[cfg(any(test, feature = "fixture"))] pub mod fixture;`. The manifest gains `[features] fixture = []` and, under dev-dependencies,
   `smith-agent-shell = { path = ".", features = ["fixture"] }`. `src/config/tests/mod.rs` reads `crate::fixture::product_document`, and
   `tests/suite/fuzzy/startup.rs` `smith_agent_shell::fixture::product_document`.
4. **`smith-agent-process-world`.**
   - `tests/live/probe.rs` moves to `src/probe.rs` as `pub mod probe` of the world, its items and fields `pub`; `skein-scratch` moves from the manifest's
     dev-dependencies to its dependencies. `tests/suite/program_configuration.rs` and `tests/live/program.rs` read
     `smith_agent_process_world::probe::Probe`.
   - `tests/live.rs` moves to `tests/live/main.rs`: its binary stays `live`, by Cargo's own discovery, and `mod program;` leads to `tests/live/program.rs`.
   - `src/program.rs` moves to `src/program_world.rs`, and `src/program/` to `src/program_world/`. `pub mod program_world` keeps its name and its 81 uses.
5. **The six stay, each with a one-line comment above it** saying the schema compiler (or for `patches/`, its patch owner's layout) writes that path: `smith-channel`,
   `smith-transcript` and `smith-charter`'s `src/lib.rs`; `smith-local-protocol`'s `src/lib.rs` and `src/format.rs`; `smith/tests/transcript-patches/src/lib.rs`.
   jig and skein declare theirs the same way (`jig/crates/jig-client-protocol-wire/src/lib.rs`).
6. `python3 -I -B scripts/old_forms.py --write` from `smith/`. Delete the item, and any citation of it that breaks. `tools/debt/debt.py`'s index, if committed, follows.
7. `cargo fmt`; `heavy cargo clippy` and `heavy cargo nextest run` naming every package you changed: `smith-agent-shell`, `smith-agent-service`, `smith-channel`,
   `smith-charter`, `smith-domain`, `smith-domain-run`, `smith-host-protocol`, `smith-local-domain`, `smith-local-protocol`, `smith-local-service`, `smith-protocol-program`,
   `smith-transcript`, `smith-session-world`, `smith-codecs-world`, `smith-run-world`, `smith-agent-process-world`, `smith-host-world`, `smith-protocol-llm-world`,
   `smith-channel-world`, `smith-fake-claude`, `smith-transcript-patches`. `heavy cargo check -p smith --tests --features end-to-end`. The citations check. Commit and push.

No test pins a new rule, so nothing here needs breaking by hand.

## Report

The common note's items, the six and their comments, and any move that went elsewhere than the table.
