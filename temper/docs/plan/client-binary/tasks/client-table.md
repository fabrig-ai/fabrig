+++
id = "client-table"
part = "client"
kind = "feature"
needs = ["shell-table", "tui-start", "ctl-start", "client-version"]
files = [
  "temper/crates/temper-client/**",
  "Cargo.lock",
  "temper/docs/design/protocol/README.status.md",
  "temper/docs/design/client/README.status.md",
  "temper/docs/design/draft/deployment-binaries.execution.md",
]
effort = "high"
minutes = 90
+++

# `temper-client`'s table: the TUI, `ctl` and `help`, by the box's code

## Goal

Give the deployment's client its table of three rows, `help`, `[ADDRESS]` and
`ctl`, its parser, `--help`, `COMMAND --help` and its refusals, and run each row
by the code the box's `temper` runs: `temper_client_shell_tui::start::run` and
`temper_client_shell_cli::start::run`, named `temper-client`
(`temper/docs/design/protocol/README.md`, section 4, *The deployment's client*;
`temper/docs/design/client/README.md`, section 2 and 6.11). Then close the
plan's status entries: the client is built.

Acceptance:

1. **`src/lib.rs`** gains `COMMANDS`, `Command`, `configure` and `run` as
   `notes/contracts.md`, section 9, has them:
   - `configure`: exactly `--help` is `Overview` and exactly `--version` is
     `Version`; otherwise the row `temper_shell::commands::find(&COMMANDS, …)`
     gives, else `Err(unknown(NAME, &COMMANDS, arguments))`; the row's words then
     exactly `--help` is `Usage(row)`; then, as `temper`'s `configure_row` takes
     these three rows: `[]` with no word `Tui { address: None }`, one word
     `Some`, more refused; `["ctl"]` `Ctl` with the rest as `OsString`s;
     `["help"]` `Help` by `temper_shell::help::request`, `None` refused. A refusal
     is `refused(NAME, row, "invalid arguments")`.
   - `run`: `Overview` prints `help(NAME, &COMMANDS)`, `Version` `version()`,
     `Usage` `row(NAME, row)`, `Help` `answer(NAME, …)` (its `Err` returned), each
     on stdout, `Ok(0)`; `Tui` returns `start::run(NAME, address.as_deref(),
     &Clock::new())` of the TUI's shell; `Ctl` returns `start::run(&arguments,
     &Clock::new())` of `ctl`'s.
2. **`src/main.rs`** as contracts, section 9: `configure` then `run`, `Err`
   printed on stderr whole, exit 1. The interim refusal of `client-version`
   goes.
3. **Dependencies:** `temper-shell`, `temper-client-shell-tui` (no `record`),
   `temper-client-shell-cli`, `skein-shell`, `skein-fail-stop`; nothing else.
4. **What it links** (`cargo tree`, which builds nothing, run directly):
   `cargo tree -p temper-client -e normal --prefix none` names none of `temper
   v` (the box), `temper-service`, `temper-box`, `temper-init`, `temper-fake`,
   `temper-forge-setup`, `temper-forge-setup-shell`, any `temper-engine-*`,
   `temper-worker-*`, `temper-client-shell-web`, `temper-client-bundle`,
   `smith-agent-*`, `smith-local-*` or `smith-host-*`; and `cargo tree -p
   temper-client -e features -i temper-protocol-records` shows no `forge`
   feature. The report lists the closure's crates that are none of temper's
   client crates, jig's client crates, skein's or `temper-shell` (plan.md, *What
   the client links*, expects them).
5. **Tests,** each seen failing first:
   - `src/tests.rs`: `every_row_parses_and_the_rest_is_refused`, a table over
     `configure`: none, an address, two addresses (`temper-client: invalid
     arguments\nusage: temper-client [ADDRESS]\n`), `bogus`, `ctl DIR me`, `ctl
     --help`, `help`, `help tui`, `help --search sign out`, `help -x` (refused),
     `--help`, `--version`, `--version extra`, and each of `box`, `serve`, `init`,
     `open DIR http://127.0.0.1:8367`, `internal agent A.json`, `fake git`,
     `program-login NAME`, `forge fetch 1` refused as
     `temper-client: no command WORD; temper-client --help lists them\n`;
     `the_help_lists_the_clients_rows`: `help(NAME, &COMMANDS)` holds `temper-client
     [ADDRESS]`, `temper-client ctl DIR GROUP VERB [ARGUMENT...] [--json]`,
     `temper-client help [NAME | --search WORDS...]`, and no `box`, `serve` or
     `open` command line.
   - `tests/suite/help.rs` (with `mod help;`):
     `the_binary_answers_its_help_and_refuses_what_it_hasnt`, a table running the
     built binary, stdin null and stdout piped: `--help` (exit 0, stdout `help(NAME,
     &COMMANDS)`), `ctl --help` (exit 0, `row(NAME, &rows::CTL)`), `help` (exit 0,
     the index page), `box` (exit 1, its refusal), and bare `temper-client`, its
     stdin and stdout no terminal (exit 1, `temper-client: no command;
     temper-client --help lists them\n`). `client-version`'s rows in
     `tests/suite/version.rs` take the table's refusal words where they differ
     (`--version extra` is `no command --version`).
6. **Status and state:** delete, in `temper/docs/design/protocol/README.status.md`,
   the paragraph `Section 4's deployment client, `temper-client`, and section 5's
   `temper-shell` are not yet built:` and its **Not yet** bullet; in
   `temper/docs/design/client/README.status.md`, the paragraph `Section 2's
   clients' start, one for two binaries, is not yet built:` and its bullet; set
   M1's state in `temper/docs/design/draft/deployment-binaries.execution.md` to
   `built: plan `client-binary`; the ctl story (10) runs after landing`. Leave
   the client story's paragraph and bullet (`ctl-story`'s). If one of them doesn't
   hold now (something it names isn't built on your base), keep that part of it,
   true, and say so.

## Inputs

- `temper/docs/plan/client-binary/notes/common.md`; `notes/contracts.md`,
  sections 4, 5, 7, 8 and 9.
- Design: `temper/docs/design/protocol/README.md`, section 4 (*The deployment's
  client*); `temper/docs/design/client/README.md`, section 2 (*The clients' start,
  one for two binaries*) and 6.11 (*In the deployment's client*).
- The code: `temper/crates/temper-client/**` as `client-version` left it;
  `temper/crates/temper/src/lib.rs`' `configure_with` and `configure_row` (the
  arms for `[]`, `["ctl"]`, `["help"]`, and `Usage`) and `run`'s for `Usage`,
  `Help`, `Ctl`, `Tui`; `temper/crates/temper/src/main.rs`;
  `temper_shell::{commands, rows, help}`; `temper_client_shell_tui::start`;
  `temper_client_shell_cli::start`.

## Owns and may change

Owns `temper/crates/temper-client/**`, `Cargo.lock`, and the three design lines
of acceptance 6. Not yours: any other package (a change the client needs in a
shell crate is a question, unless it is a one-line `pub`).

Options, each decided:

- **The parser's three arms** are the client's own copy of `temper`'s for those
  rows (a dozen lines), not a call into the box's library: the client never links
  `temper`.
- **The index page** in the suite: compare with `temper_shell::help::answer(NAME,
  &Request::Index)`.
- **A row's prose names `temper`** (the shared rows'); that is designed
  (protocol/README.md, section 4): assert it as it prints.

## Steps

1. The failing tests of acceptance 5; then `lib.rs` and `main.rs`. `heavy cargo
   nextest run -p temper-client`.
2. The `cargo tree` checks of acceptance 4; report their output's relevant lines.
3. The status and state lines; `python3 -I -B tools/citations/citations.py --base
   origin/main`.
4. Once: `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`;
   `python3 -I -B tools/lints/lints.py`; `heavy cargo clippy -p temper-client
   --all-targets -- -D warnings`; `heavy cargo nextest run -p temper-client`.
5. Commit ("temper-client: its table of three rows, run by the box's code") and
   push.

## Report

`Done:`, `Failed:` or `Question:` first, then the files, the closure check's
findings, the runs and their counts, the rule each new test bit on, anything left.
