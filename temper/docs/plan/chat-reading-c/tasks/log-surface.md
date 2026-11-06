+++
id = "log-surface"
part = "log-acts"
kind = "interface"
needs = []
files = [
  "skein/crates/skein-ui/src/binding.rs",
  "skein/crates/skein-ui/src/tree.rs",
  "skein/crates/skein-ui/src/builder.rs",
  "skein/crates/skein-ui/src/lib.rs",
  "skein/crates/skein-ui/src/tests/**",
  "skein/testing/skein-fake-person/src/**",
  "skein/testing/skein-fake-person-browser/src/face.rs",
]
effort = "high"
minutes = 40
+++

# A log may be bound, and says as an act that an edge of it is near

## Goal

The surface every later reader of a log builds on: `Act::Near { edge }`, `LogEdge`, `State::Continues` and a log's end request, in `skein-ui`, and every exhaustive reader of `Act` and `State` compiling. No shell reports the act yet: `page-reading` and `text-reading` do. skein names no temper.

Acceptance:

- `takes(Role::Log, _, &Act::Near { .. })` holds, and for every other role it doesn't; a log takes no other act.
- `Builder::bind` takes a log. The view decodes a `Near` on a bound log to a `BoundAct` with its binding, drops it on an unbound log, and refuses a `Press` on a log.
- `Builder::state(State::Continues)` and `Builder::end_request` panic on any role but a log, as `FoldsAway`'s assertion does on any role but a disclosure.
- An end request that moves makes a `Patch::Attribute`, as a focus count does.
- The random person (`skein-fake-person/src/random.rs`) acts on a bound log with `Near`, its edge drawn from the seed, and every other node's act from a given seed is drawn as it was while no log is bound.
- The clause `3.2, 6, 11: a log's Near, continues and its end request` leaves `skein/docs/design/ui.status.md`'s **Not yet**, keeping what `page-reading` owns: nothing there is yours but that clause.

## Inputs

- `skein/docs/design/ui.md`, 3.2 and section 6, *A log's edge near*.
- `temper/docs/plan/chat-reading-c/notes/contracts.md`, *skein-ui*, binding.
- `temper/docs/plan/chat-reading-c/notes/common.md`.

## Owns and may change

Owns the files in the header.

Readers and neighbours, for the compiler-forced arm of the new `Act` and `State` variants only, in any tree:

- `skein/crates/skein-ui/src/view.rs`, `skein/crates/skein-ui-text/src/command.rs`, `skein/crates/skein-ui-text/src/render.rs`, `skein/crates/skein-ui-web/src/mapping.rs` (`State::Continues` sets nothing), `skein/crates/skein-ui-web/src/page.rs`, `skein/examples/ui/view/src/state.rs`.
- `jig/crates/jig-client-view/src/binding.rs`, `jig/testing/jig-client-test-view/src/{bindings,root}.rs`, `jig/examples/ops/client-view/src/bindings.rs`.
- `temper/crates/temper-client-view/src/{frame,accounts}.rs`, `temper/crates/temper-client-view/src/forge/bindings.rs`, `temper/tools/gallery/src/worlds.rs`: a bound log has no meaning for them yet, so `Near` decodes to nothing.
- `temper/crates/temper-client-shell-web/src/journey/line.rs`: the line's words `near head` and `near foot`, written and read, beside `press`, with `again.rs`'s and `line.rs`'s copy of an act. A recording of a page read by scrolling holds these. `temper/crates/temper-client-shell-web/src/journey/{replay,walk}.rs`, `temper/tools/journeys/src/again.rs`: the forced arm alone. `real-journeys` holds these files: one arm each.

Options, each decided:

- `skein-fake-person-browser/src/face.rs`: `Act::Near` is an error, as a drawing's is: "a person scrolls a log; its `Near` is the page part's". `page-reading` gives that person a scroll where a story needs one.
- If `look-round-6`'s `State::ToLogEnd` has arrived by catch-up, `Continues` stands beside it, and keep both assertions.
- A `#[should_panic]` is how `skein-ui`'s tests pin a builder's assertion already: follow them.

Not this task: any shell's reporting, a log's measures, `skein-ui-web`'s behaviour, any temper page.

## Steps

1. Red first, in `skein/crates/skein-ui/src/tests`: `a_log_takes_near_and_nothing_else`, `near_decodes_on_a_bound_log_and_is_dropped_on_an_unbound_one`, `continues_belongs_to_a_log`, `an_end_request_belongs_to_a_log`, `an_end_request_that_moves_patches_the_log`.
2. The types, `takes`, `acted_on`, the builder and the `Attribute`'s default, by the contract.
3. The random person, and `the_random_person_scrolls_a_bound_log` in its tests.
4. The forced arms, by `heavy cargo check -p PKG --tests` for each reader above.
5. `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`, and `temper/scripts/old_forms.py` and `jig/scripts/old_forms.py` for the arms there; `heavy cargo clippy -p skein-ui -p skein-ui-text -p skein-ui-web -p skein-fake-person -p skein-fake-person-browser --all-targets -- -D warnings`; `heavy cargo nextest run -p skein-ui -p skein-ui-text -p skein-ui-web -p skein-fake-person`. `heavy cargo check --tests` naming `jig-client-view`, `jig-client-test-view`, `temper-client-view`, `temper-client-shell-web`, `temper-client-gallery` and `temper-client-journeys`. Commit and push.

## Report

As the implementer's text asks, with each file outside your own where an arm was forced.
