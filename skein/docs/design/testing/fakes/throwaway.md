# The throwaway Forgejo and the conformance

Provisional, 2026-11-16. The real Forgejo the fake forge is held to:
the throwaway that runs the pinned binary, and the conformance that
drives it and records each fact the dialect relies on. The fake forge
itself, its domain, protocol layers and peer, is `forge.md`'s; its
coverage of these two is `forge.md`, section 6.

## 1. The throwaway Forgejo

The throwaway runs the pinned Forgejo for the tools and process tiers that
need a real one. It never meets an existing Forgejo.

- **The binary.** `Forgejo::from_path` takes the path from its caller and
  reads no environment. It runs `--version` and refuses, naming it
  (`Refusal`), a binary that is unreadable or whose version line does not
  name the pin's `VERSION` (../../forge/dialects.md, section 1.3). It computes the SHA-256.
  The pin's `RUNNER_VERSION` and `names_runner` are re-exported. The kit's
  tests that need the binaries are ignored by default; their test files
  read the paths from `SKEIN_FORGEJO` and `SKEIN_FORGEJO_RUNNER`, and fail
  naming the variable when it is unset. No library code reads a variable,
  and a service's tests read their own.
- **The instance.** `start(scratch, &Settings)` writes an `app.ini` for
  loopback, SQLite and the page cap, makes the admin user and its token, and
  waits for the API. The scratch must be empty or absent and not under
  `/tmp`. The environment is cleared, with a home in the scratch, a fixed
  `PATH` and `TZ`, and no system or user git configuration.
  `Settings::small()` pages at 2, enables Actions and bounds an answer at
  16 MiB. `Settings::local_migrations`, off in `small()`, writes
  `[migrations] ALLOW_LOCALNETWORKS = true`, for the conformance's copies
  alone (section 2).
- **Attaching.** `Instance::attach(child, port, scratch, admin_token,
  &Settings)` takes a Forgejo its caller started and keeps, such as a
  service's owned forge. It waits for the API and reads the admin's login
  and id with `GET /api/v1/user`, naming no login of its own. Its `stop`
  ends the child and leaves the scratch, which stays its caller's, and
  `cli` refuses, having no work path.
- **Provisioning:** users, tokens, repositories (`Init::{Readme, Empty,
  Sha256}`), collaborators and `fresh_secret`. `Forgejo::work_path` and
  `Instance::cli` run Forgejo's command line without and with a web
  process, keeping the exit code and both streams.
- **Exchanges.** `exchange` (a token, or none) and `exchange_basic` send one
  request and return what was sent and received, through skein's HTTP client
  driven by `drive` as a one-shot host (../../runtime/shell.md, section 11). A recording
  holds no credential: `token MASKED` or `Basic MASKED`. `secrets()` lists
  every secret the instance made, for a leak scan.
- **`stop`** removes the instance and its scratch; a start that fails cleans
  up. `kill` ends the process at once and keeps the scratch, and
  `restart` starts it again on that scratch and port. `exchange_patient`
  is `exchange` with the caller's own time for the answer, for a
  migration's one long request. The conformance starts the runner
  (section 2), not the throwaway.
- **Its server dies with its test,** however the test ends: `web` starts
  dying with the thread that starts it (`spawn_dying_with_parent`,
  ../../runtime/shell.md, 3, with Kill), so an abort or a runner's kill
  leaves no Forgejo on the scratch's socket and database. Its temporary
  files are the scratch's `tmp` (`TMPDIR`), so nothing it writes lies
  outside the scratch. The commands it waits on, `migrate`, the admin's
  creation and `cli`, stay plain spawns. A service that starts its own
  Forgejo and attaches it starts it dying with its test the same way.

## 2. The conformance

`tests/forge/conformance` is a tool outside the suites. It drives a throwaway
Forgejo with a page cap of 2, so paging shows, and records what the dialect
relies on (../../forge/forgejo.md, section 2). It makes its own admin user and tokens.

- **Inputs:** `--forgejo <path>`, `--scratch` (empty or absent, not under
  `/tmp`), `--out`, and optionally `--runner <path>`, checked with the pin's
  `names_runner`. It reads no variable.
- **Transcripts:** `transcripts/<case>/{request,response}` are raw HTTP/1.1
  bytes. A response is unframed, with chunking removed and `Content-Length`
  recomputed; JSON key order, whitespace and types stay as received. A case
  replaces only its own directory. Keys, nesting, types and status lines
  must match; ids, times, counts and text may vary. `transcripts/binary.txt`
  holds the binary's version line and SHA-256.
- **Deliveries:** `deliveries/<event>/{request,response}`, one per event of
  ../../forge/README.md, section 6, with a mask map of their own.
- **Facts:** `facts.txt` has provenance lines, then a line per fact (../../forge/forgejo.md,
  section 2): name, `holds`, `fails` or `unchecked`, reason.
- **Masking:** one map masks object ids and RFC 3339 times by first
  appearance, keeping width and offset; ports, `Date` and `Last-Modified`
  take fixed values; numeric resource ids stay (provisioning has a fixed
  order).
- **Secrets never reach a file.** Before masking, the tool registers every
  token's value and its eight-digit tail, and every password. It scans each
  response body before masking and every output before writing. A leak stops
  the run naming the case and the kind of secret, never its value.
- **A second instance,** for the `setup-migrate-` facts
  (../../forge/setup/forgejo.md, section 2), run by the focused mode `--migrate`,
  which writes its own transcripts and facts alone: the source, a
  throwaway of its own on loopback, started beside the first, whose
  `[migrations] ALLOW_LOCALNETWORKS` is on in the first alone
  (`Settings::local_migrations`, `throwaway.md`, section 1, which no other caller sets).
  The default's refusal is recorded on the second, which keeps
  Forgejo's default. A loopback source speaks plain `http`, so the mode
  records Forgejo's refusal of it, then writes `[migrations]
  ALLOW_UNENCRYPTED = true` into these two instances' `app.ini` alone. The
  source holds a private repository beside its public ones, asked with
  no credentials for a private source's refusal. The
  fact of which workflow directory Actions reads takes `--runner`, and is
  `unchecked` without it. A loopback proxy of the tool's own stands between
  them and stalls a source's bytes on demand, so a migration is cut
  short at a known point: by killing the first, or by the client going
  away.
- **The groups:** the dialect's cases; the setup (accounts, tokens,
  organizations, repositories' collaborators and hooks, the admin's
  repository search, under the least admin scopes, a copy's migration); `cli-` facts
  (Forgejo's command line, outcomes only); `runner-` facts
  (registration, the runner's connection, a workflow that clones a
  private repository with the job's token); and the Actions facts, which
  run two inline host-shell jobs, one succeeding and one failing, with
  nothing downloaded. The runner groups are `unchecked` without the
  runner.
- **The runner's daemon dies with the tool,** however the tool ends:
  it starts dying with its parent (../../runtime/shell.md, 3, with
  Terminate, so it ends its host-shell jobs within its one-second
  shutdown), its home, configuration, jobs and log in the scratch's
  `runner`, and its `TMPDIR` there too.
- **`--component`** runs the kit's component (../../forge/connection.md) through
  real io sockets on the shell's ring against a fresh fixture, under the
  scopes `read:user`, `write:issue` and `write:repository`. It replaces only
  the `component-` facts and records no transcripts. Each call's cost is
  compared with the request lines counted from socket sends.

## 3. Open questions

- **The runner's start.** The conformance starts the runner (section 2).
  Whether a second user moves that start into the throwaway is open.
  Default: it stays in the conformance until a second user needs it.
