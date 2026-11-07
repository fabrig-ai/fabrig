+++
id = "pressed-step"
part = "pressed"
kind = "feature"
needs = []
files = [
  "skein/testing/skein-browser/src/boundary.rs",
  "skein/testing/skein-browser/src/browser.rs",
  "skein/testing/skein-fake-person-browser/src/face.rs",
  "skein/testing/skein-fake-person-browser/src/tests.rs",
  "skein/testing/skein-fake-person-browser/src/harness/script.rs",
  "skein/testing/skein-fake-person-browser/src/harness/script/pressed.rs",
  "skein/testing/skein-fake-person-browser/src/harness/script/tests.rs",
  "skein/tests/browser/src/tests.rs",
  "skein/tests/browser/src/world.rs",
  "skein/docs/design/browser.status.md",
  "skein/docs/design/ui-web.status.md",
]
effort = "high"
minutes = 50
+++

# A browser story awaits a control pressed, by its name

## Goal

temper's history story reads the Chats filter's pressed button by its place among its siblings, once, and fails when a Drafts button arrives before Closed (red-63). skein's kit can't say a node is pressed, and its harness has no step that waits for it. Give the kit Chromium's `pressed` state, and the harness a step that awaits a control pressed, found by its target, within the step's time. temper's story moves to it in `history-story`.

This task departs from the implementer's text in one way: the two **Not yet** entries this task closes are yours to remove (acceptance 5), since the design already says what it builds.

Acceptance:

1. `skein_browser::boundary::States` has `pub pressed: bool`, set from a found node's Chromium accessibility property `pressed` where it reads `true` (the tristate's text `"true"`, or a JSON `true`), and false for `"false"`, `"mixed"` and no property. A table test pins those five rows.
2. `skein_fake_person_browser::Step::Pressed { find: Find, within: Duration }` awaits the node the face finds for `find` until its `pressed` state holds, looking again as the face's finds do (each poll interval), within `within` from the step's start:
   - pressed at a look: the step passes and the script goes on;
   - found but not pressed when `within` passes: the step fails with `node {find:?} is not pressed within its bound`;
   - not found when `within` passes: the step fails as missing (`Why::Missing { find }`), as the focus step's absent target does.
3. A table test in `skein-fake-person-browser`'s script tests pins four rows: pressed at the first look; unpressed at the first look and pressed at a later one; unpressed until the bound; absent until the bound. Each new row bites (break the check once by hand, see the row fail for its reason, restore).
4. The deadline holds with no event: `check_deadline` ends the step at its bound, as it ends a tabs step.
5. The entries *5: `States` has no `pressed`...* in `skein/docs/design/browser.status.md` and *14.2: no step awaits a control pressed...* in `skein/docs/design/ui-web.status.md` are gone.
6. Focused tests pass in `skein-browser`, `skein-fake-person-browser` and `skein-browser-world` (`--lib` and the `suite` binary, never `browser_*`). `heavy cargo check --tests` passes for `skein-ui-web-tests`, `jig-client-fake-person-browser`, `jig-client-browser-world`, `temper-client-browser-world` and `temper-client-journeys`.

## Inputs

- `temper/docs/plan/ui-history-story/notes/common.md`.
- `skein/docs/design/browser.md`, 5: the `Seen` line and *A node's states are Chromium's*.
- `skein/docs/design/ui-web.md`, 14.2: *awaits a control pressed*, beside *keeps a value for the test*.

## Owns and may change

**Owned:** the `files` above. `script/pressed.rs` is new, if a module of its own reads better than an arm in `script.rs` (as `script/tabs.rs` is for the tabs step); otherwise leave it out.

**Readers:** every `States { .. }` literal gains `pressed: false`. The survey found three: `skein-browser/src/browser.rs`'s `ax_states`, `skein-fake-person-browser/src/tests.rs`'s `seen`, and `skein/tests/browser/src/tests.rs`'s `face_found`; none in jig or temper. `Step` is matched in `script.rs` alone; jig's and temper's stories build steps but match none. `jig-client-fake-person-browser`'s `States` is jig's own type and isn't touched.

**Options, each with its default:**

- Where the parse's table goes: default, rows in `skein-browser`'s `browser.rs` inline `tests` module over `ax_states`, each property given as the JSON Chromium sends (`{"name":"pressed","value":{"type":"tristate","value":"true"}}`). If `skein-browser-world`'s fake tree (`world.rs`, the heading's `properties`) gives the rows more cheaply, put them there, and say so.
- `pressed` reads text or a bool as `ax_property_bool` does; if that helper already gives exactly the five rows, use it.
- The face's state read: a `pub(crate) fn pressed(&self, node: NodeId) -> bool` beside `focused`, reading the remembered `Seen`.
- The step's waiting: what the face answers for a repeated find already waits for the poll (`repeated_finds_wait_for_the_poll_whether_the_target_is_present_or_absent`). Default: call `Face::find` each turn and judge its answer against the step's `until`, returning a `Next::Wait` that wakes the loop by the bound, as the person's `See` does. If the loop needs a new wake of its own, add it inside this crate and say so.
- A doc line on the new variant, as `Address` has, in the design's words.

## Steps

1. `skein-browser`, `boundary.rs`:

   ```rust
   pub struct States {
       pub focused: bool,
       pub disabled: bool,
       pub checked: bool,
       pub expanded: bool,
       pub selected: bool,
       /// A toggle's `pressed` reads `true` (browser.md, section 5).
       pub pressed: bool,
   }
   ```

   `browser.rs`'s `ax_states`: `Some(b"pressed") => states.pressed = value`. The table test.
2. `skein-fake-person-browser`, `script.rs`:

   ```rust
   pub enum Step {
       // ...
       /// Await the found node pressed, within this long (ui-web.md, 14.2).
       Pressed { find: skein_fake_person::Find, within: Duration },
       // ...
   }
   ```

   Its arm in `next` holds the find and its bound (`now.saturating_add(within)`); a turn's check beside `check_focus`, and its end in `check_deadline`. `face.rs`: `pressed(node)`.
3. The script tests' table, in `script/tests.rs`, built as `the_focus_step_checks_the_found_nodes_focus` is, with `seen` and `found` and a `target.states.pressed`; a row past the bound runs `check_deadline` at `Time::ZERO + within`.
4. The status entries out (acceptance 5).
5. Narrow tests: `heavy cargo nextest run -p skein-browser --lib`; `heavy cargo nextest run -p skein-fake-person-browser --lib`; `heavy cargo nextest run -p skein-browser-world --lib --test suite`. Check each selected tests. Then `cargo fmt`, `python3 -I -B skein/scripts/old_forms.py`, `heavy cargo clippy -p skein-browser -p skein-fake-person-browser -p skein-browser-world --all-targets -- -D warnings`, the `heavy cargo check --tests` of acceptance 6, and the citations check (notes/common.md). Commit and push.

## Report

As the implementer's text says. Also: where the parse's rows went, how the step wakes for its bound, and each row's bite.
