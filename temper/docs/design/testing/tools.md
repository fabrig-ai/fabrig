# temper's tools and fixtures

Provisional, 2026-11-11, from `../draft/cli-c8-tools.md`, milestone C8
of `../draft/cli.execution.md`, with landed QA's tool from milestone Q1
of `../draft/landed-qa.execution.md`. The binaries temper's development,
tests and QA run beside its products: each an internal binary in a package
of its own under `temper/bin/`, built as the repository's command line
says (`skein/docs/foundation/cli.md`). What each one does is designed
with its subject (section 2); this document says how each is laid out,
named, answered and tested.

## 1. In one page

- **Seven binaries,** each a package `temper/bin/<binary>/` whose
  `main.rs` installs fail-stop and calls its crate's `main`. The crate,
  `<binary>-tool` under `temper/crates/`, holds its table and everything
  it does (sections 2 and 3).
- **Every name says temper:** `twin`, `git-transcripts` and
  `playground-serve` become `temper-twin`, `temper-git-transcripts`
  and `temper-playground-serve`; the other three keep theirs (3.2).
- **Internal, all of it:** each table's commands, flags and temper
  variables are internal or test; the host's variables are public in
  effect. Each binary keeps what it does and its arguments (4.1).
- **One front:** `skein-shell`'s `Table::front` answers `-h`, `--help`,
  `help`, `--version`, a command's `--help` and a command the table
  lacks, and `Table::refusal` words a command's refused arguments
  (`skein/docs/design/runtime/shell.md`, 14.5; here 4.2).
- **Read once:** the crate's `main` captures the arguments and the
  environment, and every variable it reads after is a declared entry of
  its table (4.3).
- **Callers follow** the moved paths and the three new names: recipes,
  scripts, documents, tests, the tiers and the checks (section 5).
- **One table test per binary,** in the focused suite (section 6).

## 2. The binaries

Each binary's line is its table's, word for word: help's opening
paragraph prints it after the binary's name, wrapped at 80 columns
(`skein/docs/design/runtime/shell.md`, 14.2): where `BINARY: LINE` is
longer than 80 columns, as the gallery's, the journeys', the
transcripts' and playground-serve's are, it takes two lines of the page.

| Binary | Before | Its line | Designed in |
|---|---|---|---|
| `temper-client-gallery` | `temper/tools/gallery`, package and binary `temper-client-gallery` | the browser's pages in named states, printed for screenshots. | `../client/web/gallery.md` |
| `temper-client-journeys` | `temper/tools/journeys`, package and binary `temper-client-journeys` | recorded walks, replayed through the shipped hosts and printed. | `../client/web/journeys.md`, 5.1; `../client/tui/journeys.md`, 6.1 |
| `temper-git-transcripts` | `temper/tools/git-transcripts`, package and binary `git-transcripts` | git's transcripts, recorded from real git against a throwaway Forgejo. | `../protocol/git.md`, 9 |
| `temper-playground-serve` | binary `playground-serve` of `temper-client-browser-world`, `temper/tests/client-browser/src/bin/` | the browser tier's static host for the playground's bundle. | `../playground.md`, 7 |
| `temper-stage-one` | `temper/tools/stage-one`, package and binary `temper-stage-one` | configuring's stage 1: its golden, check, judge and evidence. | `checks.md`, 5 |
| `temper-twin` | `temper/tools/twin`, package `temper-twin`, binary `twin` | a live run's twin: a fake LLM document that replays its agents. | `fakes.md`, 1; `real-loop.md` |
| `temper-qa` | new, landed QA's | QA after landing: main's landings triaged into cases, and builds kept for them. | `qa/README.md` |

Their forms are today's:

- `temper-client-gallery OUT`;
- `temper-client-journeys`, `temper-client-journeys add [--redact
  TEXT]... NAME FILE` and `temper-client-journeys again NAME`;
- `temper-git-transcripts --git PATH --scratch DIR --out DIR`;
- `temper-playground-serve OUT`;
- `temper-stage-one obligations [--json]`, `collect` with its eight
  flags and `EVIDENCE`, `check EVIDENCE [--json]`, `judge EVIDENCE` and
  `verdict EVIDENCE [--json]`;
- `temper-twin --data DIR --base FILE --out FILE [--cue
  TASK=PHRASE]... [--replace OLD=NEW]...`;
- `temper-qa run`, `temper-qa build keep COMMIT DIR`, `temper-qa triage
  none WHY`, `temper-qa triage case FILE`, `temper-qa show` and
  `temper-qa lines` (`qa/README.md`, 2), new, its arguments its own.

## 3. A binary and its crate

### 3.1 The layout

- **The binary's package,** `temper/bin/<binary>/`, as
  `skein/docs/foundation/cli.md`, 2 has it: a `Cargo.toml` naming the
  package as the binary, one `[[bin]]` at `main.rs` beside it, no
  library, and its table test as `[[test]]` `suite` at
  `tests/suite/main.rs` (section 6), as `temper-worker`'s package has
  them. It depends on its crate and `skein-fail-stop`, and its tests on
  what they run.
- **Its `main.rs`,** the whole of it:

  ```rust
  //! The twin tool's binary (temper/docs/design/testing/tools.md, 3).
  fn main() -> std::process::ExitCode {
      skein_fail_stop::install();
      temper_twin_tool::main()
  }
  ```

- **Its crate,** `temper/crates/<binary>-tool/`: the tool's library as
  it was, with what its `main.rs` held beside it, its files with it
  (gallery's `names.txt`, the journeys' scripts, the transcripts and
  `facts.txt`, stage one's golden, the twin's fixtures, each `README.md`)
  and its tests. The playground's static host, today the whole of the
  browser world's `playground-serve.rs`, is its crate's library, and
  the world keeps its stories. The crate holds `pub const TABLE: Table`
  and `pub fn main() -> ExitCode` (4.3).
- **Lints:** both packages take the `harness` profile, as development
  tools do, each with its copy of the `ordinary` clippy variant
  (`tools/lints/design.md`, 3 and 4). `temper/tools/` goes with its
  last crate, and its `clippy.toml` with it.

### 3.2 The names

- **The binary's package is named as the binary,** so the library that
  held the name takes `-tool` after it: the binary's crate, as the
  worker's is `temper-worker-shell`, and named so that the binary finds
  its crate by its own name.
- **Every binary's name starts `temper-`:** temper's tiers select the
  tree's packages by that prefix (`.config/nextest.toml`), and a
  binary in the workspace's one target directory says its tree as a
  variable's name does (`skein/docs/foundation/cli.md`, 5). So `twin`,
  whose package was already `temper-twin`, takes its package's name,
  and `git-transcripts` and `playground-serve` take the prefix.
- **`cargo run -p <binary>`** runs each, as `cargo run -p temper-twin`
  and the other three already did.

## 4. Its table

### 4.1 The entries

- **The table** (`skein/docs/design/runtime/shell.md`, 14.1): its binary,
  its line (section 2), `temper-version`'s `VERSION` and `COMMIT`, its
  commands, no settings of its own, its variables, and as its last line
  where to read more: its design document from section 2, by path from
  the repository's root, and for a binary with commands of words,
  `BINARY COMMAND --help prints a command;` before it.
- **Its commands** are its forms (section 2), each `Set::Internal`,
  reached by its words or, for a bare form, by none. Each command's line
  is what its design says it does, and its notes keep what the tool's own
  help said, so that nothing it printed is lost: the journeys' cautions
  are `add`'s notes, and `add --help` prints them.
- **Their flags:** a flag with a value is a setting of the command's
  group (`skein/docs/design/runtime/shell.md`, 6.5), internal, with no
  twin, `Fallback::Refuse` where it's required and its default where it
  has one; one that repeats, as `--cue`, `--replace` or `--redact`, says
  so in its notes, and the tool reads it itself. A flag without a value,
  as `--json`, is a `Switch`.
- **Its variables:** every one it reads, and every `TEMPER_` one it sets
  for the programs it runs, under today's name and in the set
  `../draft/cli.md`, 4.6 gives it:
  - test: `TEMPER_TEST_FORGEJO`, read by the transcripts' tool, and skein's
    `SKEIN_TEST_CHROMIUM`, read by the journeys';
  - internal: `TEMPER_INTERNAL_BUNDLE`, which the journeys' tool sets for the
    builds it runs;
  - public, as `skein-shell`'s `HOME` is: the host's and Cargo's, as
    `PATH` and `CARGO_TARGET_DIR`.

  C2 and C4's names are shared with the products and test table
  (`../draft/cli.execution.md`); skein's names follow its round.

### 4.2 Its answers

- **The front first:** the crate's `main` hands the captured arguments
  to `TABLE.front` (`skein/docs/design/runtime/shell.md`, 14.5) and
  writes what it prints to stdout with exit 0, or what it refuses to
  stderr with exit 2. So `-h`, `--help`, `help` and `--version` are the
  front's in every binary, and neither `OUT` can be named `help`.
- **A command's run:** `Front::Run` names the command, which the crate
  matches against its table's constants and runs on its own arguments,
  as the tool did. Arguments it refuses are refused with
  `TABLE.refusal`'s words and exit 2, where each tool printed a usage
  line of its own. Past its arguments, a run's output, its words for
  what failed or what it refused, and its exit codes are the tool's, as
  before, under the binary's new name where they print one. A failure
  that Rust's `Result` from `main` reported, `Error: "WHY"` or `Error:
  WHY` with exit 1, as the transcripts' tool and the playground's host
  did, is `BINARY: WHY` with exit 1.
- **Help is the table's:** a binary's page lists no internal entry, so
  a tool's page is its line, the three flags of every binary, its public
  variables and its last line; a command's page, `COMMAND --help`, says
  it is internal and gives its forms, flags and notes.

### 4.3 The environment

- **Captured once:** the crate's `main` calls `skein_shell::arguments`
  and `skein_shell::environment` under one scoped
  `#[expect(clippy::disallowed_methods, reason = "the binary's
  capture: …")]` (`skein/docs/foundation/cli.md`, 6 and 9, check 4),
  and hands them down. Nothing else in the crate reads the process's.
- **Read through the table:** a lookup is a declared `Variable`'s
  `value` in the captured pairs, never a string; a child the tool starts
  with the environment it was given takes the captured pairs.
- **The tests' own reads,** including a golden's rewrite switch, use
  temper's test table (`../cli/README.md`, 11).

### 4.4 The reference documents

Each binary's `<binary>-args.md` and `<binary>-env.md` are rendered
from its table into `temper/docs/design/cli/`, with temper's common and
test documents (`skein/docs/foundation/cli.md`, 8), and checked against
it byte for byte in its crate's tests.

## 5. Its callers

- **Recipes and scripts:** `cargo run -p` with each binary's name
  (3.2), `--bin temper-twin` where `--bin twin` was, and every path
  under `temper/tools/` or the browser world's `src/bin/` at the crate's
  or the package's new place.
- **A test of the binary's own package** runs it by
  `CARGO_BIN_EXE_<binary>` (`skein_scratch::paths::binary`). A test of
  the tool that runs its binary goes to that package's suite; a test of
  its library stays in its crate.
- **A test of another package** finds the binary beside its own
  executable, as the browser world finds `temper-box`
  (`temper/tests/client-browser/src/lib.rs`, `temper`), and its tier
  builds it first: the browser tier's playground story runs
  `temper-playground-serve`, which `tools/tiers/packages.toml` lists
  among `browser-temper`'s binaries.
- **The gate's and the tiers' lists** name the packages as they now are:
  `.config/gate/always.txt` the packages whose tests read other
  packages' files, `.config/fixture-readers.toml` the twin's fixtures,
  and `.config/nextest.toml`'s filters, whose `package(=git-transcripts)`
  goes since the transcripts' packages take temper's prefix.
- **Documents:** each design document, status file, `README.md` and
  comment naming a moved path or an old name, the layout of
  `README.md`, section 5 among them.
- **The checks:** each binary's move removes its `cli C8` exceptions in
  `tests/boundary/tests/cli.rs` and every `expect` whose reason starts
  `cli C8` (`../draft/cli.execution.md`, 2). A `cli C5` exception that
  names a moved manifest follows it, its path changed alone.
  `temper-forge-setup-shell`'s scan of process and lock operations
  (`tests/suite/boundary.rs`, which reads every crate under
  `temper/crates/`) skips the tools' crates by name, as it never read
  `temper/tools/`: the tools spawn builds, browsers, git and agents by design,
  outside the setup's process boundary (`../protocol/setup/README.md`,
  7). Every other crate stays scanned as before, and the twin's per-file
  `ExitCode` exemption goes with the skip.

## 6. Its tests

- **One table test per binary,** its package's `tests/suite/main.rs`,
  in the focused suite: each row runs the built binary with its
  arguments, a cleared environment and stdin closed, in a scratch
  directory, and checks its exit code, stdout and stderr, as
  `temper-worker`'s does. The rows:
  - `--help`, `-h` and `help` print the same page, whose opening
    paragraph is `BINARY: LINE` as help wraps it, at spaces to 80
    columns (`skein/docs/design/runtime/shell.md`, 14.2), and which names
    no internal command: a row checks that paragraph, its lines joined
    by single spaces, never the page's first line alone;
  - `--version` prints `BINARY VERSION (COMMIT)`, the commit twelve
    hexadecimal digits, with `-dirty` or not, or `unknown`;
  - each command of words answers `--help` with a page whose opening
    paragraph, wrapped and joined as above, is `internal: not part of
    BINARY's public command line; it may change without notice.`;
  - a command the table lacks is refused, exit 2: with the front's words
    where the table has no bare command, and otherwise by its bare
    command, with `TABLE.refusal`'s words, as is a flag the bare command
    doesn't declare.

  None starts a browser, a forge or a box, and none writes outside its
  scratch directory.
- **The tools' own tests** keep what they check, their expected words
  following 4.2, in their crate or their binary's package (section 5).
- **Slower tiers:** a browser or process story a move touches is changed
  and compiled, not run, while the plan builds.

## 7. Later

- Skein's round owns `SKEIN_TEST_CHROMIUM`'s name, and C5 owns the
  journeys' build of the recorder.
