+++
id = "managed-sitting"
part = "managed-sitting"
kind = "feature"
needs = ["b0-offline"]
files = ["temper/crates/temper/tests/live_github_managed.rs", "temper/crates/temper/tests/support/managed_client.rs", "temper/crates/temper/tests/support/managed_sitting.rs", "temper/scripts/proof-live-github-managed.py", "temper/docs/design/domain/testing.status.md"]
effort = "high"
minutes = 180
+++

## Goal

`managed_ready_client` leaves `#[ignore]` and joins the process tier. It
sets up the managed GitHub sitting's fake half itself, as
`scripts/proof-live-github-managed.sh fake` does, and drives Ready
through its sign-in and its adoption there. The sitting's driver still
runs the same test as its client. `managed_relay` keeps `#[ignore]`, its
reason naming github.com.

This spec departs from the request's "effort medium". It writes a new
process story of a few hundred lines, which the plan cannot run.

Acceptance:

- `temper/crates/temper/tests/live_github_managed.rs`:
  - `managed_ready_client` has no `#[ignore]`. When
    `TEMPER_MANAGED_CLIENT` is set, it is the sitting's client, as it is
    today: `managed_client::run(mode)`, the link on stdin. When it is not
    set, it is the fake sitting, set up by the test:
    `managed_sitting::run()`. A doc comment says both and cites
    `temper/docs/design/domain/testing.md`, 2.6.
  - `managed_relay`: `#[ignore = "live: github.com, where a person makes and installs the App; scripts/proof-live-github-managed.sh live"]`.
    Its body is unchanged.
  - The module doc names both, citing 2.6 and 2.7.
- `support/managed_client.rs`:
  - `pub enum Mode { SignIn, Adopt }`;
  - `pub fn role() -> Option<Mode>` reads `TEMPER_MANAGED_CLIENT`:
    `sign-in` or `adopt`, absent is `None`, anything else panics;
  - `pub fn run(mode: Mode)` reads the private printed link from stdin,
    as today, and calls `ready(Path::new("."), &link, mode)`;
  - `pub fn ready(root: &Path, link: &str, mode: Mode)` is today's body
    from the link on, its scratch root the given `root` in place of
    `"."`.
  The person's steps, patience and assertions are unchanged.
- `support/managed_sitting.rs` (new, included by `#[path]` as the binary's
  other modules are) holds `pub fn run()`. It follows
  `scripts/proof-live-github-managed.py`'s fake mode (`Sitting.fake`,
  `configure`, `connect_fake`, `ready_client`, `scan_secrets`) step for
  step, through the adoption:
  1. A private scratch (`skein_scratch::Scratch`, mode 0700) and the
     sitting's child environment: `PATH`, `HOME` and `TMPDIR` the
     scratch, `LANG=C.UTF-8`, `GIT_CONFIG_NOSYSTEM=1`,
     `GIT_CONFIG_GLOBAL=/dev/null`, `GIT_TERMINAL_PROMPT=0`, and nothing
     else inherited.
  2. The fake GitHub: the driver's document as it stands
     (`relay_managed::OWNER` and `NAME`, the person `managed-person`, two
     repositories, three accounts, `token_life_seconds` 3600), written
     0600. `temper fake forge --document PATH --ready 'listening '`
     (`skein_scratch::paths::binary("temper")`), its `listening ADDR`
     line read.
  3. The relay: the test's own executable
     (`std::env::current_exe()`), run as
     `--ignored --exact managed_relay --nocapture`, its stdin a pipe, its
     environment the sitting's plus `TEMPER_MANAGED_MODE=fake`,
     `TEMPER_MANAGED_SCRATCH`, `TEMPER_MANAGED_UPSTREAM` (the fake's
     address), `TEMPER_MANAGED_PREFIX` (`temper/live-<run>/`),
     `TEMPER_MANAGED_EVIDENCE` (a path in the scratch that doesn't exist
     yet) and `TEMPER_MANAGED_PERSON=managed-person`. Then
     `relay-ready.json`'s `api` and `git` are read.
  4. The engine alone: `tests/support/github/engine.json` rendered as
     `configure` renders it. That is the three placeholders, the
     doubled braces, `"acme"` as `OWNER`, `policies` a copy of
     `projects`, `people.projects` 3, and the forge's `git` and `web`.
     Add `w1.digest` and `charters/chat.txt`. Then `temper serve
     --config --data --cache`, and its printed `http://` link read, whose
     address is the chosen client port
     (`temper_fixtures::ports::addresses::<1>(&[])`).
  5. Ready signs in: `managed_client::ready(scratch, &link, Mode::SignIn)`.
  6. Connect: `client/address` written 0600. `temper ctl CLIENT forge
     connect 1 github --key managed-connect`, with stdin `/dev/null`, and
     its `http://` line taken. Then the pages: the engine's page (200)
     gives the form's `action` and the `manifest` input's value,
     HTML-unescaped. The form is posted to the fake (200), whose
     `<a id="create">` is followed (302). Its `Location` is followed with
     `&account=OWNER&repository_ids=1` appended (302), and the last
     `Location` gives 200 holding `temper has the installation`. Every
     URL is `http`, on the fake's or the engine's address and no other.
     Then `ctl forge list --until settled`.
  7. Ready adopts: `managed_client::ready(scratch, &link, Mode::Adopt)`.
  8. The end, as `scan_secrets` does it: the engine and the fake stopped
     (SIGTERM, then a kill past 5 s). Then the driver's secrets line goes
     to the relay's stdin (`{"App_key_DER":[],"App_key_base64":[],"JWT":[],"installation_token":[]}`
     plus a newline) and stdin is closed. The relay exits 0 within 45 s,
     having printed `no secret bytes in the relay log or the record`.
     The evidence file exists, mode 0600.
  Every child is stopped on any ending, a panic included, by a guard of
  the module's. The scratch is removed on a pass. Waits use `Patience`
  (`support/patience.rs`), the driver's figures: 30 s for a ready line
  or file, 60 s for a ctl call, 10 s for a page.
- `scripts/proof-live-github-managed.py`, `ready_client`: the harness is
  run as `--exact managed_ready_client --nocapture`, without `--ignored`.
  The test isn't ignored now, and libtest's `--ignored` would select
  nothing and exit 0. A comment says so. Nothing else in the driver
  changes. The relay keeps `--ignored`.
- `heavy cargo nextest list -p temper --profile process-temper` lists
  `live_github_managed managed_ready_client`. `heavy cargo nextest list -p
  temper` (focused) doesn't, and still lists the six `relay_managed::`
  unit tests. `managed_relay` is listed by neither.
- `temper/docs/design/domain/testing.status.md`:
  - the bullet that begins `**Not yet:** 2.6: \`managed_ready_client\` is
    ignored` is replaced whole by:

    ```
    - **Not yet:** 2.6: `managed_ready_client`'s fake sitting has no
      runtime result in profile `process-temper`, and its client role no
      run under `scripts/proof-live-github-managed.sh fake` since the test
      took its sitting: the plan `ignored-tiers` compiled and listed it,
      and the next after-landing run and the owner's next dry run prove
      them (11).
    ```
  - in the **Departs** entry that begins `2.7: the managed sitting's
    \`fake\` and \`fake-wide\` runs`, those words become
    `` 2.7: the managed sitting's `fake` and `fake-wide` runs, `managed_ready_client`'s fake sitting ``.
    The entry's reason stays.
- `heavy cargo check -p temper --tests`,
  `heavy cargo clippy -p temper --all-targets -- -D warnings` and
  `heavy cargo nextest run -p temper` (focused) pass.

## Inputs

- `temper/docs/plan/ignored-tiers/notes/common.md`: every rule there
  holds. In particular, never run `managed_ready_client`, the relay, the
  driver or either script. The test is compiled and listed, not run.
- `temper/docs/design/domain/testing.md`, 2.6 (*Two drivers' offline
  proofs*, the managed sitting's Ready), 2.7 (*What needs a person*,
  `managed_relay`), and 7 (the `live_github_managed.rs` line).
- `temper/docs/design/protocol/setup.md`, 17.2 (Connect makes the App
  under the person's own account).
- `skein/docs/foundation/testing-strategy.md`, 2.10 (deadlines on the
  real clock) and 8.1.
- The driver, `temper/scripts/proof-live-github-managed.py`: its fake
  mode is the setup you port. Its push, planted refusals, cleanup and
  `check_wide_refusal` stay the driver's own.

## Owns and may change

- Owns: the files in the header. `support/managed_sitting.rs` is new.
- Not yours: `.config/nextest.toml` (`b0-offline` wrote the selection; if
  it doesn't select the test, that's a `Question:` with the lists'
  output), `support/relay_managed.rs` (read its `run`, `Shared::scan` and
  constants; don't change them), `temper/scripts/proof-live-github-managed.sh`,
  `support/mod.rs` and every other support module.
- The binary includes modules by `#[path]` and never `pub mod support;`,
  which would pull `support::forge_land`'s unit tests into it. Include
  another support file by `#[path]` only if it names no `super::` item
  the binary lacks. Otherwise write the small helper (an HTTP exchange, a
  child with its lines read) in `managed_sitting.rs`.
- Options, each pre-approved:
  - The children may run on `std::process` with their output drained on
    threads, as the driver does, or on skein-world's `Binary` as
    `support/cluster.rs` does. Choose one and say why.
  - If `#![expect(dead_code, ...)]` at the binary's root becomes
    unfulfilled, remove it, or narrow it to the items still dead, and
    say so.
  - If an HTML value needs more than `&amp;`, `&quot;`, `&#39;`, `&lt;`
    and `&gt;` unescaped, unescape what the page holds and name it.
  - If the engine template has gained placeholders since the driver was
    written, render them as the process tier's GitHub story
    (`support/github.rs`, `restart`) binds them, and say so.

## Steps

1. Split `managed_client.rs` into `Mode`, `role`, `run` and `ready`. Make
   `managed_ready_client` dispatch on `role()`. `heavy cargo check -p
   temper --tests`.
2. Write `managed_sitting.rs` by the driver's fake mode, step by step,
   with a guard that stops every child. `heavy cargo check -p temper
   --tests` while errors last.
3. Change `managed_relay`'s reason, the driver's `ready_client` argv and
   the two status entries.
4. The two lists of the acceptance.
5. Once: `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`;
   `heavy cargo clippy -p temper --all-targets -- -D warnings`;
   `heavy cargo nextest run -p temper`, focused only.
6. Commit, run the citations check (`notes/common.md`), push.

## Report

The implementer's report, with: the lists' lines; how the children run
and stop; each step of the driver's fake mode and where it sits in
`managed_sitting.rs`, or why it doesn't; and what the after-landing run
should watch for first. The test was not run, and the report says so.
