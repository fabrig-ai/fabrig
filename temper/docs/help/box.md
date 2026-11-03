# Run a local box

Kind: walk

Run a whole deployment on one machine, on loopback, from its configuration to a change landed on its own Forgejo, then stop it without losing the model's account.

## Before you start

- **The binary:** `temper` on `PATH`, git 2.41 or later. From a checkout:
  `scripts/install.sh "$HOME/.local"`; builds need Rust's
  `wasm32-unknown-unknown` target.
- **Host actions:** `temper-host` needs compatible Node.js on the host
  as `node` for JavaScript `uses:` actions: `actions/checkout@v4`,
  `actions/upload-artifact@v3`. Put it on
  `application.machine.owned.runner.path` (default
  `/usr/local/bin:/usr/bin:/bin`); restart the box after changing it.
  Availability only in an interactive shell is insufficient:
  the runner uses the configured PATH.
- **Capacity:** CI and tiers on `temper-host` share
  `application.machine.owned.runner.capacity`, which defaults to 1,
  the minimum. A slow tier blocks CI and landings. Use 2
  for one single-job tier plus CI, or 3 for two such tiers plus CI.
  Count concurrent jobs in multi-job workflows and other repositories
  too (temper help tier). On real backends, stop with Ctrl-C or SIGTERM;
  wait for `Stopped`. Edit this setting in `CONFIG/temper.json`, keep
  other settings, then restart as in step 2 with the same `CONFIG`,
  `DATA` and `CACHE`. This updates prepared runners, keeping their
  registration and secrets.
- **Backends:** choose before step 2; never mix them. Real needs an
  account and fetches Forgejo and its runner. Fake needs no account:
  GitHub, model, sign-in and remote Forgejo are loopback fakes;
  local Forgejo stays real. From a checkout:
  `cargo build -p temper -p temper-fake-claude` to put `fake-claude`
  beside `temper`, then `scripts/bundle.sh --loopback BUNDLE`.
- **Directories:** `CONFIG`, `DATA`, `CACHE`: absolute, empty, private
  (0700), without links; for this box alone. `DIR`: private.
- **A model account:** real backends need a private login export or API
  key, browser sign-in here, or Claude Code (`claude` on `PATH`), signed
  in at step 5 or in your configuration directory.
- `BUNDLE`: the loopback bundle directory, on fake backends.
- `REQUEST`: change request file.
- `CHANGE`: the change task's number (step 6).
- `FORGE`: Forgejo's number (step 4).
- `NAME`: a Claude Code login, 1–32 lowercase letters, digits or
  hyphens (step 5).

## 1. Write the configuration

```
temper init --config CONFIG --data DATA --cache CACHE
```

Init chooses five loopback ports: 8367–8371 for clients, owned Forgejo,
fake forge, demo LLM and webhooks, or free kernel ports if occupied.
Both files agree. Another box needs separate empty directories;
keep the first running.

Later starts refuse occupied saved addresses. Init releases ports;
step 2's `temper box` on empty directories runs init and retains them.
Finish edits before starting; fake backends require unchanged `CONFIG`.

Expect: `init: wrote DATA/worker.secret`; the last line names step 2's
real-backend command.

**Claude Code:** replace OpenAI's provider with Anthropic's below.
`MODEL`: a Claude Code model, such as `sonnet` or `opus`.
`PROGRAMS`: private login directory, such as `DATA/programs`, or your
signed-in directory's parent (step 5). `DIR` holds scratch copies.

```
jq --arg model MODEL '.engine.llm |= (
  (.endpoints[] | select(.name == "openai")) as $openai
  | (.models[] | select(.endpoint == "openai")) as $first
  | .providers = [{name: "anthropic", number: 2, signed_in_plan: false,
      manage_usage: "https://claude.ai/settings/usage", models: [$model],
      sources: {programs: ["claude_code"]}}] + [.providers[] | select(.name != "openai")]
  | .endpoints = [$openai + {name: "claude", number: 2, dialect: 2, provider: "anthropic",
      program: "claude_code", address: {host: "api.anthropic.com", port: 443},
      server_name: "api.anthropic.com"}] + [.endpoints[] | select(.name != "openai")]
  | .models = [$first + {name: $model, endpoint: "claude"}] + [.models[] | select(.endpoint != "openai")])' \
  CONFIG/temper.json > DIR/temper.json && cat DIR/temper.json > CONFIG/temper.json
jq --arg model MODEL --arg claude "$(command -v claude)" --arg programs PROGRAMS '
  (.endpoints[] | select(.name == "openai")) as $openai
  | .endpoints = [$openai + {name: "claude", number: 2, dialect: 2, provider: "anthropic",
      program: "claude_code", address: "api.anthropic.com:443", server_name: "api.anthropic.com",
      models: [$openai.models[0] + {name: $model}]}] + [.endpoints[] | select(.name != "openai")]
  | .programs.claude_code = {path: $claude, directory: $programs}' \
  CONFIG/agent.json > DIR/agent.json && cat DIR/agent.json > CONFIG/agent.json
```

Expect: no output; `me` lists provider 2, Anthropic, `claude_code`.
Use its way in step 5. Keep the demo model within init's limits.

If not: `temper init: CONFIG/temper.json: already exists` (exit 1):
use empty directories.

## 2. Start the box

Real backends:

```
temper box --config CONFIG --data DATA --cache CACHE
```

Fake backends:

```
temper box --fake-backends --bundle BUNDLE --config CONFIG --data DATA --cache CACHE
```

Keep running; redirect background output to a file.
Its sign-in link follows `Open this link in a browser on this machine.`

Expect: real, `Logs: DATA/box/logs/`. Fake,
`box: effective configuration DATA/box/fake-backends/config`; among
the next lines, `box: model key FILE` and
`box: GitHub own App id 1, key file FILE`, the files later steps read.

If not: `missing or nonexecutable adapter`: build `fake-claude` beside
`temper`. `not in current init shape`: use empty directories; `CONFIG`
changed after init. `retained documents must be private files`:
`chmod 600` the effective document. Refused state names its restore
reason or restart step; retries end at exit 1. Keep `DATA` and report it.
An incompatible store names the older build's differing family layout.
No store translation before release. To start afresh, stop and move
`active`, `generations/` and `transcripts/` out of `DATA/deployment`
together. Restart creates an empty store; clients sign in again.

## 3. Sign in

Give the box's link on stdin, never as an argument:

```
temper ctl DIR sign-in -
temper ctl DIR me
```

Expect: `party 1 · owner · Owner`; `me` lists `1 · Project 1 · owner`.

First start made project 1 and its owner. When `settings apply` raises
limits, fund its period and role 0's pool with distinct keys: temper help
weekly-budget. Settings do not resize budgets; policy 1 sizes future
projects. Read balances with `project show PROJECT`.

If not: `temper ctl: the sign-in link was refused` (exit 1): spent or
expired. Use the new link after `A fresh sign-in link:`.

## 4. Set up Forgejo on this machine

Setup adopts into the project `me` lists, 1 on a new box.

```
temper ctl DIR forge setup --org work --repository hello --project PROJECT --key KEY
```

Expect: `setting up`, then `project: PROJECT · NAME`, the project
and its name, then `Adopted work/hello on forge FORGE as owned.` and
`It is the project's home.`

Each Forgejo start prints its API and your login:

```
Forgejo: api http://127.0.0.1:PORT/api/v1
Forgejo: person LOGIN
```

Headers: `person_read.header` and `person_write.header` in
`DATA/forge-FORGE/`; use `curl -H @PATH` to keep tokens private.
`person_password` holds your first password; changing it is optional.

If not: `not met: timeout` (exit 3) after 300 seconds while fetching:
run `temper ctl DIR forge list --until made:FORGE`, then continue.
`temper ctl DIR forge list` gives failed fetches' or starts' reasons.

## 5. Give the model an account

Choose one way. `me` lists providers; init makes OpenAI provider 1.

A login from another deployment:

```
temper ctl DIR account import FILE --key KEY
```

Browser sign-in (any HTTP client completes the fake):

```
temper ctl DIR account connect PROVIDER --key KEY
```

An API key in a private file; on fake backends, the printed model key file:

```
temper ctl DIR account add-key PROVIDER FILE --deployment --key KEY
```

A Claude Code login, on step 1's Claude Code configuration;
`ACCOUNT` is the account `account list` shows for provider 2:

```
temper ctl DIR account declare 2 claude_code NAME --key KEY
temper program-login --config CONFIG --data DATA --cache CACHE NAME
temper ctl DIR account list
temper ctl DIR account verify ACCOUNT --source program
```

`program-login` signs Claude Code into `PROGRAMS/NAME` here.
For an existing login, use its parent as `PROGRAMS`, its name as `NAME`,
and skip `program-login`. Only one box uses it at a time.

```
temper ctl DIR account list --until usable
```

Expect: `moved in: account N`, an address, `done: add-key`, or
`Program source S: NAME`, then `asked` for verify; the wait ends `met`.

If not: `key file mode 0o644 permits group or other access` (exit 1):
`chmod 600 FILE`, then the same call with the same key. The person's
browser is on another machine: provider 1's sign-in calls back to
`127.0.0.1:1455` on this one, which that browser reaches only through
`ssh -L 1455:127.0.0.1:1455 HOST`, HOST this machine; give the person
that line with the address. A wait that ends `not met: timeout`
(exit 3) after a sign-in: the browser did not finish it; open the
address again, then wait again.

## 6. Ask for a change

```
temper ctl DIR chat start --project PROJECT - --key KEY < REQUEST
temper ctl DIR task show TASK --until replied
temper ctl DIR chat show TASK
temper ctl DIR task show TASK --tree
```

`TASK` is the chat; its tree lists `CHANGE`. `chat show` prints the
conversation and commands for earlier pages.

Expect: `done: task TASK, chat`; the wait's `task TASK · chat · `
panel ends `met`; `chat show` prints the reply.

If not: fake backends reply `Hello from the local fake model.`
without a change: go to step 8. `chat exceeds offered bounds`
(exit 1): read `me`.

## 7. Follow the change to its landing

```
temper ctl DIR change show CHANGE --until landed --timeout 3600
temper ctl DIR change show CHANGE
```

The starter CI passes.
The pull request printed by `change show` has its diff, CI runs and
logs; temper's review gate is read with `change show`.

Expect: the change's panel, its first line `change CHANGE ·
work/hello on forge FORGE · `, its branch and `into main`, then `met`;
`change show` then names its step, `landed`, and its merge commit.

If not: `not met: timeout` (exit 3): read `temper ctl DIR change
show CHANGE` and `temper ctl DIR task show CHANGE --tree` for its step, its
hold and its acts, act on them, and wait again. `not met: ended:
cancelled` or `not met: ended: failed` (exit 3), at once: the change
ended without landing; its tree says why. `not met: held` (exit 3):
its `held: task N (REASON)` names the change, its writer or its
reviewer; `temper ctl DIR task show N` prints its way out. `held
before registration: REASON` (exit 2): the change was held before its
pull request was made; `temper ctl DIR task show CHANGE` prints its
way out.

## 8. Export the account, then stop

On real backends, move a signed-in account out before the stop:

```
temper ctl DIR account list
temper ctl DIR account export ACCOUNT FILE
```

`ACCOUNT` is the account whose source is `sign_in`, `granted`;
`FILE` must not exist. Then stop the box with `SIGINT` (Ctrl-C) or
`SIGTERM`.

Expect: `moved out: account ACCOUNT to FILE`; the last line ends `Stopped`.

If not: `account export: not signed in` (exit 2): add a key where needed.
`temper ctl: the engine holds its N answers of the last day; room again
at TIME` (exit 2): no room for the move's answers; nothing moved.
Keep the box running and export again after TIME.
`temper ctl: unsent` (exit 1): busy engine, nothing moved; export later.
Other failures: keep running; the engine keeps the account and cancels
the move at its deadline. Export after it.

## Clean up

- `CONFIG`, `DATA` and `CACHE` hold the deployment: start the box
  on the same three to resume. On fake backends, removing
  `DATA/box/fake-backends` resets it.
- Keep `FILE` from step 8 private until an import on the next
  deployment removes it.
- Remove a key file once its account is usable, and `DIR` when done.

## Next

- temper help ctl: the loop, keys and exit statuses of every call.
- temper help github: GitHub beside this box's Forgejo.
