+++
id = "core-verbs"
part = "world-verbs"
kind = "feature"
needs = ["core-kit"]
files = ["jig/testing/jig-core-kit/src/fixtures.rs", "jig/tests/core/tests/**", "jig/tests/conformance/src/scenarios.rs", "jig/tests/conformance/src/deletion.rs"]
effort = "high"
minutes = 60
+++

# jig's core world: one `ask`, and its numbers named

## Goal

Jig D035's core part. `ask` has seven copies, and the core world's fixtures are named per file. The item's first target.

Acceptance:

- `jig_core_kit::fixtures` has one `ask(world, request, ask)` beside `ask_as`, and the copies in `core/tests/suite/{refusals,credentials,funding}.rs`, `core/src/stories/public.rs` (now in the kit) and `conformance/src/{scenarios,deletion}.rs` go. **The token bases differ** (6000, 8000, `HOLDER_REQUEST_BASE`): a file whose assert reads a reply token by its number keeps it through a one-line wrapper naming the base, and your report says which. A file nothing reads a token in takes the kit's. `rg -n 'fn ask\(' jig/tests/core jig/testing/jig-core-kit jig/tests/conformance -t rust` then reads the kit's and those wrappers.
- **`tests/suite/stories.rs` and `tests/stories/note_room.rs` aren't yours:** other drafts hold them, and `verbs-close` takes them last.
- `PROJECT`, `PARTY` and `TASK` are named once in the kit's `fixtures`, for each value the core world's files hold: 12 of its 13 `const PROJECT: u32 = 1` go. A file with another value keeps its own, named for what it is.
- **`ask`'s literal key stays.** It's the request's identity, which a retry repeats, and `key: [request; 16]` and its token follow from it, so an auto-allocated one would change them. The Measure doesn't read it.
- The Measure's core files (`claim`, `lift`, `move_in`, `insert` among them) read 0 but where the number is the point; each left is in the report with why. Prefer a verb or a table: a `const` per file adds a line, and the request wants fewer.
- `heavy cargo nextest run -p jig-core-kit -p jig-core-world -p jig-conformance-world` passes, with no assertion changed. Tests' net lines fall.

## Inputs

- `temper/docs/plan/debt-24/notes/common.md`
- `jig/docs/development/debt/D035-bare-numbers-in-worlds.md`
- `jig/tests/core/tests/suite/funding.rs`, its `ask` and its rows

## Owns and may change

- **Owns:** the files in the header.
- **`jig-ops-world`'s `src/conformance/scenarios.rs`** has its own `ask`, over its own world: a verb of another world, so it stays until `const-sweep` reads it.
- **`stories/public.rs`'s and `credentials.rs`'s `ask` return the reply,** and the plain ones don't. They stay as thin wrappers over the kit's, with their own returns, if that's shorter than teaching one `ask` to do both. Report which.
- **`SEED`s stay,** each naming its run. So do seed-first helpers (`fixture`, `workers`, `procedure_world`).

## Steps

One `ask` first, then a file at a time. `cargo fmt`; `(cd jig && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p jig-core-kit -p jig-core-world -p jig-conformance-world --all-targets -- -D warnings`; the tests above. Commit and push.

## Report

The common note's items; the Measure's core count before and after; `const PROJECT`'s; the wrappers. No new rule: nothing to break by hand.
