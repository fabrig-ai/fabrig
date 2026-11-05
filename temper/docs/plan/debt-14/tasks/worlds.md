+++
id = "worlds"
part = "worlds"
kind = "feature"
needs = []
files = ["smith/tests/session/src/translate.rs", "smith/tests/session/Cargo.toml", "smith/tests/session/tests/suite/main.rs", "smith/tests/session/tests/suite/window_room.rs", "smith/tests/session/tests/suite/startup_receiving.rs", "smith/tests/session/tests/suite/recorded/openings.rs", "smith/tests/tools/src/translate.rs", "smith/tests/tools/Cargo.toml", "smith/tests/tools/tests/suite/simulation.rs", "smith/tests/machine/Cargo.toml", "smith/tests/machine/tests/suite/stories.rs", "smith/tests/machine/tests/suite/listing.rs", "smith/tests/protocol-llm/tests/suite/main.rs", "smith/tests/protocol-llm/tests/suite/window_room.rs", "smith/tests/protocol-llm/tests/suite/startup_receiving.rs", "smith/tests/protocol-llm/tests/suite/turn_codec.rs", "smith/tests/protocol-llm/tests/suite/world_fakes.rs", "Cargo.lock", "smith/docs/design/testing.status.md", "smith/docs/development/debt/D017-domain-worlds-on-protocols.md"]
effort = "high"
minutes = 120
+++

# The session and tools worlds stand on domains and fakes

## Goal

"Domain worlds use domains and fakes only, never protocol crates" (`smith/docs/design/testing.md`, 2). Four tests of
protocol behaviour move to the protocol worlds where they belong. What the two worlds still call in a protocol crate
becomes a fake of what they need, held to the real code by a test in a protocol world. Every test's events and outcome
stay.

Acceptance:

- `cargo tree -p smith-session-world -e normal,dev --prefix none | grep -c '^smith-protocol-'` and the same for
  `smith-tools-world` read 0 (`cargo tree` builds nothing, so no `heavy`).
- The names `heavy cargo nextest list` gives for `-p smith-session-world -p smith-tools-world -p smith-machine-world -p
  smith-protocol-llm-world` are those at your base, but for the four moved tests' new places and the new equivalence
  tests.
- `heavy cargo nextest run` passes for those four, and for their readers `-p smith-agent-world -p smith-local-world -p
  smith-run-world`; and `--profile fuzzy` for `-p smith-session-world -p smith-tools-world -p smith-agent-world -p
  smith-local-world`, since the noisy calls reach the fake decode there.
- No assertion changes in any test. A moved test differs only in its `use` lines.
- `tests/session/src/translate.rs` grows by at most 100 net lines, `tests/tools/src/translate.rs` by at most 45, and the
  equivalence tests are at most 100 new lines together.
- `testing.status.md` is as below. `smith/docs/development/debt/D017-domain-worlds-on-protocols.md` is gone.
  `python3 -I -B tools/citations/citations.py --all` reads 0.

## Inputs

- `temper/docs/plan/debt-14/notes/common.md`
- `smith/docs/development/debt/D017-domain-worlds-on-protocols.md`. Its `smith-protocol-channel` is stale: the manifest
  no longer names it.
- `smith/docs/design/testing.md`, 2, 2.5 and 2.6
- `smith/crates/smith-protocol-llm/src/decode.rs`: `decode`, `admit`, and the tools' arms
- `smith/crates/smith-protocol-llm/src/render.rs`: `render_outcome`'s four path arms, `resolved_path`
- `smith/crates/smith-protocol-machine/src/listing.rs`

## Owns and may change

- **Owns:** the files in the spec's `files` list, for the changes below.
- **May change:** `smith/docs/design/testing.status.md`: only the **Departs** entry for 2, as Steps, 7 says. A departure
  from the implementer's text.
- **Not to touch:** `smith/tests/agent` and `smith/tests/run`, which keep their protocol edges: the item doesn't name
  them. No crate under `smith/crates/`. No public signature of `smith-session-world` or `smith-tools-world` changes:
  `decode` and `render` keep theirs, so `tests/agent` and `tests/local` need no edit.
- **Options:**
  - If `listed` can't match `Listing` on the equivalence table within its cap, stop and ask, stating the size and the
    case. The item allows that one to stay as a recorded departure, but it keeps a protocol edge in both worlds, so
    that is the user's.
  - If adding `smith-tools-world` to `tests/machine`'s dev-dependencies changes a machine test, say so: it turns on
    `smith-domain-tools/fixture` in those test builds, which should change compilation alone.
  - `render_with_namespace` may become `pub`, for the equivalence test. That is an addition.
  - `testing.md`'s "No copies of shipped code" is about hosted processes (the agent, the local host). A finite adapter of
    a world is what `tests/agent/src/translate.rs` and `tests/tools/src/translate.rs` already are. Don't ask about it.

## Steps

The moves are for tier, not for time: say so in their commit, and report the moved tests' times before and after. They
stay in smith's focused suite.

1. **Move to `tests/protocol-llm/tests/suite/`,** with its `main.rs`'s `mod` lines. It already depends on all they read
   (`Cargo.toml`: `smith-agent-service`, `smith-session-world`, `smith-protocol-translate`, `smith-transcript`,
   `smith-domain-session`):

   - `tests/session/tests/suite/window_room.rs`, whole, and `startup_receiving.rs`, whole: they are profile and framing
     tests (`testing.md`, 2.5).
   - `a_domain_turn_round_trips_its_account_and_backend` and `round_trip`, out of
     `tests/session/tests/suite/recorded/openings.rs`, into `turn_codec.rs`: a codec's round trip.

   Then `tests/session/Cargo.toml` loses `[dev-dependencies]` whole, if nothing else in the suite reads them.
2. **Move to `tests/machine/tests/suite/stories.rs`:** `a_command_whose_child_outlives_it_ends_with_its_group_at_the_deadline`,
   from `tests/tools/tests/suite/simulation.rs`, with a copy of `exited`, which `simulation.rs` keeps. Its machine half
   has its twin there. `tests/machine` gains a dev-dependency on `smith-tools-world`, and `tests/tools/Cargo.toml` loses
   `smith-machine-world`.
3. **`listed`,** in `tests/tools/src/translate.rs`, replacing `smith_protocol_machine::Listing`:

   ```rust
   /// What the machine makes of a listing command's `output`: the name-ordered prefix that fits `max` entries and
   /// `max_bytes`, with its ancestors, to `depth`. A failed command is `Failed`, whatever it printed.
   pub fn listed(output: &[u8], exit: fake::Exit, depth: Depth, max: u32, max_bytes: u64) -> Done;
   ```

   It checks the exit first: a session world has no listing fixture, so its `list` is an unscripted command, exit 127,
   and answers `Failed { Other }`. Then it splits, adds ancestors, filters by depth, sorts, and takes the prefix at
   `size_of::<Entry>() + name.len()` a name, and sets `more`. The fixtures' lists are sorted and hold no `.git`, so
   `Listing`'s incremental retention gives the same prefix: a prefix's cost only grows. `skein-io` leaves
   `tests/tools/Cargo.toml` if this was its last use.
4. **`call`,** in `tests/session/src/translate.rs`, with `pub fn decode`'s signature as it is. It reads the arguments
   with `skein_llm::Json::from_bytes` at the limits `admit` uses, from `skein_llm_world::limits()` (`client.input` and
   `depth` are all that count: `fixture::small` and `Stated` don't reach `decode`), and builds calls with
   `smith_tools_world::calls`. It covers the six tools of `TOOLS`, and the failures reached: `UnknownTool`,
   `NotAnObject` (including a cut or a non-object), `Missing { .. }` for each required field, and `TooLarge`. Optional
   fields a decoder reads and the worlds never send stay out; `tests/agent/src/translate.rs` overlays `yield_seconds` and
   `wait_seconds` itself. `skein-json` joins the manifest for `Kind`. A byte-find won't do: JSON escapes reach this.
5. **`path_feedback`,** in the same file, for `render_with_namespace`'s four arms:

   ```rust
   /// The words `render_outcome` gives a path the tool refused, and that they're a failure.
   fn path_feedback(outcome: &Outcome, namespace: &Namespace) -> Option<(Box<[u8]>, bool)>;
   ```

   Same words, same `resolved_path` cases. `a_resolved_missing_path_reaches_the_llm_in_the_fixed_conversation_namespace`
   asserts these bytes and must not change. `smith-protocol-llm` leaves the manifest.
6. **Hold the fakes to the code.** Both are required: `tests/session`'s bytes reach token counts and traces.

   - `tests/protocol-llm/tests/suite/world_fakes.rs`: `smith_session_world`'s `decode` against
     `smith_protocol_llm::decode` over the six tools crossed with valid, missing-field, malformed and three-byte-cut
     arguments, and an unknown name; its `render` against `render_outcome` over the four arms and the namespaces. Mind
     that the two `Decoded` enums differ.
   - `tests/machine/tests/suite/listing.rs`, which exists: a new test in it, `listed` against `Listing` over the tools
     fixtures' lists and caps, depths and a failed exit.

   **Bite:** change a word in `path_feedback`, a problem in `call`, and `more` in `listed`. Each must go red here,
   and be restored.
7. **`testing.status.md`:** the **Departs** entry for 2 becomes this, which is what stays true:

   ```md
   - 2: domain worlds use domains and fakes only, never protocol crates,
     but `tests/agent/src/backend.rs` uses `smith-protocol-llm`, and
     `tests/run/tests/suite/checks.rs` runs `smith-machine-world`. No
     reason is recorded.
   ```

   Delete D017's file. `STATUS.md`'s "Two domain worlds call protocol crates" stays true of these two: leave it.
8. `cargo fmt`, the ratchet, clippy on the four worlds, the tests above, `citations.py --all`. Commit and push. Commits:
   one for the moves, one for tools, one for session, one for the tests and the item.

## Report

The two `cargo tree` counts. The name-list comparison. What each bite printed. The moved tests' times before and after.
Any fuzzy seed whose trace moved is a stop, not a report. Net lines per file.
