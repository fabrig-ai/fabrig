+++
id = "demo-project"
part = "demo"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-engine-domain/src/route/playground.rs",
  "temper/crates/temper-client-view/src/pages.rs",
  "temper/crates/temper-client-view/src/tests/frame.rs",
  "temper/crates/temper-client-view/src/tests/task.rs",
  "temper/crates/temper-client-view/src/tests/playground.rs",
  "temper/tests/client/tests/suite/welcome_m4.rs",
  "temper/crates/temper/tests/support/playground.rs",
  "temper/crates/temper/tests/support/person.rs",
  "temper/crates/temper-client-shell-terminal/src/tests/golden/first_run.txt",
  "temper/crates/temper-client-shell-terminal/src/tests/golden/chat_delegation_landed.txt",
  "temper/docs/design/protocol/setup.status.md",
  "temper/docs/design/client/README.status.md",
]
effort = "medium"
minutes = 45
+++

# The playground's project is "Demo", shown by its stored name

## Goal

The playground's start makes an ordinary project named `Demo` (today `Playground`), and the
client names the playground's project by the name its record holds, never by a fixed word, so a
renamed Demo shows its new name in the banner, the context line, the chats, a task's page and
the inbox (`temper/docs/design/protocol/setup.md`, 8.4, its start's act 2;
`temper/docs/design/client/README.md`, 5.6, "Its names").

Acceptance:

- The playground's start sends jig's `MakeProject` named `Demo` on policy 1.
- `temper-client-view`'s `pages::project_name` returns the project's name from `me` for every
  project, the playground's among them; it no longer reads the playground.
- The tests below pass; the two status entries below are gone.

## Inputs

- `temper/docs/plan/first-run-projects/notes/common.md`.
- `temper/docs/design/protocol/setup.md`, 8.4: "Its start", act 2.
- `temper/docs/design/client/README.md`, 5.6: "Its names" and "The context line" (PROJECT the
  project as the banner names it); 4.8 (what shows) for context only.

## Owns and may change

- `temper/crates/temper-engine-domain/src/route/playground.rs`, `project` (line ~116): the
  `MakeProject`'s `name` becomes `"Demo"`. Nothing else in the route changes.
- `temper/crates/temper-client-view/src/pages.rs`, `project_name`: the playground arm goes, and
  with it the `playground` parameter:

  ```rust
  pub(crate) fn project_name(me: Option<&jig_client::contract::Me>, project: u32) -> Option<Box<[u8]>>
  ```

  If `words` is then unused, drop it too. Its call sites change only in their arguments (call
  sites only): `root.rs` (`adoption_main`), `frame.rs` (`project_buttons`), `pages/task.rs`,
  `pages/chats.rs`, `pages/chat.rs`, `pages/inbox.rs` (two), `forge/connect.rs` (two in
  `project_name`). `frame.rs`'s banner is another plan's look (`notes/common.md`): there, edit the
  one call's arguments and nothing else. `Word::Playground` stays: it names the playground's
  banner region and the accounts' section, not a project.
- Tests that encode the fixed word, each kept to its purpose:
  - `temper-client-view/src/tests/frame.rs`,
    `the_running_playground_project_button_is_named_playground` becomes
    `the_running_playground_project_button_is_named_by_its_record`: the button is found by the
    fixture's stored project name (the `me` row's, `b"Shop"` today) and presses to
    `Act::Project`; no button is named `Playground`.
  - `temper-client-view/src/tests/task.rs` (line ~51): the task header holds the stored name, not
    `Playground`.
  - `temper-client-view/src/tests/playground.rs` (line ~278), in
    `the_playground_chooser_names_the_provider_and_model_but_keeps_the_configured_choice`: if the
    `Paragraph` `Playground` it finds is the context line's project, it now finds the stored
    name; if it is another node (the banner's words), leave it and say so.
  - `temper/tests/client/tests/suite/welcome_m4.rs`, `running_playground_project_is_named_playground`
    becomes `running_playground_project_is_named_by_its_record`, asserting the fixture's stored
    name and no node named `Playground` among the project buttons.
  - `temper/crates/temper/tests/support/playground.rs` (lines ~134, ~187, ~201):
    `"Playground"` becomes `"Demo"` (the project the reset must remove).
  - `temper/crates/temper/tests/support/person.rs` (lines ~165-166): the project button the person
    presses is `Demo`.
  - The terminal shell's goldens `temper-client-shell-terminal/src/tests/golden/first_run.txt`
    and `chat_delegation_landed.txt` (lines ~4 and ~18: `[2] Playground (pressed)`,
    `Playground · playground/widget…`) print the playground's project: regenerate them with
    `SKEIN_CODEGEN_WRITE=1` on `render.rs`'s `each_fixture_page_prints_as_its_golden_file`
    (`notes/common.md`), so they print the fixture's stored name; read the diff and keep only
    those lines. If `the_first_run_prints_forges_and_the_playground_banner` asserts the fixed
    word, it asserts the stored name instead.
  - Any other test that fails because the playground's project is now named `Demo` or by its
    record: change its literal alone, and list it in the report. `temper/tests/real-loop/src/calls.rs`'s
    unit test of `ReadCondition::NoProjectNamed` (line ~1539) tests the matcher on its own literal:
    leave it.
- Status, as `notes/common.md` says:
  - `temper/docs/design/protocol/setup.status.md`, **Not yet**: remove the sentence `8.4: the
    playground's start names its project \`Playground\`, not \`Demo\`
    (\`temper-engine-domain/src/route/playground.rs\`, \`project\`).`
  - `temper/docs/design/client/README.status.md`, **Not yet**: remove the sentence `5.6: the
    playground's project named "Playground" by the phrasebook, not by the name its record holds
    (\`temper-client-view\`'s \`pages.rs\`, \`project_name\`).`

## Steps

1. The route's name; `heavy cargo nextest run -p temper-engine-domain --lib -E 'test(playground)'`
   and `heavy cargo nextest run -p temper-engine-domain-world --test suite -E 'test(playground)'`.
2. `project_name` and its call sites; `heavy cargo nextest run -p temper-client-view --lib -E
   'test(playground) | test(frame) | test(task)'`; `heavy cargo nextest run -p
   temper-client-shell-terminal --lib -E 'test(render)'` and the goldens as above; then `heavy cargo nextest run -p
   temper-client-world --test suite -E 'test(welcome_m4)'` (find the package that owns
   `temper/tests/client` in its `Cargo.toml`).
3. The process tier's support (`crates/temper/tests/support/`) is compiled by `temper`'s test
   binaries: `heavy cargo check -p temper --tests`; run the story that presses the project button
   if its test binary is in the focused profile, else say it was only checked.
4. The two status edits; the citations check.
   `temper/docs/help/playground.md` already names the project `Demo` (the lead's text, on the
   trunk): run `heavy cargo nextest run -p temper-docs` once to see the help still checks.
5. `cargo fmt`; old forms for temper; `heavy cargo clippy -p temper-engine-domain -p
   temper-client-view -p temper --all-targets -- -D warnings` (add the client world's package);
   the focused tests of `temper-engine-domain`, `temper-client-view` and their direct dependents
   in temper (`temper-service`, `temper-client-shell-web`, `temper-client-shell-terminal`).
   Commit the code and tests, then the status text; push.

## Report

As the implementer's text says, and each test whose literal you changed beyond those listed.
