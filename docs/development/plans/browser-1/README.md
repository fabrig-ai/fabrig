# browser-1: the browser shell, milestone 1

2026-10-19. Start commits: skein `8644b70` ("Separate connection
behaviours and name sampled capacities"), temper `52709d278` ("Ruling 166
of forge-kit"); jig `8cc7e79f`, smith `fa1205e5` and hearth `61ab3d1` are
read only. Every command of this plan ran once at skein `b481ad9` and
temper `cd8fada9a` (`findings/base-runs.md`), whose code under the paths
this plan changes is the start commits'; each session starts from its
repository's `origin/main`. Checked again at skein `31bbf58` and temper
`475136369`: the tools plan's repin has landed (`8c02b3b9e`), and
real-loop 08.2 has copied the configuration (Rules, hotspots).

temper gets a browser shell: a person opens the sign-in link `temper
serve` printed, and Chromium shows Chats and a Chat, served by temper's
own listener from a bundle it read at start (`client/browser.md`,
milestone 1). skein first gives its fake person what the stories need
(the page's address, Shift+Enter, a loaded page before back and forward)
and fixes a click inside a button or a menu item (`ui-web.md`, 6). temper
then serves the bundle under jig's page policy, maps its addresses, builds
its shell and look, and proves in a browser tier of ten stories only
what a browser alone shows (decisions.md, For the user). Milestone 2, the
inbox, a task's page and the adoption form, is `later.md`'s.

## Inputs

- temper: `docs/design/client/browser.md` (milestone 1, sections 2, 4
  and 6), `docs/design/client/README.md`, `docs/design/protocol/README.md`
  (3, 5), `docs/design/protocol/configuration.md` (3, 7, 8),
  `docs/design/domain/testing.md` (2, whose opening sets the browser
  tier; 2.6, the process tier), `docs/design/STATUS.md`.
- jig: `docs/design/protocol/browser.md` (12.1, the page policy; 12.2,
  the guards).
- skein: `docs/design/ui-web.md` (4.3, 4.4, 6, 14), skein's commit
  `15868de21` (back and forward awaits a loaded page),
  `docs/foundation/programming-model.md`,
  `docs/foundation/testing-strategy.md`.
- This plan's `decisions.md`, `code-organization.md` and `findings/`.

## The sessions

A session waits for what "Needs at start" names; an increment that needs
more says so in its row and its file. "temper 01.2" means that increment
has landed on temper's `origin/main`. The tools plan's closing repin,
`tools/temper-repin`, has landed (temper `8c02b3b9e`), so temper's
sessions wait on nothing at the start.

### skein

| File | Session | Needs at start | Increments that need more |
|---|---|---|---|
| `skein/01-prerequisites.md` | skein 01: the fake person reads the page's address and presses Shift+Enter; back and forward awaits a loaded page; a click inside a button or a menu item finds its target | nothing | |

### temper

| File | Session | Needs at start | Increments that need more |
|---|---|---|---|
| `temper/01-serving.md` | temper 01: the bundle in the configuration, its limits and start's checks; the clients' listener's routes; jig's page policy on every answer | nothing | |
| `temper/02-shell.md` | temper 02: the address codec and the client's wasm32 lints; `temper-client-shell-web`; `scripts/bundle.sh` and the gate's bundle; history, links and the title | nothing | |
| `temper/03-look.md` | temper 03: the browser's signed-out words; the composers submit on Enter and keep line breaks; the look's tokens and their rules in `temper.css` | nothing | 03.3: temper 02.3 |
| `temper/04-browser.md` | temper 04: the browser tier, `temper-client-browser-world`, and its ten stories | temper 01.2, 02.3; skein 01.4 | 04.2: temper 02.4, 03.1; 04.3: temper 02.4; 04.4: temper 03.2; 04.5: temper 01.3 |

**How many at once:** three on hel8 (4 cores, 7 GB, 36 GB free), of
which at most two are temper's: a temper worktree's target holds about
10 GB after the suite (skein's about 0.7 GB). At the start: skein 01, temper 02 and temper 01. Then temper 03 in the slot temper 01
frees (its three increments to temper 02's four), and temper 04 in the
slot temper 02 frees, once temper 01.2, 02.3 and skein 01.4 have landed.
skein 01's slot then stays empty: a third temper target would leave
hel8 about 6 GB.

**The critical path:** skein 01.1 → 01.2 → 01.3 → 01.4 → temper 04.1 →
04.2 → 04.3 → 04.4 → 04.5: nine steps, about 3.6 hours at 24 minutes a
step (temper's tools run's `chain_step_min`, development.md 5.2), and
about 4.1 at the longest step of smith's live-text run, 27.1 minutes, as
the upper bound: no run has computed a `chain_step_max` yet. temper 02.1
to 02.3 and 01.1 to 01.2 land beside skein 01, before 04.1 needs them;
temper 03.1 and 03.2 land before 04.2 and 04.4 need them.

## Rules of this plan

- **The graph:** projects `home-free-src-rust-temper`,
  `home-free-src-rust-skein` and `home-free-src-rust-jig`. Until the
  supervisor reports an index past a repository's `origin/main`, use
  `rg` for that repository. Check every name the graph gives with `rg`
  in the worktree.
- **Browser tests sparingly:** the browser tier holds the ten stories
  decisions.md's "For the user" names, and no other. What else a session
  proves, it proves on the host: in unit tests, in the service's and the
  view's worlds, and in the process tier. A browser test beyond the ten
  is a question to the supervisor.
- **The repin:** temper 04.1 starts with a commit of its own that moves
  temper's skein and jig to their `origin/main` (development.md, 5.7).
  What else on skein's or jig's `main` breaks temper there is that
  commit's to repair, within its increment's Size; a repair larger than
  that is a question to the supervisor.
- **Dependency lines:** a step that adds one copies the repository URL
  its `Cargo.toml` already uses, `github.com/fabrig-dev/…` since the
  organization's rename (temper `5ecc737c`); a workspace that named two
  URLs for one repository would hold two copies of its types.
- **A new crate's license:** skein 01 adds no crate. temper's new crates
  (`temper-client-shell-web`, 02.2; `temper-client-browser-world`, 04.1)
  take every workspace-inherited field their neighbours take, and
  `license.workspace = true` once temper's workspace declares a license,
  as skein's members do (skein `1222e7b`); no gate checks it yet.
- **The Chat page stays open to live text:** temper 03 changes the Chat
  page's composer and its tokens only. It does not reorder or rebuild
  the turns' list, which live text's later plan streams into.
- **The configuration's new key:** temper 01.1 adds
  `application.clients.bundle` to every configuration the repository
  holds, in the same increment (code-organization.md, 4).
- **Hotspots,** files another run changes while this one runs:
  - temper's `Cargo.toml` and `Cargo.lock` (temper 02.2, 02.3, 04.1): the
    real-loop plan and forge-kit's temper steps;
  - `crates/temper-service/src/startup/application.rs` and `src/limits.rs`
    (temper 01.1): forge-kit's temper 03.1;
  - the configuration's literals, which temper 01.1 gives
    `"bundle": null`: `docs/examples/owned.json` and
    `crates/temper-service/src/tests/fixtures/` (forge-kit's temper 02.1
    and 03.1); `tests/one-process/src/fixtures.rs` (forge-kit's temper
    03.1 and 05.2, real-loop 13); `crates/temper/tests/support/configuration.rs`
    and real-loop 08.2's copy of it, `tests/real-loop/src/fixtures.rs`,
    which real-loop 09 changes, beside `support/forgejo.rs` and
    `support/mod.rs`, which real-loop 10.1, 10.2 and 13.3 change (its
    README, hotspots; temper `1db350240`);
  - `crates/temper/src/lib.rs` (temper 01.1): real-loop 05.6 and
    forge-kit's temper 03.1;
  - `crates/temper-client-shell-cli/src/limits.rs` (temper 01.3): the
    real-loop plan's `temper ctl`, whose 05.1 wrote it;
  - `tests/client/src/fixtures.rs` (temper 03.1): real-loop 06.3 to 06.5;
  - `crates/temper-client-view/src/pages/chat.rs` (temper 03): live
    text's later plan, whichever lands second taking the other's
    changes;
  - temper's `docs/design/STATUS.md` and `docs/design/domain/testing.md`:
    every plan, and the designer's amendments to `client/browser.md`;
  - skein's `docs/design/STATUS.md`, `docs/design/testing.md`, root
    `Cargo.toml` and `Cargo.lock`: forge-kit (carbon) and LLM accounts
    (x1). skein 01 changes no other file of theirs: it adds no crate,
    and leaves `skein-fake-peers` alone (decisions.md, Design
    questions 1).

## Live services

None. The browser tier runs Chromium 147 from `SKEIN_TEST_CHROMIUM` on
loopback, with no network.

## One test of a suite that needs a built artifact

skein's browser suite, in a worktree of skein:

```
sh -c 'cargo build --target wasm32-unknown-unknown -p skein-ui-example-shell-web && cargo run -p skein-ui-bundle -- --module target/wasm32-unknown-unknown/debug/skein_ui_example_shell_web.wasm --name skein_ui_example_shell_web --prefix /bundle/ --stylesheet examples/ui/shell-web/notes.css --out target/ui-example-bundle --noscript "Notes needs WebAssembly and scripts." && cargo build -p skein-ui-example-server'
sh -c 'SKEIN_UI_EXAMPLE_BUNDLE="$(pwd)/target/ui-example-bundle" cargo nextest run -p skein-ui-web-tests --profile browser -E "test(=back_and_forward_move_between_the_list_and_a_note)"'
```

temper's browser suite, from temper 04.1, in a worktree of temper:

```
sh -c 'scripts/bundle.sh target/temper-bundle && cargo build -p temper'
sh -c 'TEMPER_TEST_BUNDLE="$(pwd)/target/temper-bundle" cargo nextest run -p temper-client-browser-world --profile browser -E "test(=<the story>)"'
```

## Process tier

`scripts/proof-one-process.sh`, in a checkout of temper's
`origin/main`, with `TEMPER_FORGEJO` and `TEMPER_FORGEJO_RUNNER` set:
temper 01 changes the clients' listener that `temper serve` runs. hel8
held neither binary at first, and the tier's first test stopped
("TEMPER_FORGEJO must name the pinned Forgejo", `findings/base-runs.md`,
10). With Forgejo 17.0.0 and runner 13.2.0 under `~/.local/opt`, it runs
19 tests in about 2.5 minutes (10b); the run's prerequisites check both.

## Prerequisites the run checks at its start

The coordinator runs each check once, before the first session (3 when
temper 01 becomes ready), and writes `blocked.md` (kind: process) on a
failure:

1. `"$SKEIN_TEST_CHROMIUM" --version | grep -q 'Chromium 147\.'` exits 0.
2. `rustup target list --installed | grep -c -x wasm32-unknown-unknown`
   prints 1.
3. **When temper 01 becomes ready:** `test -x "$TEMPER_FORGEJO"` and
   `test -x "$TEMPER_FORGEJO_RUNNER"`: the process tier starts a
   throwaway Forgejo and its runner, at the versions
   `skein_forge::forgejo::pin` names.

## Repin

temper's, in temper 04.1 (Rules). skein 01 adds two steps to its fake
person, a request to `skein-browser` and a field to its `Request::Key`,
whose constructors are all skein's; it gives `back_and_forward` an
argument whose only caller is skein's own, and changes the click
listener of `skein-ui-web`, so nothing of smith, jig or hearth breaks; they take skein at their next repins
(decisions.md; `later.md`).

## Kit steps

None: the plan changes nothing in `scripts/dev`.

## Pulse

Other runs land on the same `main`, so the aims cover the packages this
plan's sessions change, measured as their tests' summed time
(development.md, 5.2): `temper-service`, `temper-client-domain`,
`temper-client-view`, `temper`, `temper-client-shell-terminal`,
`temper-client-world`, and `temper-client-browser-world`, whose units
run in the default profile and its stories in the browser profile (Done
when, 3). `temper-client-shell-web` holds no host code. At the base
they sum to 5.50 s focused and 0.41 s fuzzy run alone
(`findings/base-runs.md`, 8c), and to 5.09 s and 0.38 s inside the whole
suite, as the pulse reads them from a land gate's log (8d); the focused
sum swings by about 0.6 s between gate runs (8b).
The run may add what its Proof bounds sum to (development.md, 5.6):

- **Focused, 5.7 s.**
  - Thirty-four unit tests (01.1 to 01.3, 02.1, 03.1 to 03.3, 04.1),
    each under 20 ms: 0.7 s.
  - temper 01.1's three start tests, each under 0.5 s: 1.5 s.
  - temper 01.2's five and 01.3's one world tests, and 03.2's story,
    each under 0.5 s: 3.5 s.

  Expected: about a third. The service's world tests take 0.01 s to
  0.03 s at the base.
- **Fuzzy, nothing:** no session adds a fuzzy test.

```
words: actual|genuine|legacy|compat|hack|workaround
types: ^(Legacy|Old|Compat)[A-Z]|V2$
file-lines: 800
budgets: 15 60
aims: temper 12 1 temper-service temper-client-domain temper-client-view temper temper-client-shell-terminal temper-client-world temper-client-browser-world
```

## Done when

Each command runs once, after `done.md`, in a worktree of temper (or
skein, for skein's) at `origin/main`, the plan's worktrees removed.

1. Every step is landed or dropped: `plan status` lists none waiting,
   ready, running or blocked.
2. Each session file's "Done when" holds, in the sessions table's order.
3. temper's gate builds the bundle and runs the browser tier:
   `scripts/check.sh` exits 0; then `sh -c 'test
   "$(cargo nextest list -p temper-client-browser-world --profile
   browser --message-format oneline | wc -l)" -eq 10'` exits 0.
4. The process tier: with `TEMPER_FORGEJO` and `TEMPER_FORGEJO_RUNNER`
   exported (Prerequisites, 3), `scripts/proof-one-process.sh` exits 0. At the base it ran 19 tests in
   142.5 s (`findings/base-runs.md`, 10b).
5. The test-time aims (Pulse), measured once, three runs a profile:

   ```
   bash -c 'set -o pipefail; export NEXTEST_DOUBLE_SPAWN=0; for p in default default default fuzzy fuzzy fuzzy; do cargo nextest run --workspace --profile $p 2>&1 | sed -nE "s/^ *PASS \[ *([0-9.]+)s\] \([^)]*\) ([^: ]+)[: ].*/\1 \2/p" | awk "\$2 ~ /^temper(-service|-client-domain|-client-view|-client-shell-terminal|-client-world|-client-browser-world)?\$/ {t += \$1; n += 1} END {if (n == 0) exit 1; print t}" || exit 1; done'
   ```

   prints six sums, and exits non-zero when a run fails or reads no
   `PASS` line. The
   median of each profile's three is set against its aim (12 s and
   1 s). An aim missed is recorded with the three slowest tests by name
   in decisions.md, for the user; the run is done either way.
6. No budget was raised: `rg -n -A2 '^\[profile\.(default|fuzzy)\]'
   .config/nextest.toml` shows `global-timeout = "15s"` and `"60s"`, as
   at the base, in temper; skein's, as at its base
   (`findings/base-runs.md`).
