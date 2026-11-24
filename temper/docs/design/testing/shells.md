# The shells' checks

Provisional, 2026-11-24. Where the checks of temper's shell crates run: forge setup's host, the two fetches, the box shell, the common shell, the TUI's start, the service shell and the engine shell. Which test binary holds each kind, the facts a check that replays is handed at the shell's boundary, and what stays in the slow binaries and why. Written from `../draft/slow-tests-down.md`, 3, 4 and 5.3, for `temper-forge-setup-shell::suite`, `temper-box-shell::real`, `temper-shell::real`, `temper-client-shell-tui::real`, `temper-service-shell::real`, `temper-fetch-shell::real` and `temper-engine-shell::suite`. The rule a check that replays keeps is `tools/tiers/layers.md`, 6.1; the box binary's own checks are `box/README.md`'s. Each test that left a slow binary is listed beside its holders in `../../development/slow-tests-moves-2026-11-24-temper-shells.md`.

## 1. In one page

- **The homes below the slow binaries** (section 2): each shell's library tests, step tests by rule, for its decisions over supplied facts; `temper-forge-setup-shell::step` for preparation, private files, locks and commands' captures; `temper-one-process-world::suite` for the engine's stopped rehearsal and its going back, on the simulated disk.
- **Facts at the shell's boundary** (section 3): a check that replays is handed a file's metadata and bytes, the results of each file effect in order, a lock's verdict, a command's completion and output, a request's head and an answer's events, a child's output and end, the terminal's verdict and the environment's rows, never the machine's. Each seam is a companion of the function that reads the machine today, taking the fact where that function made the call; the function becomes the companion with today's call as its argument, so the shell keeps its behaviour (`box/README.md`, 3).
- **No new fake.** Where a check needs file state (kinds, modes, owners, links, renames, removals, locks held through a handle) it is skein's fake machine's (`skein/crates/skein-fake-machine`); where it needs order, the effects are recorded with their results. The fake machine has no mode change, no link creation and no inode number: a check records the mode asked, plants links as fixtures, and tells a new node from the old by a handle kept on the old one.
- **What stays above** (section 4): TLS, a lock between handles or processes, a descriptor transferred, a real child's status, streams and teardown, a real shell, ports contended for on the kernel, the terminal, native key generation and validation, and one native witness of each moved table whose path is a real one. `temper-shell::real`, `temper-client-shell-tui::real` and `temper-service-shell::real` keep nothing and go.

## 2. Where each check lives

### 2.1 Forge setup's host: `temper-forge-setup-shell`

- **Its step binary,** `temper-forge-setup-shell::step`: a private file's publication, a rename and the forge directory's layout and clearing over the fake machine (3.1); a lock's acquisition (3.2); a command's capture driven over scripted kernel turns (3.3); and preparation through `prepare::run_with` over its host, the host's files the fake machine's through the same publication, its commands' captures recording each registration and modelling the runner's row.
- **Its library tests:** a bottle's extraction verdict (3.5).

### 2.2 The fetches: `temper-fetch-shell`

- **The pinned fetch's host** (`fetch`), in its library tests: the cache's effects and the published file (3.4), each request's head and each answer's events (3.4), the progress lines at the writer, a writer that refuses, and Claude Code's cache, a miss checked again under its lock and a hit asking metadata alone. The forge's bottle runs here with its extraction's verdict supplied; the verdict itself is forge setup's (2.1).
- **The box binary's host** (`box_binary`), in its library tests: a candidate's identity over its facts (3.6), and a staged build of another identity refused under the lock, nothing published.

### 2.3 The box shell: `temper-box-shell`

Its library tests: the fake's retained outputs over its `Disk` face (`box/README.md`, 3.9) with distinct node identities; the fake backends' sandbox, fresh and reused (3.7); init's documents, ports, programs and key over its deployment's io (3.8); the link's form (3.9); and the upgrade's standalone preflight and its handoff to the engine (3.10).

### 2.4 The common shell: `temper-shell`

Its library tests: a private root made and checked (3.11), the link file kept, forgotten and read (3.12), and a program's lookup (3.13).

### 2.5 The TUI's start: `temper-client-shell-tui`

Its library tests: an origin's directories through `start::deployment::prepare_with` over its fake files, each made private and checked by the common shell's verdict, `temper_shell::directories::private_root_with`, over the fake's metadata, never a verdict of the fake's own.

### 2.6 The service shell: `temper-service-shell`

Its library tests: a named file's read (`box/README.md`, 3.1), the push key's among them, and the provider copy's write (3.14).

### 2.7 The engine shell and the one-process world

- **`temper-engine-shell`'s library tests:** the stopped preflight's held lines (3.15); the rehearsal's scratch, swept, removed and removed again at the rehearsal's end (3.16); a forge's setup file over `forge::setup_file_with`'s facts and the setup's bounded bytes; the checkpoint's and the restore's terminal rows; the program environment's widest answer at its wire bound.
- **`temper-one-process-world::suite`:** a configured floor's rehearsal reaching its report, already there; a source held by another refusing a rehearsal after its scratch is made, through the store-first opening; a configured floor with no previous generation refused going back, with its report and exit.

## 3. Facts at the shell's boundary

### 3.1 A private file's publication and the forge's layout

- **The facts:** each effect's result, in order: the exclusive creation of a sibling at `0600`, its write, its sync, the rename over the name and the parent's sync; for a rename, the source's open and sync, the rename and the destination parent's sync; a path's no-follow presence; for the layout and its clearing, each directory's creation and mode, and each file's or tree's removal.
- **Production:** `files::write_private`, `files::rename`, `files::exists`, `files::layout`, `files::present` and `files::clear` make each call as today.
- **What it carries:** the bytes read back whole, the mode asked, the old node replaced and not written in place, one entry left in the directory; clearing's six entries removed and `state` and `backups` never; a failure before the rename leaving the old file, apart from one after it.

### 3.2 A lock's acquisition

- **The facts:** the open's result for the named path, and the non-blocking lock's verdict on that same opened handle: acquired, held, or an error.
- **Production:** `process::lock` and `process::lock_file`, as today; the handle returned is the one locked, and `process::command` still passes it on as a child's stdin.
- **Below:** the fake machine's locks, held through a handle until it closes, so a second acquisition is refused as held, a rename between the open and the lock leaves the renamed node held and the old name lockable apart, and nothing is opened again by path.

### 3.3 A command's capture

- **The facts:** the kernel's and the clock's turns that `command::Running` is driven by, through skein's `drive_with` (`skein/docs/design/runtime/shell.md`, 11): the spawn's completion or its refusal, each pipe's reads and ends, the child's exit.
- **Production:** `command::run_one` and `command::run_until` open the root and the signals and drive the real ring, as today.
- **What it carries:** each stream cut at its bound while the command is read to its end, the exit's code, a refused spawn as `command::Error::Spawn`, never a status.
- **What it doesn't:** a real child's status and streams, and a deadline's teardown of a live child's group (section 4).

### 3.4 A pinned download's cache and wire

- **The cache's facts:** the cached file's bytes, absent, or an error; each effect's result: the part file's creation with its truncation and mode, a write at its offset, the mode `0755`, the sync, the rename to the final name, the parent's sync, a removal.
- **The wire's facts:** each `Get` the machine asks for becomes a request head built apart from its connection, checked as bytes: its method and path with the query kept, `Host` with its port, `User-Agent: temper`, `Connection: close`, and the registry's token on the first request alone. Each answer is projected into the machine's events from a scripted plaintext response: a status with its `Location`, a declared length or a chunked body, its pieces and its end.
- **Production:** `Download` answers each request over the cache and `Connection` over TLS, as today; `Connection`'s use by the box binary's fetch keeps its shape.
- **What it carries:** a digest refused naming the URL and both digests; a cached file matching, nothing asked, and one differing, refused and kept; a stale part file truncated; a status refused naming the URL; a body past its bound; the bottle's redirects, its archive's and its member's digests and bounds and its extraction's refusals, every temporary file removed; the progress lines' exact bytes at the writer, in order, and a refusing writer failing no fetch; Claude Code's miss, metadata, prepare, lock, metadata again and download, in that order, and its hit or wrong size asking metadata alone.
- **What it doesn't:** the TLS handshake, a refused root, an authorized redirect to a second origin over TLS, and two commands contending for one cache (section 4).

### 3.5 A bottle's extraction

- **The facts:** the member's bytes as the child's stdout gives them, in pieces, and the child's end.
- **Production:** `process::extract` runs `/usr/bin/tar -xzOf ARCHIVE MEMBER` and pipes its stdout, as today. The copy is bounded and hashed as it is written, stopped at the bound with the child killed and waited for, and a digest is returned only after a clean exit with at least one byte.
- **What it carries:** a member past its bound refused as its size, a missing member, an empty member and a corrupt archive refused as extraction, the digest taken from the bytes and never supplied.

### 3.6 A candidate's run

- **The facts:** the candidate's metadata (absent, its kind and its mode) and, when it qualifies, its run's raw stdout and whether it exited 0.
- **Production:** `box_binary::executable` and `box_binary::identity` read the metadata and run the candidate with `--version`, an empty `HOME` and its 10-second cutoff, as today.
- **What it carries:** stdout kept whole to 256 bytes and a candidate past it failed; a failed exit keeping its stdout; a non-executable file, a directory or a missing path never run.

### 3.7 The fake backends' sandbox

- **The facts:** for each path the sandbox reads or keeps: its no-follow metadata (kind, mode, links, a node identity), its bounded bytes, and the results of its exclusive creation with its mode, write, sync, rename and removal; the ports a reservation was given, as values, and a reservation's bind result; for the retained fake adapter, its record's metadata, the prepared marker's presence and the sibling program's metadata.
- **Production:** `fake_backends::prepare`, `replace_bundle`, `open_staging`, `scripts_for_start_with`, `configured_claude`, `reserve_from` and `reserve_chosen` make each call as today. The sandbox's lock and its listeners' descriptors stay production's (section 4).
- **What it carries:** fresh against reused preparation; the given scripts joined per start with the model's fallback kept or replaced; a reservation that loses its port taking another once; the bundle replaced by staging, write, sync and rename with every other edit kept, the same bundle written nowhere, a refused replacement or retained document leaving the document as it was; the fake adapter's record kept outside the workers' configuration and its absence named; the GitHub App's key published from the fake's document; the named ports mapped and kept on restart.

### 3.8 Init's facts

- **The facts:** init's deployment io as it is today (`init::write_deployment_with` over `DeploymentIo`): each bind's and listen's result, the kernel's fallback ports, each lookup's metadata, the provider key's presence with its identity and size, and each write's and link's result.
- **Production:** `init::write_deployment`, as today; no new seam.
- **What it carries:** for two deployments, which preferred ports each gets and which fall back; the effects' order and modes, the git link's target, and what the document names and leaves out; the retained file's `listen` alone or the webhooks' address written out; a bare document admitted as serve admits it, with its prospective agent `fetched` and preflighted; an existing provider key kept with no OpenSSL asked.

### 3.9 The link's form

- **The facts:** whether stdout is a terminal, and the environment's rows.
- **Production:** `box_command::link_form` asks stdout's terminal verdict as today and hands it on.
- **What it carries:** `TERM` taken from the rows, and `TERMCAP` or `HOME` never read as it; OSC 8 on a terminal unless `TERM` is `dumb`, bare otherwise.

### 3.10 The box's upgrade preflight and handoff

- **The facts:** the box's configuration read, its programs' and its agent's facts, the named files, the provider and setting copies and the seeds, as the box's serve takes them (`box/README.md`, 3.1, 3.4 and 3.7); and the engine's prepared run as a record of what it is handed.
- **Production:** `upgrade::prepare` reads and `upgrade::run` calls `temper_engine_shell::upgrade::run_prepared`, as today.
- **What it carries:** the standalone preflight making no write under an existing `--data`; each checkpoint request, `--check`'s and `--workers`'s, handed to the engine unchanged with the configured deployment. The checkpoint's lock against a serving deployment is the engine shell's (section 4).

### 3.11 A private root's making

- **The facts:** the final entry's metadata, the recursive creation's result with its mode, and the metadata read again after it.
- **Production:** `directories::make_private`, as today: created only where absent, then checked; its verdict and words are `directories::private_root_with`'s (`box/README.md`, 3.2).
- **What it carries:** an absent root and its parent made `0700`; a private one kept; a public one, a link at the root's own name and a file refused with the one line naming it; a link among the ancestors followed, the root made through it.

### 3.12 The link file

- **Keeping:** the parent's preparation as a private root (3.11), a seed for the temporary name, and the results of its exclusive creation at `0600`, write, sync, rename and removal. The URL and its LF are written whole, a replacement leaves one entry, and a parent refused leaves no file and names the target, never the URL.
- **Forgetting:** the removal's result; an absent file is quiet.
- **Reading:** the bounded private read's result for the target, the user and 4,096 bytes; any refusal is no link, and one final LF is stripped. The reader's own owner, mode, bound and end-of-file checks are skein's private reader's.
- **Production:** `link_file::keep`, `link_file::forget` and `link_file::read`, as today.

### 3.13 A program's lookup

- **The facts:** each candidate's metadata, following links: absent or an error, its kind and its mode.
- **Production:** `lookup::find` and `lookup::is_executable`, as today; the candidates are `lookup::candidates`'.
- **What it carries:** a missing candidate skipped for a later executable one; a file without an execute bit never found; empty and relative entries never candidates.

### 3.14 The provider copy

- **The facts:** the copy's previous bytes, or absent, and the results of the stale temporary's removal, the exclusive creation at `0600`, the write, the sync, the rename, the directory's sync and the cleanup.
- **Production:** `providers::write`, as today; its read back is `providers::read_using`'s.
- **What it carries:** an absent copy written at `0600`; equal bytes asking no effect at all; changed bytes replaced, no `.new` left.

### 3.15 The stopped preflight

- **The facts:** the configuration's read (`configuration::Read`), the held file's text, absent, or its error, and a record of every opening or adoption of a store.
- **Production:** `stopped::prepare` and `stopped::prepare_rehearsal` read as today; `serve::limits::parse_held` and the stock limits' resolution judge the text.
- **What it carries:** an unknown limit and an out-of-range figure each named by its line, with no recovery hint, and no store opened.

### 3.16 Rehearsal scratch

- **The sweep's facts:** the cache's entries, each one's no-follow kind and modification time, the wall time, and for a stale entry the open's and the non-blocking lock's result apart from the recursive removal's.
- **The end's facts:** the source's open result and the rehearsal's end, a report's exit or an error, with the scratch the run made.
- **Production:** `upgrade::scratch::sweep`, `upgrade::scratch::remove` and the upgrade's run, as today.
- **What it carries:** only a stale, unlocked directory at an `upgrade-` name of 16 lowercase hex digits removed; a held one, any other name, a file and a link left without a lock or a removal asked; a removal taking the whole tree, a hard link's outside file kept, before the lock is released; the run's own scratch removed after its report, on 0 and on 1.

## 4. What the slow binaries keep

Each stays because only the real kernel, a real program or a path through `main` shows it (`skein/docs/foundation/testing-strategy.md`, 2.8 and 2.9):

- **`temper-forge-setup-shell::suite`:** a pinned fetch over local TLS with its trusted root, and refused under another root; a bottle's two TLS hops, the token on the first alone, extracted by the real `tar`; two Claude Code commands at once making one request; a real child's status and both streams; a source command's deadline closing a live child; a child holding the lock as its stdin after the parent lets go.
- **`temper-fetch-shell::real`:** one real candidate's run, its argument and empty `HOME`; one whole publication under the native cache lock, with its modes, renames and read back.
- **`temper-box-shell::real`:** init's two bind-and-listen races and one native init from kernel-chosen ports, its modes, its link and its held listeners, a second init refused those ports; init publishing a 3072-bit provider key OpenSSL reads; the fake backends' peer reservations transferred and closed, a fresh sandbox's native modes and a second one refused by its lock, the reused document's rename seen through a held handle, the named ports bound while prepared, and both named-port contentions; program login's line run by a real shell; `id` and the effective user; the standalone check's descriptors; a held non-socket descriptor; the link's form on a pseudo-terminal and on `/dev/null`.
- **`temper-engine-shell::suite`:** the stopped checkpoint taken and removed, and the stopped restore switching generations; the checkpoint lock against a serving deployment, for `--check` and `--workers`, through the engine's run and through its prepared run; the rehearsal's own scratch held by its lock and swept once released; the playground's random key; a forge's setup file written private; forge sign-in's credentials made by OpenSSL and kept; the program environment through a real shell.

## 5. Open questions

None.

## 6. Later

- **One publication face for the shells' private files:** forge setup's files, the pinned fetch, the link file, the provider copy and the fake backends each publish a private file by an exclusive creation, a write, a sync and a rename, each over a seam of its own (section 3). One face in skein's shell could serve them all; no request asks it.
- **The source command's deadline witness** contests a 10 ms product deadline against `sleep 0.1`, and shows no live child before its expiry (`../../development/slow-tests-survey-2026-11-23-temper-2.md`, "Flake causes found"); it stays as it is until its flake is seen.
- **`temper-forge-setup-shell`'s `tests/suite/boundary.rs`** is the step binary's, by its module path, and would read better under `tests/step/`.
