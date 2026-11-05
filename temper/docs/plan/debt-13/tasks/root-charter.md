+++
id = "root-charter"
part = "root-charter"
kind = "feature"
needs = ["root-leaves"]
files = ["jig/crates/jig-protocol-charter/**", "jig/examples/ops/domain/src/assemble.rs", "jig/examples/ops/domain/src/link.rs", "jig/examples/ops/domain/src/route.rs", "jig/examples/ops/domain/Cargo.toml", "jig/testing/jig-test-service/src/tools.rs", "jig/tests/boundary/src/tests/graph.rs", "jig/docs/design/domain/hosts.status.md", "Cargo.lock"]
effort = "high"
minutes = 75
+++

# The charter crate puts a run's hand-over together

## Goal

`ops`'s `assemble.rs` and `jig-test-service`'s `tools.rs` hold the same 99 lines: sections and their titles, start
messages, answered calls. So `ops`'s root renders words, which `jig/docs/design/domain/root.status.md` records as a
departure. `jig-protocol-charter` already translates the charter (`jig/docs/design/domain/hosts.md`, 8), so it takes the
whole hand-over, and a root gives only what's its own.

Acceptance:

- `rg -n 'fn (title|delivery|brief|replay)\b' jig/examples/ops/domain/src/assemble.rs jig/testing/jig-test-service/src/tools.rs` finds nothing.
- `rg -n 'jig_protocol_tools::words' jig/examples/ops/domain/src jig/testing/jig-test-service/src` finds nothing. The
  test root's `link.rs` keeps its one call: it doesn't link the charter crate, and adding smith to it is no part of
  this task. Exempted.
- `ops`'s `assemble.rs` is at most 70 lines (155 at base). `tools.rs` loses at least 90.
- `jig-protocol-charter/src/hosted.rs` is at most 115 lines, tests apart.
- Both roots' assignments are the same bytes as before: `heavy cargo nextest run -p jig-protocol-charter -p jig-ops-world
  -p jig-test-service -p jig-boundary` passes.
- `jig/docs/design/domain/hosts.status.md`'s **Not yet** clause for 8 and `hosted` is gone.

## Inputs

- `temper/docs/plan/debt-13/notes/common.md`
- `temper/docs/plan/debt-13/notes/contracts.md`, 4
- `jig/docs/development/debt/D004-ops-root-jig-wiring.md`
- `jig/docs/design/domain/hosts.md`, 8
- `jig/docs/design/domain/root.md`, 7 and 11

## Owns and may change

- **Owns:** `jig-protocol-charter` (new `src/hosted.rs`, a test, the `pub use`).
- **May change:**
  - `jig-protocol-charter/Cargo.toml`: `jig-domain-host`, `jig-domain-hub-link`, `jig-domain-brief` and
    `jig-protocol-tools`.
  - `jig/tests/boundary/src/tests/graph.rs`: `jig-domain-brief` joins `hosts_and_translation_use_only_their_core_vocabularies_even_when_empty`'s
    charter list, under its existing `hosts.md, 8` comment. That's the one rule changed, on the design's word: nothing
    else in that file.
  - `ops`'s `assemble.rs`, the two callers and the `Message` arm in `link.rs`, the one caller in `route.rs`.
    `jig-test-service/src/tools.rs`'s `charter`.
  - `ops`'s `Cargo.toml`, dropping `jig-protocol-tools` if `translate.rs` no longer needs it. It does at base, for
    `batch`, so expect it to stay.
  - `jig/docs/design/domain/hosts.status.md`, the clause alone. `Cargo.lock`, by the build alone.
- **Not to touch:** `jig-protocol-tools`'s manifest: `jig/tests/boundary/src/tools.rs` pins it and refuses `host`, so
  what needs both `words` and `host` is the charter crate's. `declarations`, `family` and the service's other tools:
  those are its own. `assemble.rs`'s
  `read_answer` and `connector_read`, `ops`'s own. `temper-service/src/assignment.rs`: a branch stays in its tree, and
  the report says whether `hosted` would fit it, under `## Left`. `route.rs` beyond its one call: `root-now` holds it.
- **Options:**
  - A placed connector section at base panics in `ops` and answers `None` in the test service. `hosted` answers `None`:
    an application with no connector sections never places one, and a refused claim is the louder of the safe answers.
  - `ops` bounds answered calls and messages by its limits and refuses past them; the test service passes each list's
    own length: `Given`'s two counts say which.
  - If a cycle appears (a crate that `jig-protocol-charter` now depends on depends on it), stop and ask.

## Steps

1. `src/hosted.rs`: `Given`, `hosted` and `message`, by `notes/contracts.md`, 4. Bodies are `ops`'s `assignment`, `brief`,
   `title` and `delivery`, with `domain.tools`, `domain.charter_endpoints`, `domain.deployment`, `env.limits.charter`
   and the two counts read from `Given`. `message` is `ops`'s `Message` arm in `link.rs`, and `hosted`'s start messages
   share its fields.
2. A test in `src/tests.rs`: an assignment of every `GatherPlaced` but `Connector`, two messages from different parties,
   a settled call of each kind. Assert the section titles and the `[missing]`s in order, each sender and words, the call
   names and the delivery outcomes. A second case: `Connector` answers `None`. A third: `Given::answered` below the
   count answers `None`. Break a title by hand and see it red; restore.
3. `ops`: `assemble::assignment` becomes a short call building `Given`, kept as a function so `link.rs` and `route.rs`
   read as they did. `brief`, `title`, `delivery` go. `link.rs`'s `Message` arm calls `charter::message`.
4. `tools.rs`: `charter` keeps its signature for its callers, and is `declarations` and a call of `hosted`. `replay`,
   `delivery`, `title` go.
5. `cargo fmt`, `heavy cargo clippy -p jig-protocol-charter -p jig-ops-domain -p jig-test-service --all-targets -- -D
   warnings`, the tests above, `heavy cargo check -p temper-service --tests`, because temper links the charter crate,
   commit, push.

## Report

What changed, the commit, what ran with counts, the rule broken and what failed, the net lines, and `## Left` for
temper's assembly.
