+++
id = "init-box"
part = "document"
kind = "feature"
needs = ["welcome-member"]
files = ["temper/crates/temper-init/src/lib.rs", "temper/crates/temper-init/src/template.rs", "temper/crates/temper-init/template/**", "temper/crates/temper-init/src/tests/**", "temper/crates/temper/src/init.rs", "temper/crates/temper/src/init/tests.rs", "temper/crates/temper/src/lib.rs", "temper/crates/temper/src/commands.rs", "temper/crates/temper/src/tests.rs", "temper/crates/temper/src/tests/commands.rs", "temper/crates/temper-service/src/startup/reference.rs", "temper/crates/temper-docs/src/lib.rs", "temper/docs/help/reference/**", "temper/docs/design/protocol/setup.status.md"]
effort = "high"
minutes = 90
+++

# `temper init` writes a bare deployment; `--box` adds the box's parts

## Goal

`temper init` alone writes a bare deployment that `temper serve` starts: no welcome, no playground,
no Forgejo on this machine, and it ends saying to start it with `temper serve`. `temper init --box`
writes what init writes today, with `"welcome": true`, and ends saying `temper box`. Every
test that wants the box's parts asks for them.

Acceptance:

- `temper init --config C --data D --cache K` writes `worker.secret`, `worker.digest`, `agent.json`
  and `temper.json`, no `demo/` directory; the document is contracts K3's; it holds two listeners;
  its stderr ends `init: start it with temper serve --config C --data D --cache K`, its person
  lines never printed.
- `temper init --box` writes today's six files, the document with `application.welcome` `true`,
  and its stderr ends `init: start it with temper box` with the same flags.
- `temper box` on empty directories and `temper box --fake-backends` behave as they do today, with
  `application.welcome` `true` in the document they run.
- Refusals: `temper init` is refused by `temper.json`, `worker.secret` and `worker.digest`;
  `--box` by those and by `demo/llm.key` and `demo/bin/git`; any other word after `init` is refused
  as it is today.
- Tests that pin it, each named here, all new, all biting:
  - `temper-init/src/tests/documents.rs`, `the_bare_document_is_the_box_less_its_parts`: the two
    documents and the two agents' configurations, parsed, are equal once the parts K3 lists are
    removed from the box's, so the templates cannot drift; `Refusal`s as `files` answers them.
  - `temper/src/init/tests.rs`, `a_bare_configuration_passes_the_same_checks_as_serve`, twin of
    `generated_configuration_passes_the_same_checks_as_serve`: `crate::setup::check` passes,
    `machine.owned` and `machine.fake` are none, and no `demo/` directory exists; and a refusal row
    for each file of the acceptance, writing nothing.
  - `temper/tests/suite/startup.rs`, `serve_prepares_once_on_a_bare_init`, twin of
    `serve_prepares_once_on_the_owned_example`; a bare row in the stderr test at
    `temper/tests/suite/startup.rs:1214-1226`, a `--box` row keeping today's assertions.
  - The parse table in `temper/src/tests.rs` (`(vec!["init"], true)`) gains `init --box`, true, and
    `init --box --box`, false.
- `heavy cargo nextest run -p temper-init`, `-p temper --lib`, `-p temper --test suite`,
  `-p temper-client-browser-world --no-run` are green (the last builds the tier; its tests are
  not the gate's), and `the_pages_configuration_are_what_the_tables_render` and the commands
  page's test pass on the regenerated pages.
- `temper/docs/design/protocol/setup.status.md` loses "7.1, 8.2, 8.3: `temper init` has no
  `--box`: it writes the box's parts always, says to start with `temper box`, and `temper box`'s
  first run passes it no flag."

## Inputs

- `temper/docs/plan/serve-without-box/notes/common.md`; `notes/contracts.md`, K1 and K3.
- `temper/docs/design/protocol/setup.md`, 7.1 (An empty configuration directory), 8.2, 8.3.
- `temper/docs/design/draft/serve-without-box.md`, 2.

## Owns and may change

- The files of the header, and creating `temper-init/template/bare.json` and `bare-agent.json`.
- **`"welcome": true`, one line each:** `temper-init/template/temper.json`,
  `temper/crates/temper-playground/stories/a/deployment.json`, and the `application` literal of
  `temper/tests/client-browser/src/configuration.rs:84`. They are inert until the client reads the
  fact. Regenerate story A's log if its document moves it (`notes/common.md`).
- **Readers that want the box's parts, `.arg("--box")` after `.arg("init")` alone:**
  `temper/crates/temper/tests/support/forge_land.rs:42`,
  `temper/crates/temper/tests/support/sign_in.rs:36`,
  `temper/crates/temper/tests/suite/startup.rs:13,351,376,1614`,
  `temper/crates/temper/tests/suite/box_listeners.rs:93`,
  `temper/crates/temper/tests/process.rs:1920,2099`,
  `temper/tests/client-browser/src/configuration.rs:153`. A line here that a test's edit shows
  needs nothing, leave it and say so. Any other test that runs `temper init` and then fails
  on a missing box's part: the same one-word edit, listed in the report.
- `temper/crates/temper/src/commands.rs`, the init row: `forms` `["[DIRECTORIES] [--box]"]`, an
  argument `--box`, "Also write the box's parts: the welcome, the owned Forgejo and the
  playground.", and its notes in those terms: alone, two ports, preferring 8367 for clients and
  8371 for webhooks; with `--box`, the five as they read today. The rest of its notes stay. Keep
  the words `ports` and `init` where `temper/src/tests/help.rs:509` needs them.
- The reference's head line, `temper-service/src/startup/reference.rs:9,15` and its twins in
  `temper-docs/src/lib.rs:456,463`: "what temper init --box writes". `init_document()` in
  `temper/src/tests/configuration.rs` stays on `files`, the box's document.
- The prose help pages are `help-box`'s: `temper/docs/help/box.md` is read by
  `temper/src/tests/help.rs`; change it only if a test of this task fails without it, one word
  (`temper init --box`), and say so.
- If `temper-init`'s `template.rs` `#![expect]` or its "every placeholder is known" test needs a
  new placeholder set, widen them. Default for the templates' shape: two whole files, the crate's
  way, over any splice.
- `temper/crates/temper/src/box_command.rs` and `fake_backends*`: no edit is expected. If one is
  needed, make it and say why.

## Steps

1. `temper-init`: `Bare`, `bare`, the two templates; the shared part of `template::documents`
   reused, not copied, where it can be.
2. `init.rs`: split the checks and the writing so that `write_with_program` and `write_bare` share
   them; `bare`, `run(directories, boxed)`, the last line. `Written.person` for a bare write is
   `Person::Found` with no login: a bare document names no person.

   ```rust
   ["init"] if rest.is_empty() => Ok(Command::Init { directories, boxed: false }),
   ["init"] if rest == ["--box"] => Ok(Command::Init { directories, boxed: true }),
   ```

3. The box's `"welcome": true` and the two other documents; the readers' `--box`.
4. The tests of the acceptance; `commands.rs`; regenerate the pages (`notes/common.md`; the
   commands page by `temper/src/tests/commands.rs`'s `SKEIN_CODEGEN_WRITE`).
5. The implementer's checks, packages `temper-init`, `temper`, `temper-service`, `temper-docs`,
   `temper-playground`; commit and push.

## Report

As the implementer's text asks; list every `--box` edit made or found unneeded.
