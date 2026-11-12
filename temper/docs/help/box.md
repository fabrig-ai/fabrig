# Run a local box

Kind: walk

Run a whole deployment on one machine, on loopback, from its configuration to a change landed on its own Forgejo, then stop it without losing the model's account.

## Before you start

- **Binary:** `temper-box` on `PATH`, git 2.41+. Checkout install:
  `scripts/install.sh "$HOME/.local"`; builds need Rust's `wasm32-unknown-unknown` target.
- **Host actions:** `temper-host` needs compatible Node.js on the host
  as `node` for JavaScript `uses:` actions: `actions/checkout@v4`,
  `actions/upload-artifact@v3`. Put it on
  `application.machine.owned.runner.path` (default
  `/usr/local/bin:/usr/bin:/bin`); restart after edits.
  Availability in an interactive shell is insufficient.
- **Host delivery checks:** `.temper/pre-pr` tools inherit the
  box process's `PATH` captured at startup: the box unit's `PATH` in a
  service. Install there before step 2.
  For fabrig: `cargo`, rustfmt (`cargo fmt` and `rustfmt`), `python3`.
  Kit PATH: `W/bin:/usr/local/bin:/usr/bin:/bin`.
  Unit example:
  `PATH=/home/box/.cargo/bin:W/bin:/usr/local/bin:/usr/bin:/bin`;
  use absolute paths for Cargo's bin and the kit's `W/bin`.
  Stop (step 8); restart the box unit after changing its environment.
  `application.machine.owned.runner.path` configures the runner alone.
- **Capacity:** CI and tiers on `temper-host` share
  `application.machine.owned.runner.capacity`, which defaults to 1,
  its minimum. Slow tiers block CI and landings. Use 2
  for one single-job tier plus CI, or 3 for two such tiers plus CI.
  Count concurrent jobs in multi-job workflows and other repositories
  too (temper help tier). Stop real backends with Ctrl-C or SIGTERM;
  closing the terminal stops the box too. Then wait for it to exit.
  Edit only this setting in `CONFIG/temper.json`; restart (step 2) with the
  same `CONFIG`, `DATA` and `CACHE`, keeping runners' registration and secrets.
- **Backends:** choose before step 2; never mix. Real: account required,
  Forgejo and runner fetched. Fake: no account; local Forgejo real;
  GitHub, model, sign-in, remote Forgejo on loopback fakes. Build:
  `cargo build -p temper-box -p smith-fake-claude` puts `fake-claude`
  beside `temper-box`; then `scripts/bundle.sh BUNDLE`.
- **Directories:** `CONFIG`, `DATA`, `CACHE`: absolute, empty, private
  (0700), no links, for this box. `DIR`: private.
- **A model account:** real: private login export, API key, browser
  sign-in, or Claude Code; sign in at step 5 or in `CONFIG`.
- `BUNDLE`: fake backends' bundle directory.
- `REQUEST`: change request file.
- `CHANGE`: the change task's number (step 6).
- `FORGE`: Forgejo's number (step 4).
- `NAME`: a Claude Code login, 1–32 lowercase letters, digits or
  hyphens (step 5).

## 1. Write the configuration

```
temper-box init --box --config CONFIG --data DATA --cache CACHE
```

Init writes loopback ports 8367–8371 (clients, owned Forgejo, fake forge,
demo LLM, webhooks), or free ports if busy.
Another box: empty directories; keep the first running.
A box trusts every program listening on this machine's loopback, since browser cookies ignore ports.

Init releases ports; starts refuse occupied addresses. Step 2 on empty
directories initializes and retains ports.
Plain `temper-box init` then `temper box` has no playground; `--fake-backends`
refuses it.

Expect: `init: wrote DATA/worker.secret`; last line: box command.

**Claude Code:** init adds Anthropic's provider and endpoint beside
OpenAI's, using `claude` on `PATH` when init runs, else fetching its pinned
build on first use. Logins: `DATA/programs`, apart from your Claude Code configuration.
`PROGRAMS`: that directory or an existing login's parent;
`CONFIG/agent.json`'s `programs.claude_code.directory` (step 5).

Expect: `me` lists provider 2, Anthropic, `claude_code`.
Use it in step 5; keep the demo model within init's limits.

If not: `temper-box init: CONFIG/temper.json: already exists` (exit 1): empty directories needed.

**Coding runs:** edit `CONFIG/agent.json`'s `environment` before step 2;
preserve the rest of the document. Init's array is empty. Commands and
checks get exactly the configured environment, independent of unit and
runner PATHs. Use deployment-specific absolute paths; no `~` or `$HOME` expansion:

```json
{"environment": [
  {"name":"PATH","value":"/home/coder/.cargo/bin:/usr/local/bin:/usr/bin:/bin"},
  {"name":"HOME","value":"/home/coder"}
]}
```

Install project tools on this PATH: Rust's `cargo` in `.cargo/bin` and
its toolchain. Run user: usable `HOME`; exclude credentials.
After edits, stop (step 8); restart the box (step 2).

## 2. Start the box

Real backends:

```
temper-box --config CONFIG --data DATA --cache CACHE
```

Fake backends:

```
temper-box --fake-backends --bundle BUNDLE --config CONFIG --data DATA --cache CACHE
```

`temper box` in a terminal runs the TUI in front, signed in and showing
the link (temper help tui); the block's other lines, fake backends'
`box:` lines and errors go to `DATA/box/logs/box.log`, and quitting the
TUI stops the box. This walk reads the link, so runs `temper-box`
without a TUI.

Keep running; step logs sit beside `DATA/box/logs/box.log`.

Expect:

```text
Open this link in a browser on this machine.

http://127.0.0.1:PORT/jig/v1/sign-in/link?secret=...

It signs you in as this temper's owner, once, within 2 hours.
Opened after that, it prints a fresh one here.
In a terminal: temper http://127.0.0.1:PORT
```

Fresh fake box: the kernel chooses `PORT` and four other ports, leaving
8367–8371 to a person's box. The sandbox retains them on restart.
Link alone: stdout; other lines: stderr. Fakes follow the block with
`box: effective configuration DATA/box/fake-backends/config`, then
`box: model key FILE` and `box: GitHub own App id 1, key file FILE`
(read later).

If not: the box gave up: the link gets no answer; the terminal says why.
Mend it; rerun for a new link.
Occupied port: refusal before any link.
`missing or nonexecutable adapter`: build `fake-claude` beside `temper-box`.
`not in current init shape`: changed `CONFIG`; need empty directories.
`retained documents must be private files`: `chmod 600` the effective document.
Refusals name reason or restart; retries exit 1. Keep and report `DATA`.
Incompatible store: no translation before release. Reset: stop; move
`active`, `generations/`, `transcripts/` together out of `DATA/deployment`;
restart with an empty store. Clients sign in again.

## 3. Sign in

Give the box's link on stdin, never as an argument:

```
temper ctl DIR sign-in -
temper ctl DIR me
```

Expect: `party 1 · LOGIN · Owner`, LOGIN the configured person; `me` lists no project yet.

Policy 1 sizes future projects; `project-policy` immediately resizes the
open period and parties' pools (temper help configure). After setup:
`project show PROJECT`; funding: temper help weekly-budget.

If not: refused (exit 1). Use fresh link alone on the terminal after a blank line.

## 4. Set up Forgejo on this machine

When `me` lists no project, leave out `--project`: setup creates a project named `hello`.

```
temper ctl DIR forge setup --org work --repository hello --key KEY
```

Expect: `setting up`, then `project: PROJECT · hello`,
`Adopted work/hello on forge FORGE as owned.` and
`It is the project's home.`

Use the printed `PROJECT` number in later commands.
Add `--project PROJECT` to choose an existing project.

`--playground` without `--project` creates `widget`:
`project: 1 · widget` on a new box.

`DATA/box/logs/box.log`:

```text
HH:MM:SSZ Forge FORGE (Forgejo): starting
HH:MM:SSZ Forge FORGE (Forgejo): started
```

The forge's hands are in `DATA/box/logs/forgejo-FORGE.log`, each after
its UTC time:

```text
YYYY-MM-DDTHH:MM:SSZ Forgejo: api http://127.0.0.1:PORT/api/v1
YYYY-MM-DDTHH:MM:SSZ Forgejo: person LOGIN
```

Headers: `DATA/forge-FORGE/person_read.header`, `person_write.header`;
`curl -H @PATH` keeps tokens private.
First password: `person_password`. Repository links sign in through temper;
if unavailable, use the password (`box.log` says why).

If not: `not met: timeout` (exit 3, 300 seconds):
run `temper ctl DIR forge list --until made:FORGE`.
`temper ctl DIR forge list` names fetch/start failures.

## 5. Give the model an account

Choose one; `me` lists providers (init's OpenAI: 1).

Imported login:

```
temper ctl DIR account import FILE --key KEY
```

Browser sign-in (fake: any HTTP client):

```
temper ctl DIR account connect PROVIDER --key KEY
```

Private API key file (fake: printed model key file):

```
temper ctl DIR account add-key PROVIDER FILE --deployment --key KEY
```

A Claude Code login (step 1);
`ACCOUNT`: provider 2's account from `account list`:

```
temper ctl DIR account declare 2 claude_code NAME --key KEY
temper-box program-login --config CONFIG --data DATA --cache CACHE NAME
temper ctl DIR account list
temper ctl DIR account verify ACCOUNT --source program
```

Use fresh NAME (even after forgetting) and KEY.

`program-login`: signs in at `PROGRAMS/NAME`; for an existing login, use
its parent as `PROGRAMS`, name as `NAME` and skip `program-login`.
One box at a time.

```
temper ctl DIR account list --until usable
```

Expect: `moved in: account N`, an address, `done: add-key`, or
`Program source S: NAME`, then verify's `asked`; wait ends `met`.

If not: `key file mode 0o644 permits group or other access` (exit 1):
`chmod 600 FILE`; repeat call and key. Remote browser: provider 1's callback
is `127.0.0.1:1455` here; give the person the address and
`ssh -L 1455:127.0.0.1:1455 HOST`, HOST this machine.
`not met: timeout` (exit 3): browser sign-in unfinished; reopen its address and wait again.

## 6. Ask for a change

```
temper ctl DIR chat start --project PROJECT - --key KEY < REQUEST
temper ctl DIR task show TASK --until replied
temper ctl DIR chat show TASK
temper ctl DIR task show TASK --tree
```

`TASK`: chat; tree lists `CHANGE`. `chat show`: conversation and earlier-page commands.

Expect: `done: task TASK, chat`; `task TASK · chat · ` ends `met`;
`chat show` prints the reply.

If not: `Hello from the local fake model.`: no change; step 8.
`chat exceeds offered bounds` (exit 1): read `me`.

## 7. Follow the change to its landing

```
temper ctl DIR change show CHANGE --until landed --timeout 3600
temper ctl DIR change show CHANGE
```

The starter CI passes.
The pull request printed by `change show` has its diff, CI runs and
logs; temper's review gate is read with `change show`.

Expect: panel's first line `change CHANGE ·
work/hello on forge FORGE · `, its branch and `into main`, then `met`;
`change show` names its step, `landed`, and merge commit.

If not (exit 3): `not met: timeout`: inspect `change show CHANGE` and
`task show CHANGE --tree`; act, wait again.
`not met: ended`, `not met: ended: cancelled` or `not met: ended:
failed`: ended without landing. `completed, no change` means success
without a change. `temper ctl DIR task show CHANGE` prints its report.
`not met: held`: `held: task N (REASON)` names change, writer or
reviewer; `temper ctl DIR task show N` prints its way out.
`held before registration: REASON` (exit 2): no pull request yet;
`task show CHANGE` prints its way out.

## 8. Export the account, then stop

Real backends: export first:

```
temper ctl DIR account list
temper ctl DIR account export ACCOUNT FILE
```

`ACCOUNT`: `sign_in`, `granted`; `FILE`: absent.
Then stop with `SIGINT` (Ctrl-C) or `SIGTERM`.

Expect: `moved out: account ACCOUNT to FILE`; the last line of
`DATA/box/logs/box.log` ends `Stopped`.

If not: `account export: not signed in` (exit 2): add a key.
`temper ctl: the engine holds its N answers of the last day; room again
at TIME` (exit 2): no room for move's answers; nothing moved. Keep running;
export after TIME. `temper ctl: unsent` (exit 1): engine busy, nothing moved;
export later. Other failures: keep running; account stays; export after its deadline.

## Clean up

- Reuse `CONFIG`, `DATA` and `CACHE` to resume.
  Fake: remove `DATA/box/fake-backends` to reset.
- Keep step 8's `FILE` private until import removes it.
- Remove key files once accounts are usable; `DIR` when done.

## Next

- temper help ctl: calls, keys and exits.
- temper help github: GitHub beside Forgejo.
