+++
id = "settle"
part = "stories"
kind = "feature"
needs = []
files = [
  "skein/tests/browser/tests/suite/folds_away.rs",
]
effort = "high"
minutes = 90
+++

# The folds-away stories wait for the page before they act

## Goal

`skein-browser-world::browser_real`, `folds_away::a_press_outside_closes_a_menu_and_goes_on`, failed on its run and its rerun at `2a158edd47` (red-65); the family `folds_away::*` has flaked in about 4 of every 10 after-landing runs since 2026-11-06, in all three stories. Run the three stories alone until they fail, read what each failure says, find the cause, and fix it in all three where it is shared, keeping every check.

**What the cut found by reading** (confirm or refute it by the runs): a `Go` ends at the page's load event (`skein-browser`'s `browser.rs`, `Page.loadEventFired`). Since `e8600f44be` (2026-11-06 14:05Z, the day the flakes began) the example loads behind the loader (`skein/docs/design/ui-web.md`, 4.3, *The loader*): `loader.js` fetches and compiles the module after the load and starts it later, so the Wasm fixture's first apply can come after the go has ended. A person's `Press` whose find comes back absent fails at once with `Missing` (`skein-fake-person`'s `lib.rs`; a press waits for nothing), and only a `Gone` refusal is retried (14.2). Two stories press `Panel` straight after their `Go`. red-57 (`1d8ba3e0d2`) fixed the same race in the third story alone, with a `See` of `Editor` after its `Go`. The sibling `log_end` story sees before it presses.

Acceptance:

1. **Before any edit,** each of the three stories runs alone five times (Step 2), fifteen runs. If none of the fifteen fails, each runs five times more. The report gives each run's outcome, and for each failure its step number, its message, and what its snapshot shows (step 2's note says where).
2. The report names the cause the runs show, as one of the options below, and why.
3. The fix is made in all three stories where the cause is shared. Under option A or E, every story starts with the same arrival (Step 3): no story presses, keys or reads before it has seen the fixture's first apply.
4. Every check each story makes stays: each `check` selector, each `Focused`, each `see`'s words, each `Keep`, in the same order. No deadline widens, and no new browser test is added (notes/common.md).
5. **After the edit,** each story runs alone five times and all fifteen pass. A failure after the edit is read and handled by the options below, and then that story runs five more times.
6. Under option D alone: a focused test of the fault comes first, as a row of an existing test, and fails without the fix.
7. `heavy cargo check -p skein-browser-world --tests`, its clippy, and its focused tests pass (Step 4).

## Inputs

- `temper/docs/plan/ui-folds-away/notes/common.md`.
- `temper/docs/plan/ui-folds-away/inputs/request.md`: the red, its family and its one task.
- `skein/docs/design/ui-web.md`, 14.2: a press's refusals (only `Gone` is looked for again; "Other refusals fail at once"), and "What a story must see on a person's page ... the story awaits it: never for a fixed time"; "the page's load, which the kit sees, comes before its first read is answered".
- `skein/docs/design/ui-web.md`, 4.3, *The loader*, its **Handing over**: the module starts after the page has loaded.
- `skein/docs/design/ui-web.md`, 7, **Menus and disclosures are the page part's to fold**: what the stories check (a panel that folds away, a menu closed by a press outside it, where the focus goes).
- `skein/docs/design/ui-web.md`, 14.3: the example and its test build's addresses.
- The fixture: `skein/examples/ui/shell-web/src/folds_fixture.rs` (one apply draws the whole tree; the status reads `Pressed <token>`: `Form escape` 1, `Inside …` 2, `Menu item` 3, `Outside` 4, `Close editor` 5).

## Owns and may change

**Owned:** `skein/tests/browser/tests/suite/folds_away.rs`. Nothing outside it reads its helpers; `browser_real.rs` only declares the module.

**Options, each with its default.** Read the runs of Step 2, then:

- **A (expected): a step acts or reads before the fixture's first apply.** A failure at a story's first step after its `Go`, `Missing` for `Panel` (or another of the fixture's nodes), with a snapshot showing the loader's placeholder or an empty mount. Default: Step 3, the shared arrival. No product change.
- **B: a read once that runs ahead of the act before it.** A `check` (a `Keep`, which reads once) or a `Focused` (which finds once and reads the node's focus once) fails right after a press or a key, and the snapshot shows the state the story expected. Default: before that read, add a step that waits for what a person would see of the same state, and keep the read: a `see` of the status words that act writes (`Pressed N`, where that act writes them), or a `harness::Step::Person(Step::Gone { find: Find::exactly_named(Role::MenuItem, b"Menu item"), within: Duration::from_secs(2) })` before a `[role=menu][hidden]` check. If only a change to skein's harness or face would wait for it, don't make it: list it under `## Left` with the step, message and snapshot lines.
- **C: a press landing where the story didn't mean.** `browser refused act: Covered`, `Hidden`, a press deadline passed, or the status reading another token than the story's press writes. Read the snapshot for what covered or moved. If the story's own steps left it in the way, fix the story's order or wait as in B, keeping its checks. If the page moved under the kit's press (between its hit test and its mouse input), that is the browser kit's, not this task's: list it under `## Left` with the logs.
- **D: a fault in skein-ui-web's folding.** Past a settled page, a menu stays open after a press outside it, a panel that folds away stays open, or the focus lands where section 7 says it doesn't, and it repeats. That is a person's fault. First its focused test, in the lowest tier that shows it: if the pure half (`skein/crates/skein-ui-web/src/page.rs`) is at fault, a case in `skein/crates/skein-ui-web/src/tests/page/events.rs`, as a row of `each_event_gives_its_act`'s table if it is an event's act, or else by turning the nearest outside-press test (`a_press_outside_goes_on_to_what_it_lands_on_and_keeps_its_focus` and its neighbours) into a table over its cases, the new case one row. If the browser half (`src/page/web.rs`, its listeners' order or `press_outside`) is at fault, which no focused test can see, the story is the lowest tier that shows it: say so. Then the fix, in `page.rs` or `page/web.rs`. Those three files are pre-approved for this case alone, and so is `skein/docs/design/ui-web.status.md`'s first **Departs** entry under 7, if the fix changes what it says. Run `heavy cargo nextest run -p skein-ui-web`, and `heavy cargo check --tests -p skein-ui-web-tests -p skein-fake-person-browser -p jig-client-shell-web -p temper-client-shell-web`. Say in the report's first lines that it is a skein-ui fault, for skein/tiers. The overlap with `look-round-16/field-options` is in notes/common.md.
- **E: no failure in thirty runs.** Make Step 3's change anyway: the code shows the race, and red-57 met it in the third story. Say the runs didn't reproduce it.
- **Several causes:** fix each by its option, the shared one once for all three.
- **A failure at the arrival's own `See` after its 2 s:** don't widen it. Report the run with its timing; it is a starved run, for the director.

## Steps

1. **Build what the stories need,** through heavy, one command at a time, from the repository root (`tools/tiers/design.md`, 2, the `browser-skein` row; the tier tool's `bundle`, which you don't run). `cargo metadata` builds nothing and runs directly:

   ```sh
   target="$(cargo metadata --format-version 1 --no-deps | python3 -I -c 'import json, sys; print(json.load(sys.stdin)["target_directory"])')"
   heavy cargo build --target wasm32-unknown-unknown -p skein-ui-example-shell-web
   heavy cargo run -p skein-ui-bundle -- \
     --module "$target/wasm32-unknown-unknown/debug/skein_ui_example_shell_web.wasm" \
     --name skein_ui_example_shell_web --prefix /bundle/ \
     --stylesheet skein/examples/ui/shell-web/notes.css \
     --placeholder skein/examples/ui/shell-web/loading.html \
     --placeholder-stylesheet skein/examples/ui/shell-web/loading.css \
     --noscript 'Notes needs WebAssembly and scripts.' \
     --out "$target/ui-example-bundle"
   heavy cargo build -p skein-ui-example-server
   ```

2. **Run each story alone,** five times each before any edit (and five more each if none failed), one command at a time, checking each run selected one test:

   ```sh
   NEXTEST_DOUBLE_SPAWN=0 SKEIN_UI_EXAMPLE_BUNDLE="$target/ui-example-bundle" \
     heavy cargo nextest run -p skein-browser-world --profile browser-skein --test browser_real \
     -E 'test(=folds_away::a_press_outside_closes_a_menu_and_goes_on)'
   ```

   and likewise `folds_away::a_panel_folds_away_without_taking_the_outside_press_or_a_forms_escape` and `folds_away::removing_a_focused_form_returns_to_its_disclosures_summary`. A failure's `assert_eq` prints the harness's `Outcome::Failed { person, step, report }`: `step` counts the script's steps from 0 (a failure before the script starts says 0 too), and `report` holds the failure's message, the page's accessibility snapshot, its screenshot's path and the server's stderr tail. Read it by `grep` and `tail` around the failure, never whole. If the test panics `build and supply the example bundle` with the variable set, pass it inside heavy's command instead: `heavy env NEXTEST_DOUBLE_SPAWN=0 SKEIN_UI_EXAMPLE_BUNDLE=... cargo nextest run ...`, and say so.

3. **Option A's or E's fix,** in `folds_away.rs`. One helper starts every story, and each story's script is built by it:

   ```rust
   /// A go ends at the page's load, and the loader starts the module after it
   /// (skein/docs/design/ui-web.md, 4.3 and 14.2), so each story sees the
   /// fixture's first apply before it acts or reads.
   fn arrive(steps: impl IntoIterator<Item = harness::Step>) -> Script {
       let arrived = [
           harness::Step::Person(Step::Go { address: b"/test/folds-away".as_slice().into() }),
           harness::Step::Person(Step::See {
               find: Find::exactly_named(Role::Disclosure, b"Editor"),
               within: Duration::from_secs(2),
           }),
       ];
       Script::new(arrived.into_iter().chain(steps).collect())
   }

   #[test]
   fn a_press_outside_closes_a_menu_and_goes_on() {
       let mut person = arrive([
           disclosure(b"Panel"),
           press(b"Actions"),
           // ... every step after the story's `Go`, as it stands
       ]);
       assert_eq!(harness::Harness::new(setup()).run(&mut [&mut person]), Outcome::Done);
   }
   ```

   `removing_a_focused_form_returns_to_its_disclosures_summary` loses its own `Go`, its comment and its `See` of `Editor`, which the helper now carries, so its step numbers don't move; the other two's move by one. One apply draws the whole tree, so seeing `Editor` shows every node is there. Then B's, C's or D's changes, only where the runs showed them.

4. `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`; `heavy cargo clippy -p skein-browser-world --all-targets -- -D warnings`; `heavy cargo check -p skein-browser-world --tests`; `heavy cargo nextest run -p skein-browser-world` (the focused profile; check it selected tests). Then each story alone five times, as in Step 2. Under D, Step D's runs as well. The citations check (notes/common.md). Commit and push.

## Report

As the implementer's text says. First the runs before the edit and after: each outcome, and for each failure its step, its message and what its snapshot showed. Then which option the runs showed and why, and, under D, that it is a skein-ui fault. Since the stories add no test, the rule-breaking check of a new test doesn't apply; say so. If another story in `skein/tests/browser` acts or reads straight after a `Go` with no wait, list it under `## Left`: it is outside this task.
