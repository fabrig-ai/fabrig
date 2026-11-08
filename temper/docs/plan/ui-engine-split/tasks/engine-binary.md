+++
id = "engine-binary"
part = "engine-binary"
kind = "feature"
needs = ["engine-serve", "version-crate", "jig-levels", "engine-limits"]
files = ["temper/crates/temper-engine/**", "temper/docs/design/protocol/README.status.md", "temper/docs/design/protocol/setup.status.md", "temper/docs/design/README.status.md", "temper/docs/design/client/browser.status.md"]
effort = "high"
minutes = 120
+++

## Goal

The engine's own binary, `temper-engine`: `serve` for an engine alone's
document, the owned forge's commands, `versions`, `help`, `internal
forge-child` and `--version`, refusing a worker's or a standalone
document by naming the binary that runs it, with the browser's bundle
embedded (the request's second item;
`temper/docs/design/protocol/README.md`, 4.1 and 4.2). Its commands'
code is `temper-engine-shell`'s and `temper-shell`'s; this package holds
its table and `main`. `upgrade` is not in it: its jig mechanisms aren't
built (4.2's status entry stays).

Acceptance:

- `temper/crates/temper-engine`: a binary package on the `binary` lint
  profile with the `ordinary` `clippy.toml` copy, `[[bin]] name =
  "temper-engine"`, no library (`notes/contracts.md`, 7). It depends on
  `temper-engine-shell`, `temper-shell`, `temper-version`,
  `temper-service` by the workspace's entry with no feature,
  `jig-protocol-store`, `temper-protocol-store`, `skein-shell`,
  `skein-fail-stop` and what `main` names besides; never on `temper`.
- **`main`** calls `skein_fail_stop::install()` first, then: `--version`
  alone prints `temper_version::line("temper-engine")` and a newline,
  exit 0, before the table; `--help` alone prints `ENGINE.help()`;
  otherwise the table's row (`ENGINE.find`), `COMMAND --help`
  (`ENGINE.row`), and the row's command; no row is `ENGINE.unknown`'s
  line on stderr, exit 1.
- **Its table**, `ENGINE: Table` of `temper-shell`: `HELP`; its own
  `serve` row, words `["serve"]`, `Who::Deployment`, forms `[DIRECTORIES]`,
  its line and notes from the box's `serve` row but saying it serves an
  engine alone's document and refuses the others, naming the binary that
  runs them (4.2); a `versions` row, `Who::Deployment`, no arguments, its
  line "Print this binary's patch levels, its store's and families'
  versions, and its wires, one line each." and a note that it reads no
  configuration and no store; then `temper-engine-shell`'s `FETCH`,
  `PREPARE`, `RUN`, `SIGN_IN`, `SOURCE`, `REMOVE` and `FORGE_CHILD`.
- **The commands:**
  - `help` is `temper_shell::help`'s request and answer, as the box's.
  - `serve [DIRECTORIES]` resolves the directories, reads the document,
    and refuses by `temper_service::startup::nearest` (4.2's two lines,
    exactly, on stderr, exit 1, `COMMAND` `serve`) before calling
    `temper_engine_shell::serve::run("temper-engine", …, false, None)`;
    a document `nearest` can't read goes on to `run`, which refuses it in
    the checks' words.
  - The forge rows: `temper_engine_shell::forge::configure`, then, where
    `forge::reads_document(row)`, the same refusal by `nearest` (its
    `COMMAND` the row's words, `forge fetch`), then
    `forge::command("temper-engine", …)`. `internal forge-child` the same,
    with no document.
  - `versions` prints 4.2's lines, in its order: `level skein|smith|jig|temper
    LEVEL` from `jig_protocol_store::levels` of the patch sets
    `temper-service` hands its store (none today, `PatchSets::NONE`; use
    the service's constant if one exists), `none` for `None`; `store N`
    and `family NAME N` from `jig_protocol_store::format::current(temper_protocol_store::STORE_VERSION,
    [0; 16])`'s fields (`component`; `core`, `tasks`, `people`, `notes`,
    `accounts`, `settings`; `temper` for `application`); `wire NAME N`
    for each of `temper_service::store::WIRES`. Exit 0.
- **The step tests**, `tests/suite/main.rs`, one test binary, one table
  test running the built binary (`skein_scratch::paths::binary("temper-engine")`),
  a row each: `(arguments, the document written as CONFIG/temper.json or
  none, exit, a check of stdout, a check of stderr)`:
  - `--version`: exit 0, stdout `temper-engine 0.0.0 (…)\n` (the
    workspace's version, a COMMIT of twelve hex digits, `-dirty` or not,
    or `unknown`);
  - `--help`: exit 0, stdout's first line `usage: temper-engine COMMAND
    [ARGUMENT...]`, a row for each of `serve`, `forge fetch` and
    `versions`, and none whose words are `box` or `program-login` (the
    words, not the text: `forge remove`'s line says "box");
  - `serve --help`: exit 0, `serve`'s usage under `temper-engine`;
  - `box`: exit 1, stderr `temper-engine: no command box; temper-engine
    --help lists them\n`;
  - `program-login a`: exit 1, the same line for `program-login`;
  - `serve --config DIR --data DIR --cache DIR` on a worker alone's
    document (`{"workers":[{"engine":{"socket":"127.0.0.1:1"}}]}` or the
    smallest `nearest` reads as one): exit 1, stderr exactly
    `temper-engine serve: temper.json holds a worker alone; temper-worker serve runs it\n`;
  - the same on a standalone document (an `engine` whose
    `hosts.listener` names `{"in_process": "workers"}`): `… holds an
    engine and its worker in one process; the box's temper serve runs it\n`;
  - `forge fetch … 1` on the worker alone's document: `temper-engine
    forge fetch: temper.json holds a worker alone; temper-worker serve runs it\n`;
  - `versions`: exit 0, first line `level skein none`, a line `family
    temper N` with `temper_protocol_store::STORE_VERSION`, and a `wire
    temper-payload` line.

  For each refusal, break it once (let the shape pass), see its row
  fail, restore.
- The status entries close, each its lines alone: in
  `temper/docs/design/protocol/README.status.md` the **Not yet** 4.2
  entry "`crates/temper-engine` doesn't exist: …"; in
  `temper/docs/design/protocol/setup.status.md` the **Not yet** 7.4, 11
  entry "`temper-engine` doesn't exist: …"; in
  `temper/docs/design/README.status.md` the **Not yet** 1, 9 entry, if
  every crate it names now exists; in
  `temper/docs/design/client/browser.status.md` the **Not yet** 2.6
  entry. The **Not yet** 4.2 `upgrade` entry stays. This departs from
  the implementer's text, which closes `docs/design/` to you: these
  entries are this task's.

## Inputs

- `temper/docs/design/protocol/README.md`, section 4 (the rows' shape,
  `temper serve`), 4.1, 4.2, 15.1 and section 9 (`STORE_VERSION`).
- `temper/docs/design/release.md`, section 3.
- `jig/docs/design/protocol/store.md`, 5.3 (the levels and the format's
  versions).
- `temper/docs/plan/ui-engine-split/notes/contracts.md`, 4 to 7.
- `temper/docs/plan/ui-engine-split/notes/common.md`.

## Owns and may change

- Owns the files in `files`.
- The root `Cargo.toml`: the member line `"temper/crates/temper-engine",`
  right after `"temper/crates/temper-engine-smith",`; no dependency line
  (nothing depends on it). `Cargo.lock` as cargo writes it.
- Pre-approved:
  - If `temper-shell`'s `Table` or `temper-engine-shell`'s `forge` lack a
    function `main` needs (a `Row`'s `serve` notes constant, a
    `pub` on an item), add it there, its one line, and name it.
  - If `nearest` or `WIRES` is not `pub` on your base, make it `pub`, its
    one line, and name it.
  - If the fail-stop or lints boundary tests read the new package, they
    pass as they are once `main` installs the hook first and the profile
    is set; if one needs the package listed, list it.
- Not yours: the box's `temper` and its table; `docs/help/` (the
  `commands` page is the box's table's; naming each command's binary is
  M5's); the boundary test of the closure (`engine-closure`).

## Steps

1. The manifest, `python3 -I -B tools/lints/lints.py --set binary
   temper/crates/temper-engine`, the `ordinary` `clippy.toml` copy.
2. The suite's table first, against a `main` that prints nothing; see
   its rows fail.
3. `main`, the table and the commands; `heavy cargo nextest run -p
   temper-engine`; read the count. Break each refusal and restore.
4. `cargo tree -p temper-engine -e normal --locked --offline | rg
   'temper-worker|smith-agent|smith-host'`: report what it prints (the
   closure test is `engine-closure`'s; this is the report's).
5. The status entries.
6. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `python3 -I
   -B tools/lints/lints.py`; `heavy cargo clippy -p temper-engine
   --all-targets -- -D warnings`; `heavy cargo nextest run -p
   temper-engine -p fabrig-boundary`.
7. `python3 -I -B tools/citations/citations.py --base origin/main`.
   Commit and push.

## Report

As the implementer's text says, with `--version`'s and `versions`'
output from your build, and step 4's output.
