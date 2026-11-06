# Run a local box

Kind: walk

Run a whole deployment on one machine, on loopback, from its configuration to a change landed on its own Forgejo, then stop it without losing the model's account.

## Before you start

- **Binary:** `temper` on `PATH`, git 2.41+. From a checkout:
  `scripts/install.sh "$HOME/.local"`; builds need Rust's
  `wasm32-unknown-unknown` target.
- **Host actions:** `temper-host` needs compatible Node.js on the host
  as `node` for JavaScript `uses:` actions: `actions/checkout@v4`,
  `actions/upload-artifact@v3`. Put it on
  `application.machine.owned.runner.path` (default
  `/usr/local/bin:/usr/bin:/bin`); restart the box after changing it.
  Availability in an interactive shell is insufficient.
- **Host delivery checks:** `.temper/pre-pr` tools inherit the
  box process's `PATH` captured at startup: the box unit's `PATH` in a
  service. Install tools there before step 2.
  For fabrig, install `cargo` with rustfmt (`cargo fmt`) and `python3`.
  Kit PATH: `W/bin:/usr/local/bin:/usr/bin:/bin`.
  Unit example:
  `PATH=/home/box/.cargo/bin:W/bin:/usr/local/bin:/usr/bin:/bin`;
  use absolute paths for Cargo's bin directory and the kit's `W/bin`.
  Stop (step 8); restart the box unit after changing its environment.
  `application.machine.owned.runner.path` configures the runner alone.
- **Capacity:** CI and tiers on `temper-host` share
  `application.machine.owned.runner.capacity`, which defaults to 1,
  its minimum. Slow tiers block CI and landings. Use 2
  for one single-job tier plus CI, or 3 for two such tiers plus CI.
  Count concurrent jobs in multi-job workflows and other repositories
  too (temper help tier). Stop real backends with Ctrl-C or SIGTERM;
  wait for it to exit. Edit this setting in `CONFIG/temper.json`, keep
  other settings, then restart (step 2) with the same `CONFIG`,
  `DATA` and `CACHE`, keeping runners'
  registration and secrets.
- **Backends:** choose before step 2; never mix. Real needs an account,
  fetches Forgejo and runner. Fake: no account; local Forgejo real;
  GitHub, model, sign-in, remote Forgejo on loopback fakes. Build:
  `cargo build -p temper -p smith-fake-claude` to put `fake-claude`
  beside `temper`, then `scripts/bundle.sh --loopback BUNDLE`.
- **Directories:** `CONFIG`, `DATA`, `CACHE`: absolute, empty, private
  (0700), no links, for this box. `DIR`: private.
- **A model account:** real: private login export, API key, browser
  sign-in, or Claude Code; sign in at step 5 or in `CONFIG`.
- `BUNDLE`: fake backends' loopback bundle directory.
- `REQUEST`: change request file.
- `CHANGE`: the change task's number (step 6).
- `FORGE`: Forgejo's number (step 4).
- `NAME`: a Claude Code login, 1–32 lowercase letters, digits or
  hyphens (step 5).

## 1. Write the configuration

```
temper init --box --config CONFIG --data DATA --cache CACHE
```

Init writes loopback ports 8367–8371 (clients, owned Forgejo, fake forge,
demo LLM, webhooks), or free ports if busy.
Another box: empty directories; keep the first running.

Init releases ports; starts refuse occupied addresses. Step 2 on empty
directories initializes and retains ports.
Plain `temper init` then `temper box` has no playground; `--fake-backends`
refuses it.

Expect: `init: wrote DATA/worker.secret`; last line: step 2's real command.

**Claude Code:** init writes Anthropic's provider, endpoint and model beside
OpenAI's. A `claude` on `PATH` when init runs names that program; otherwise
temper fetches its pinned build when first needed. Its login directory is
`DATA/programs`, apart from your own Claude Code configuration.
`PROGRAMS`: that directory, or an existing login's parent, set as
`programs.claude_code.directory` in `CONFIG/agent.json` (step 5).

Expect: `me` lists provider 2, Anthropic, `claude_code`.
Use it in step 5; keep the demo model within init's limits.

If not: `temper init: CONFIG/temper.json: already exists` (exit 1):
use empty directories.

**Coding runs:** edit `CONFIG/agent.json`'s `environment` before step 2;
preserve the rest of the document. Init leaves the array empty. Commands
and checks receive exactly the configured environment, independent of
unit and runner PATHs.
Use deployment-specific absolute paths; no `~` or `$HOME` expansion:

```json
{"environment": [
  {"name":"PATH","value":"/home/coder/.cargo/bin:/usr/local/bin:/usr/bin:/bin"},
  {"name":"HOME","value":"/home/coder"}
]}
```

Install project tools on this PATH: Rust's `cargo` in `.cargo/bin` and
its toolchain. Give the run's user a usable `HOME`; exclude credentials.
After changes, stop (step 8) and restart the box (step 2).

## 2. Start the box

Real backends:

```
temper box --config CONFIG --data DATA --cache CACHE
```

Fake backends:

```
temper box --fake-backends --bundle BUNDLE --config CONFIG --data DATA --cache CACHE
```

Keep running. Logs: `DATA/box/logs/box.log` and each step's log beside it.

Expect:

```text
Open this link in a browser on this machine.

http://127.0.0.1:PORT/jig/v1/sign-in/link?secret=...

It signs you in as this temper's owner, once, within 2 hours.
Opened after that, it prints a fresh one here.
```

For a fresh fake box, the kernel chooses `PORT` and its four other ports,
leaving 8367–8371 to a person's box. The sandbox keeps its chosen ports on restart.
The link alone is on stdout; the other lines are on stderr. Fake backends
follow the block with
`box: effective configuration DATA/box/fake-backends/config`, then
`box: model key FILE` and `box: GitHub own App id 1, key file FILE`
(read later).

If not: a link opened once the box has given up gets no answer, and the
terminal has said why. Mend what it said and run `temper box` again, for
a new link. A taken port refuses before any link is printed.
`missing or nonexecutable adapter`: build `fake-claude` beside
`temper`. `not in current init shape`: changed `CONFIG`; use empty
directories. `retained documents must be private files`:
`chmod 600` the effective document. Refusals name the reason or restart;
retries exit 1. Keep `DATA` and report it.
Incompatible store: no translation before release. To reset, stop; move
`active`, `generations/` and `transcripts/` out of `DATA/deployment`
together; restart creates an empty store. Clients sign in again.

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

If not: refused (exit 1). Use the fresh link printed alone on the terminal:
a blank line, then the link.

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

Headers: `DATA/forge-FORGE/person_read.header` and `person_write.header`;
`curl -H @PATH` keeps tokens private.
First password: `person_password`. Repository links sign you in through temper; if unavailable, use this password (`box.log` says why).

If not: `not met: timeout` (exit 3) after 300 seconds:
run `temper ctl DIR forge list --until made:FORGE`.
`temper ctl DIR forge list` names fetch/start failures.

## 5. Give the model an account

Choose one; `me` lists providers (OpenAI 1 from init).

Imported login:

```
temper ctl DIR account import FILE --key KEY
```

Browser sign-in (fake: any HTTP client):

```
temper ctl DIR account connect PROVIDER --key KEY
```

A private API key file (fake: the printed model key file):

```
temper ctl DIR account add-key PROVIDER FILE --deployment --key KEY
```

A Claude Code login (step 1);
`ACCOUNT`: provider 2's account from `account list`:

```
temper ctl DIR account declare 2 claude_code NAME --key KEY
temper program-login --config CONFIG --data DATA --cache CACHE NAME
temper ctl DIR account list
temper ctl DIR account verify ACCOUNT --source program
```

`program-login`: signs in at `PROGRAMS/NAME`.
Existing login: parent as `PROGRAMS`, name as `NAME`; skip `program-login`.
One box at a time.

```
temper ctl DIR account list --until usable
```

Expect: `moved in: account N`, an address, `done: add-key`, or
`Program source S: NAME`, then verify's `asked`; the wait ends `met`.

If not: `key file mode 0o644 permits group or other access` (exit 1):
`chmod 600 FILE`; repeat call and key. Remote browser: provider 1 calls
back to `127.0.0.1:1455` here; give the person
`ssh -L 1455:127.0.0.1:1455 HOST`, HOST this machine, with the address.
`not met: timeout` (exit 3): browser sign-in unfinished; reopen the
address and wait again.

## 6. Ask for a change

```
temper ctl DIR chat start --project PROJECT - --key KEY < REQUEST
temper ctl DIR task show TASK --until replied
temper ctl DIR chat show TASK
temper ctl DIR task show TASK --tree
```

`TASK`: the chat; its tree lists `CHANGE`. `chat show` prints the
conversation and earlier-page commands.

Expect: `done: task TASK, chat`; `task TASK · chat · ` ends `met`;
`chat show` prints the reply.

If not: `Hello from the local fake model.` makes no change: step 8.
`chat exceeds offered bounds`
(exit 1): read `me`.

## 7. Follow the change to its landing

```
temper ctl DIR change show CHANGE --until landed --timeout 3600
temper ctl DIR change show CHANGE
```

The starter CI passes.
The pull request printed by `change show` has its diff, CI runs and
logs; temper's review gate is read with `change show`.

Expect: the panel, first line `change CHANGE ·
work/hello on forge FORGE · `, its branch and `into main`, then `met`;
`change show` names its step, `landed`, and merge commit.

If not (exit 3): `not met: timeout`: inspect `change show CHANGE` and
`task show CHANGE --tree`, act and wait again.
`not met: ended`, `not met: ended: cancelled` or `not met: ended:
failed`: ended without landing. `completed, no change` means success
without a change. `temper ctl DIR task show CHANGE` prints its report.
`not met: held`: `held: task N (REASON)` names change, writer or
reviewer; `temper ctl DIR task show N` prints its way out.
`held before registration: REASON` (exit 2): no pull request yet;
`task show CHANGE` prints its way out.

## 8. Export the account, then stop

Real backends: export before stopping:

```
temper ctl DIR account list
temper ctl DIR account export ACCOUNT FILE
```

`ACCOUNT`: the `sign_in`, `granted` account; `FILE`: absent.
Then stop with `SIGINT` (Ctrl-C) or `SIGTERM`.

Expect: `moved out: account ACCOUNT to FILE`; the last line of
`DATA/box/logs/box.log` ends `Stopped`.

If not: `account export: not signed in` (exit 2): add a key.
`temper ctl: the engine holds its N answers of the last day; room again
at TIME` (exit 2): no room for the move's answers; nothing moved.
Keep running; export after TIME.
`temper ctl: unsent` (exit 1): busy engine, nothing moved; export later.
Other failures: keep running; account stays. Export after its deadline.

## Clean up

- Reuse `CONFIG`, `DATA` and `CACHE` to resume.
  Fake: remove `DATA/box/fake-backends` to reset.
- Keep step 8's `FILE` private until import removes it.
- Remove key files once accounts are usable; `DIR` when done.

## Next

- temper help ctl: calls, keys and exits.
- temper help github: GitHub beside Forgejo.
