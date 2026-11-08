+++
id = "box-side"
part = "box-side"
kind = "feature"
needs = []
files = ["temper/crates/temper/src/link_file.rs", "temper/crates/temper/src/lib.rs", "temper/crates/temper/src/host.rs", "temper/crates/temper/src/box_command.rs", "temper/crates/temper/src/tests.rs", "temper/crates/temper/src/tests/link_file.rs", "temper/crates/temper-box/src/**", "temper/crates/temper-box/tests/real_loop.rs", "temper/crates/temper/tests/fake_backends/box_link.rs", "temper/crates/temper/tests/suite/box_listeners.rs", "temper/crates/temper/tests/process.rs", "temper/docs/help/box.md", "temper/docs/design/client/README.status.md", "temper/docs/design/protocol/setup.status.md"]
effort = "high"
minutes = 180
+++

# tui-link-colours: the engine keeps its live link at rest; the box names the TUI's command

## Goal

While the printed link is enabled, `temper serve` keeps its live printed link in a
private file, rewritten at each new link and removed at exit, so that a TUI on the same
machine signs in with nothing to paste (the TUI's side is later tasks'). The box's first
sign-in block gains one line naming the TUI's command at the link's origin.

Acceptance:

- `temper/crates/temper/src/link_file.rs`, `pub mod link_file;` in `lib.rs`, as
  `temper/docs/plan/tui-link-colours/notes/contracts.md`, section 6, spells it:
  - `path(data, runtime, linux)`: Linux, `RUNTIME/temper/printed-link-KEY` when `runtime`
    is an absolute path, else none; macOS, `DATA/printed-link`. KEY: the first 8 bytes of
    `skein_lib::sha256::digest(data)` as 16 lowercase hex digits, `data` the data
    directory's absolute path bytes as `Directories::data` holds them.
  - `deployment(environment, working)`: the data directory `temper box` with no flag
    resolves from that environment (`directories::resolve(&[], environment, working)`'s
    `data`), none when it refuses.
  - `keep(path, url, user)`: makes the parent `0o700` with its missing parents and refuses
    it unless it is this user's with no group or other bits (`directories::make_private`'s
    check), then writes `url` and `\n` whole, `0o600`, a synced temporary in the same
    directory renamed over it. Its error is `PATH: WHY`, never the URL.
  - `forget(path)`: removes it; absent is no error, and nothing is said.
  - `read(path, user)`: `skein_shell::read_private_file(path, MOST, user)`, the bytes
    without one final `\n`, none for any refusal.
- Serve (`temper/crates/temper/src/lib.rs`'s `serve`, `host.rs`'s drain), as
  `temper/docs/design/client/README.md`, 6.12, *The engine's link, kept at rest*:
  - the path from serve's `Directories::data`, `XDG_RUNTIME_DIR` from its environment
    and the platform (`cfg!(target_os = "linux")`), once at start, kept by the host;
  - under `--box` with a given first secret, `keep` its URL once the service is prepared
    and before `drive`, the URL made as `box_command::first_link` makes it
    (`setup::check(..).named`, `address(&named.clients)`, `named.public`,
    `temper_service::clients::first_link_url`), printing nothing;
  - at each link the drain prints, `keep` it after it is written to stdout;
  - after `drive` returns, by any exit it gives, `forget`;
  - a `keep` that fails is one line on stderr, `temper serve: link file PATH: WHY\n`, and
    serve goes on; with no path, nothing is kept and nothing is said.
- The box's block (`temper-box`, `terminal.rs`), as `temper/docs/design/protocol/setup.md`,
  7.2, *The sign-in block*: the first block's footer ends with `In a terminal: WORD
  http://HOST:PORT\n`, WORD the box's given word for this binary (`box_command.rs`
  passes `printed::running_word(&skein_shell::environment())` in the box's settings),
  HOST:PORT the first link URL's origin, cut before its path. A fresh link prints as
  now. The item's bound: where WORD and the origin would pass the box's fixed or line
  bound, the word is `temper`; say which bound you used.
- Rows and stories, each a failing test first where it pins a refusal:
  - `temper/crates/temper/src/tests/link_file.rs` (new, `mod link_file;` in
    `src/tests.rs`): `path`'s table (Linux with an absolute runtime, a relative one and
    none; macOS; two data directories giving two keys; the KEY of one known path
    spelled out); `deployment`'s table (`TEMPER_DATA` absolute and relative to the
    working directory, `XDG_DATA_HOME`, `HOME` alone, none) each equal to what
    `directories::resolve(&[], ..)` gives the box, so the engine's and the TUI's paths
    agree; `keep` on a scratch directory writes `0o600` in a `0o700` parent and a second
    `keep` replaces the first whole; `keep` refuses a group-writable parent with an error
    holding no URL; `forget` removes and `forget` again is quiet; `read` gives the bytes
    without the line feed, and none for a group-readable file and for one past `MOST`.
  - `temper/crates/temper/tests/suite/box_listeners.rs`,
    `a_fresh_box_prints_its_block_and_nothing_else`: the block's stderr is the prelude,
    the footer and the new line, its origin the link's; while the box runs, its runtime
    directory's `temper/printed-link-KEY` holds the printed link and `\n`, `0o600` in a
    `0o700` directory; after `stop`, it is gone.
    `the_first_link_opened_at_once_waits_and_signs_in`: after its sign-in, opening the
    spent link once more is answered used, the box prints a fresh link, and the file
    holds the fresh one; if the fixture can't read a second printed link, write that half
    as a process-tier story beside the box's in `tests/process.rs`, compiled not run, and
    say so.
  - `temper/crates/temper/tests/fake_backends/box_link.rs`: `Boxed::with_command` and
    `command` give the box `XDG_RUNTIME_DIR` as the scratch's `runtime` directory (made
    `0o700`), so that no story writes the runner's own; `FOOTER` gains the line, or a
    function of the link gives it.
  - `temper-box`'s own tests (`src/tests/host.rs` near its footer bytes,
    `tests/real_loop.rs`'s `FOOTER`): the line, as the block's other bytes are checked.
  - `temper/crates/temper/tests/process.rs`, its block's lines: the new line; compiled,
    not run.
- `temper/docs/help/box.md`'s expected block gains `In a terminal: temper
  http://127.0.0.1:PORT`, the page within its 12,288 bytes.
- Status: delete `temper/docs/design/client/README.status.md`'s **Not yet** "6.12, *The
  engine's link, kept at rest*: ... are not built." and
  `temper/docs/design/protocol/setup.status.md`'s **Not yet** "7.2, *The sign-in block*:
  ... is not printed.", nothing else (a stated departure from the implementer's text).

## Inputs

- `temper/docs/design/client/README.md`, 6.12, *The engine's link, kept at rest* (all of
  it), and section 7, *The printed link*.
- `temper/docs/design/protocol/setup.md`, 7.2, *The sign-in block* and the fresh link's
  paragraph after it.
- `jig/docs/design/protocol/client.md`, section 8, *The printed link*: a fresh link is
  drawn when a spent or expired one is tried and none is live.
- `temper/docs/plan/tui-link-colours/notes/common.md`;
  `temper/docs/plan/tui-link-colours/notes/contracts.md`, sections 6 and 8.

## Owns and may change

- Owns the files in `files` for this wave. `temper/crates/temper/src/tui.rs` and its
  tests are the task `binary-tui`'s, later: leave them.
- `temper-service` is not this task's: the URL is made in the binary from the parsed
  configuration, as the box makes it. If serve's binary can't reach the configuration's
  `named` without parsing it twice, parse it twice (it is small) and say so.
- `temper_box::host::Settings` (or the Formatter's constructor) gains the word: its
  literals in `temper-box`'s tests and in `box_command.rs` are compiler-forced edits.
- Other spawns of `temper` in `temper/crates/temper/tests/` that don't clear their
  environment inherit the runner's `XDG_RUNTIME_DIR`: leave them; serve removes its file
  at exit. Name in the report each story that kills a serve without its exit.
- The help page's bound bites: condense another sentence of `box.md`, losing no fact.

## Steps

1. The rows; `heavy cargo nextest run -p temper --lib -E 'test(/link_file/)'`; see them
   fail. Then the box stories: `heavy cargo nextest run -p temper --test suite -E
   'test(/box_listeners/)'`.
2. `link_file`; serve's keeping and forgetting; the box's line.
3. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper
   -p temper-box --all-targets -- -D warnings`; `heavy cargo nextest run -p temper-box`;
   `heavy cargo nextest run -p temper --lib`; `heavy cargo nextest run -p temper --test
   suite -E 'test(/box_/)'`; `heavy cargo check -p temper --tests` (the process tier
   compiles); `heavy cargo nextest run -p temper-docs` (the help page).
4. Commit, push.

## Report

As the implementer's text says; the bound chosen for the line, the stories that kill a
serve, and each row's failure seen.

## Rebase (2026-11-08, step merge, foreman tui-link-colours/box-side/foreman-1)

resolve the status-file conflict by preserving both completed-feature deletions
Rebase onto the current integration tip.
The sole conflicting path is `temper/docs/design/client/README.status.md`.
Commit `4bcf13170b` (merged by `0920482a03`) removes the 6.12/6.14 `temper-client-tui` sign-in progress gap.
This branch’s `1ed3cc4783` removes the adjacent 6.12 “The engine’s link, kept at rest” gap.
Resolve the conflict with both entries absent; preserve every other status entry.
Keep the branch’s deletion in `temper/docs/design/protocol/setup.status.md`.
No code change is indicated by this merge refusal.
