+++
id = "claude-fetch"
part = "fetched"
kind = "feature"
needs = ["always-offered"]
files = ["temper/crates/temper-forge-setup/src/fetch.rs", "temper/crates/temper-forge-setup/src/claude_code.rs", "temper/crates/temper-forge-setup/src/lib.rs", "temper/crates/temper-forge-setup/src/tests/fetch.rs", "temper/crates/temper-forge-setup/src/tests/claude_code.rs", "temper/crates/temper-forge-setup-shell/src/fetch.rs", "temper/crates/temper-forge-setup-shell/tests/suite/fetch.rs", "temper/crates/temper/src/claude_code.rs", "temper/crates/temper/src/main.rs", "temper/crates/temper/src/lib.rs", "temper/crates/temper/src/program_login.rs", "temper/crates/temper/src/worker_alone.rs", "temper/crates/temper/tests/suite/startup.rs"]
effort = "high"
minutes = 90
+++

## Goal

Choosing Claude works on a machine with no `claude`. The first command of the source, its sign-in, its status or a run, finds temper's pinned build missing, fetches it into the cache and goes on. A `"fetched"` program's every command passes through a launcher, a `temper` run by the name `claude`, as the playground's git is; smith and jig do not change. A machine that never chooses Claude downloads nothing.

Acceptance:

- `temper_forge_setup::claude_code` holds the pin of `setup.md`, 6.6, all six platforms, and a test holds each digest and size against that table's text.
- At the host, with a fake download: a missing build fetched once, then found with no request; a wrong digest refused, nothing left under its name and no part file; two commands at once make one request; a file of another size refused and left as it was.
- `temper program-login NAME` on a fresh init with no `claude` on `PATH` reaches a build placed in the cache, through the launcher, its arguments unchanged and `DISABLE_AUTOUPDATER=1` in its environment. No test reaches the network.
- `temper serve`, a worker alone and `temper program-login` write the launcher's two links when the program's path is the launcher's, and write nothing when it is a found `claude`.
- Forgejo's fetch is unchanged, its lines and refusals word for word.

## Inputs

- `temper/docs/plan/claude-always/notes/common.md`
- `temper/docs/plan/claude-always/notes/contracts.md`: "`"fetched"`", "The launcher's two links", "The pin and its host"
- `temper/docs/design/draft/claude-provider.md`, 3
- `temper/docs/design/protocol/setup.md`, 6.6, the whole section: its first part is what is shared, "Claude Code" what differs
- `smith/docs/design/protocol/programs.md`, 4.2

## Owns and may change

**Owns:** `temper/crates/temper-forge-setup/src/fetch.rs`, a new `claude_code.rs` and its tests; `temper/crates/temper-forge-setup-shell/src/fetch.rs` and `tests/suite/fetch.rs`; a new `temper/crates/temper/src/claude_code.rs`; `temper/crates/temper/src/main.rs`; the program blocks in `temper/crates/temper/src/lib.rs` (around line 463), `worker_alone.rs` and `program_login.rs`.

**Readers you may change, each only as said:**

- `Pin` and `Fetched` literals in `temper/crates/temper-forge-setup/src/tests/fetch.rs`, `temper/crates/temper-forge-setup-shell/tests/suite/fetch.rs` and `temper/crates/temper-box/src/**`: the field's new type alone.
- `lib.rs` and `tests.rs` of each crate: the module line.
- `temper/crates/temper/tests/suite/startup.rs`, and a helper in `temper/crates/temper/tests/support/`: the new story.

**Pre-approved options:**

- `Fetched`'s `read` stays forge-only: a box never relays Claude Code's lines. If that needs a change, keep `read` refusing `claude-code` and say so.
- If `File::lock`'s blocking form is not on this toolchain, use `process::lock` in a bounded retry at 100 ms and say so.
- `Failure::Bytes` says `setup.fetch_bytes`, which is untrue here. Add a variant with its own words for Claude Code; Forgejo's stay.
- If a status or run needs a second look at `argv[0]`: spawned programs get their path there (`skein-shell/src/linux/process.rs`, 216), and `exec` by path does the same. If you find a way that does not, stop and ask.

## Steps

1. **The machine.** In `fetch.rs`, `Pin.binary` and `Fetched.binary` take a new `pub enum Named { Forge(Binary), ClaudeCode }`; `word()` answers `claude-code` for it. Nothing else of the machine changes, but for the `Bytes` option above.

   New `claude_code.rs`, as the note has it, step code. Its digests are `[u8; 32]` literals, and a test compares `fetch::hex` of each to the text in `setup.md`, 6.6. `build` answers `Failure::Architecture` for another `machine`: `x86_64` is `x64`, `aarch64` is `arm64`, `Macos` is `darwin-*` with no musl form, `Musl` adds `-musl`. `pin`'s `path` is `/claude-code-releases/VERSION/PLATFORM/claude`, its origin the caller's, `bottle` none. Tests: every row; `fetch::url` of a pin is the design's URL; `file` under a cache with and without a trailing `/`.

2. **The host.** `temper_forge_setup_shell::fetch::claude_code`, on `run`:

   - a file at `bytes` answers at once, holding no lock; a file of another size is `Failed::Refused(Failure::Present { file })`;
   - otherwise make the parent private, as `open` does, take an exclusive blocking lock on `<file>.lock`, and look again;
   - still missing: `fetching Claude Code\n` to `out`, then `run` with `Fetch::new([pin], bytes)`.

   A failed line to `out` fails nothing. Tests in `tests/suite/fetch.rs`, on its local TLS `server`, with a `Pin` of a small body and its digest: the four host rows of the acceptance. The two-at-once row runs two threads, the server counting requests.

3. **The launcher.** `temper/crates/temper/src/claude_code.rs`:

   ```rust
   //! Claude Code's launcher (protocol/setup.md, 6.6).
   pub fn is_launcher(program: &OsStr) -> bool;          // the file name `claude`, as `fake::git::is_git`
   pub fn run(program: &OsStr, arguments: &[Box<[u8]>]) -> u8;
   pub(crate) fn place(directory: &Path, cache: &Path) -> Result<(), String>;
   ```

   `main.rs` calls it beside `is_git`. `run` reads `cache` beside `program`, builds the pin (`Platform` as `forge.rs`'s `fetch` chooses it, `std::env::consts::ARCH`, `cfg!(target_env = "musl")`, origin `downloads.claude.ai:443`, trust `Trust::Machine` with `SSL_CERT_FILE`, limits as `forge.rs`), calls the host with stderr as `out`, sets `DISABLE_AUTOUPDATER=1`, and `exec`s `claude_code::file`, the build, with its arguments and the environment it was given. On a refusal it says `temper: claude: WHY` on stderr and exits 1. Stdout is the command's: nothing of the launcher's reaches it.

   `place` makes the launcher's directory private and writes both links, replacing one that points elsewhere by a new link and a rename, leaving one that is right untouched, so two starts at once are harmless. `claude` targets `std::env::current_exe()`.

   Call sites: `lib.rs`'s block at `agent.programs.first()`, `worker_alone.rs`'s, and `program_login::command` ahead of `sign_in`. Each calls `place` only when `program.path` equals `agent::launcher(program.directory)`, and a refusal stops the start, naming `programs.claude_code`. `temper-service`, smith and jig do not change.

4. **The story.** `temper/crates/temper/tests/suite/startup.rs`, `a_fetched_program_runs_through_its_launcher`: `temper init --box` on an empty environment, then a build placed at `<cache>/claude-code/2.1.311/claude`: a small shell script recording its arguments and its `DISABLE_AUTOUPDATER`, grown with `File::set_len` to this machine's pin size, a hole past its `exit`. Run `temper program-login some-name` with its flags and read what it recorded: `auth login`'s arguments as `smith_agent_shell::program::sign_in` gives them, the variable set. Then run the link itself with an empty environment and two odd arguments, as a worker's spawn does, and see them unchanged and stdout the script's alone. Check nothing was fetched: no `.part`, no `fetching` line. If the machine has no pin (another architecture), the test says so and passes nothing silently: `#[cfg]` it to the pinned ones.

   A test that the launcher's two links are written by `temper program-login`, and that a found `claude` writes neither: `temper/crates/temper/tests/support/program_login.rs`'s `configuration` is the shape.

5. `cargo fmt`, `python3 -I -B temper/scripts/old_forms.py`, `heavy cargo clippy -p temper-forge-setup -p temper-forge-setup-shell -p temper-box -p temper --all-targets -- -D warnings`, then `heavy cargo nextest run -p temper-forge-setup -p temper-forge-setup-shell -p temper-box`, `-p temper --lib`, `-p temper --test suite`. Run `heavy cargo nextest run -p temper --test process -E 'test(temper_box) | test(forge_fetch)'`. Commit, push.

## Report

As your text says, and: how long a launcher adds to a command on your machine; what you chose for `Fetched::read`; any command of the source that did not pass through the launcher.

## Answers

2026-11-05 (fabrig/ui). Allow it: add the Claude fetch's module to the boundary test's declared modules, for just the lock and the launcher exec, with a comment saying why; the test keeps its purpose, naming where these operations live. Edit that test in this task. Then recheck on the merged head.
