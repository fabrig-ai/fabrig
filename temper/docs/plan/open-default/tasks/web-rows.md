+++
id = "web-rows"
part = "web-rows"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-client-shell-web/src/limits.rs",
  "temper/crates/temper/tests/suite/configuration.rs",
  "temper/crates/temper/Cargo.toml",
  "Cargo.lock",
  "temper/tests/client-browser/tests/browser_all/browser_task_pages.rs",
  "temper/docs/design/client/README.status.md",
]
effort = "high"
minutes = 70
+++

# The browser's client receives the box's short lists: `rows` 8, as the terminal's

## Goal

Making a project, by the welcome's Open Default or the playground's start, drops the browser's
link to `temper serve` for good: the web client's `rows` (3) is below the eight entries the box
sends in `Me`'s creating offer, so the link cannot decode `Me` and drops its stream on every
connect (`notes/common.md`, *The fault*). Give the web client the terminal's eight, pin the two
shells' client bounds together, and show the link staying up in the browser tier.

Acceptance:

- `temper_client_shell_web::limits::client()` equals
  `temper_client_shell_terminal::limits::native().client`, field for field; a new test in
  `temper`'s `suite` binary says so, and fails on today's `rows: 3`.
- Browser test 11 (`browser_task_pages.rs`) also checks, after the playground's project is made,
  that the link is Live and not Behind on the Chats page, and again after a reload, with the
  story's chat still listed.
- Run before the fix (the new steps in, `rows` still 3), the browser tier shows the fault: test
  11, or another browser test whose deployment holds a project, fails at a step that waits on a
  delivered watch (Live, Chats, a chat or card). After the fix, all 11 browser tests pass, and
  the tier's CPU figures are within `tools/tiers/budgets.toml`'s `[browser-temper]`.
- The `2:` item of `README.status.md`'s **Not yet** is gone.

## Inputs

- `temper/docs/plan/open-default/notes/common.md`: the fault, the files to keep off, the
  browser tier and its budget.
- `temper/docs/design/client/README.md`, 2: the paragraph *Both shells receive eight entries
  ...* and the bullet *The client's limits* (this cut wrote the paragraph).
- `temper/docs/design/client/browser.md`, 4.3 (the tier, its cap and budget) and 4.6, item 11.
- `temper/crates/temper-client-tools/src/tests/limits.rs`,
  `the_tools_preset_is_the_terminals_but_its_rows`: the shape of the equality test (read only).

## Owns and may change

- **`temper-client-shell-web/src/limits.rs`:** `rows: 3` becomes `rows: 8`. Nothing else there
  changes; the module doc's "the terminal's client bounds" and `client()`'s "The root's bounds:
  `temper-client-shell-terminal`'s `native().client`" become true as written.
- **The guard, `temper/crates/temper/tests/suite/configuration.rs`:** a new test beside
  `the_process_and_client_world_documents_fit_their_clients`:

  ```rust
  #[test]
  fn the_browsers_client_bounds_are_the_terminals() {
      assert_eq!(
          temper_client_shell_web::limits::client(),
          temper_client_shell_terminal::limits::native().client,
          "the browser receives what the terminal does (client/README.md, 2)"
      );
  }
  ```

  The terminal's documents-fit tests in this file then hold for the browser too; add nothing
  else to them.
- **`temper/crates/temper/Cargo.toml`:** `temper-client-shell-web.workspace = true` in
  `[dev-dependencies]` (beside `temper-client-shell-terminal`), and `Cargo.lock`'s resulting
  line in `temper`'s dependency list. If `temper-client-domain::Limits` (the type `client()`
  returns) does not implement `PartialEq` and `Debug`, compare the `jig_client::Limits` inside
  (`.client`) and the `forge` and `accounts` fields one by one instead, and say so.
- **Browser test 11, `browser_task_pages.rs`:** after the story's last step
  (`Step::Address(b"/tasks/3"...)`), append, as one more numbered comment (`// 6:`) citing
  `temper/docs/design/client/README.md, 2`:

  ```rust
  // 6: the playground's project made, the link stays up across a reload: the page's own
  // client bounds receive the box's Me (client/README.md, 2).
  Step::Person(PersonStep::Go { address: b"/chats".as_slice().into() }),
  see(heading(1, b"Chats")),
  see(tier.states().live),
  Step::Person(PersonStep::Gone { find: behind(), within: ABSENT }),
  see(/* the story's chat on the Chats page */),
  Step::Person(PersonStep::Reload),
  see(heading(1, b"Chats")),
  see(tier.states().live),
  Step::Person(PersonStep::Gone { find: behind(), within: ABSENT }),
  see(/* the story's chat again */),
  ```

  `behind()` is `Find::exactly_named(Role::Status, b"Behind")`, the frame's Behind status
  (`temper-client-view`'s `frame.rs` ~341; check `Word::Behind`'s English words in `words.rs`
  and use them if they differ). The story's chat is the one "Add CSV export to widget" started:
  find it as the Chats page lists it (its link, as `browser_sign_in.rs` finds `FIRST_CHAT`, or
  its title); the find is yours, and must be one only a delivered watch draws. `ABSENT` is
  `temper_client_browser_world::ABSENT`. Extend the module doc's first line to say the story
  also holds the link across a reload once the project is made. No other test, file or step of
  the tier changes, and no new test.
- **Status, after the code:** in `temper/docs/design/client/README.status.md`, remove the
  **Not yet** item that begins `2: the browser's shell receives three entries` through
  `browser's link.`, so the bullet begins `- **Not yet:** 3.4, 5.5:`; re-wrap that first line
  only. This departs from the implementer's text ("Not yours: any `docs/design/`"): this one
  status edit is the task's.

Pre-approved:
- If the guard test shows another field that differs between the two shells, stop: that is a
  question (name the field and both values).
- If the browser tier cannot run in the slot (`notes/common.md`), the fix, the guard and the
  status edit still land; put the tier's output under `## Left`.
- If no browser test fails before the fix, still land the fix, the guard and the new steps (a
  web bound below the box's `documents.lines` is a fault whatever the tier shows), and report
  the tier's before-run first under `## Left`, with the steps it passed.
- If a browser test fails after the fix at a step this task did not add, run it once alone; a
  pass is a flake (both logs); a failure is reported under `## Left` with its step, if your new
  steps and the guard pass.

## Steps

1. The guard test and the `Cargo.toml` line; run it red on `rows: 3`:
   `heavy cargo nextest run -p temper --test suite -E 'test(=configuration::the_browsers_client_bounds_are_the_terminals)'`
   (check the count is 1; take the module path from `tests/suite/main.rs` if it differs).
2. Test 11's new steps. With `rows` still 3: `temper/scripts/browser.sh` (the whole tier,
   once): record which tests fail and at which step (the harness prints it).
3. The fix (`rows: 8`). Step 1's test again: green. Then `temper/scripts/browser.sh` again: all
   11 pass. Then the CPU figures: `tools/tiers/run --cpu <new dir> temper browser`.
4. The change of meaning reaches the web client's readers: `heavy cargo nextest run -p
   temper-client-shell-web`, then `heavy cargo nextest run -p temper-client-journeys`. The wasm
   build is the tier's bundle step (step 3).
5. `cargo fmt`; `cd temper && python3 -I -B scripts/old_forms.py`; `heavy cargo clippy -p
   temper -p temper-client-shell-web -p temper-client-browser-world --all-targets -- -D
   warnings`; `heavy cargo nextest run -p temper --test suite -E 'test(/^configuration::/)'`.
6. The status edit; the citations check (`notes/common.md`); commit (the fix and its guard; the
   browser steps; the status), and push.

## Report

As the implementer's text says, and: the before-run's failing tests and steps (step 2), the
after-run's counts, the tier's CPU figures against its budget (suite and slowest test), the find
you chose for the story's chat, and whether `Limits` compared whole.
