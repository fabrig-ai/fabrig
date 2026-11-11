# Run temper on a server

Kind: walk

Run a long-lived deployment on a server: the engine behind a TLS proxy with Google sign-in, an owned Forgejo beside it, the workers on a second host, and a redeploy that keeps the model's accounts.

## Before you start

- **Two hosts:** `engine`, which faces the world (the proxy, the
  engine, Forgejo), and `worker` (the worker, Forgejo's runner), joined
  by WireGuard. On `engine`, a service user `temper`, home
  `/var/lib/temper` (0700, no login shell): `CONFIG`, `DATA` and
  `CACHE` are its `config/`, `data/` and `cache/`.
- **Two names,** one for temper and one for Forgejo, here
  `temper.example.com` and `git.example.com`.
- **Google's client:** a project whose OAuth consent is in Testing,
  your members its test users, the scopes `openid`, `email` and
  `profile`; a Web application client whose one redirect URI is
  `https://temper.example.com/sign-in/google/return`.
- **The binary,** built at the commit you deploy, with its browser:
  `TEMPER_INTERNAL_BUNDLE=embed cargo build --release --locked -p temper-box`,
  which needs Rust's `wasm32-unknown-unknown` target; copy
  `target/release/temper-box`, `docs/examples/owned.json` and
  `docs/examples/owned/` of the same commit to both hosts.

## 1. Write the engine's configuration

In `CONFIG/temper.json`, change these from `owned.json`:

- `engine.hosts`: `{"workers": [{"name": "w1", "digest":
  "secrets/w1.digest"}], "listener": [{"socket": "127.0.0.1:8369"}]}`;
  `workers` and `application.workers`: `[]`, an engine alone;
- `application.clients`: `listener` `{"socket": "127.0.0.1:8367"}`,
  `public` `"https://temper.example.com"`;
  `application.webhooks.listener`: `{"socket": "127.0.0.1:8368"}`;
- `application.machine.owned`, the owned Forgejo's: `listen`
  `{"socket": "127.0.0.1:3000"}`, `remote` `"http://127.0.0.1:3000"`,
  `forgejo` `"fetched"`, `runner` `{"binary": "fetched", "labels":
  ["w1-jobs:docker://docker.io/library/debian:trixie-slim"],
  "capacity": 1, "path": "/usr/bin:/bin"}` or `null` for no CI,
  `person` your Forgejo login and address, `public`
  `"https://git.example.com"`;
- `engine.providers` gains `{"number": 3, "kind": "application"}`,
  Google's; `engine.owners`: `[{"provider": 3, "subject":
  "sam@example.com"}]`;
- `application.sign_in`: `{"google": {"provider": 3, "client":
  "CLIENT_ID", "secret": "secrets/google-client.secret", "endpoints":
  null}, "members": [{"address": "sam@example.com", "login": "sam"}]}`.

Every provider 3 owner is a member; edit this file and restart to add one.

For Forgejo sign-in through temper, add
`engine.clients.provider`: `{"key": "provider.key", "clients": [{"id":
"forgejo", "secret": "forgejo-client.secret", "redirect_uri":
"https://git.example.com/user/oauth2/temper/callback"}], "addresses":
[3]}`, and in `application.sign_in`, `"forges": [{"forge": 1, "client":
"forgejo"}]`. Before step 3, this creates key and secret and prints the command
to run once after step 4:

```
temper-box forge sign-in --config CONFIG --data DATA --cache CACHE 1
```

`engine.memory_bytes`, 1 GiB (`1073741824`) in `owned.json`, budgets
held transcripts and turns, never process memory. At or past it, due tasks
wait to start runs until earlier runs reach a worker and their turns are
saved, also after restart. People's reads, calls starting no run and
started runs' turns never wait. A start refuses only `0`:
`engine.memory_bytes: past its bound`. Far above held bytes it rarely
binds; small budgets start runs one at a time. Older documents keep their
figure; workers' `memory_bytes` are read and unused.

To run more at once, raise `workers[0].slots` with the worker's `hub.slots`,
the engine's `fleet.slots`, and `checkout.workspaces` and
`git_component.operations`: a start names any that is short
(`temper help configuration`). Each slot is an agent process, up to
`agent.json`'s `declared.memory`.

Expect: valid JSON; nothing runs yet.

If not: step 3 names the refused member.

## 2. Keep the secrets

As `temper`, type Google's client secret without echo, never through
chat or a copied file:

```
sudo -u temper bash -c 'umask 077; mkdir -p DATA/secrets; IFS= read -rsp "Google client secret: " s; echo; printf %s "$s" > DATA/secrets/google-client.secret; unset s'
```

The worker's secret on `worker`, in its data's `secrets/w1.secret`, and
its digest, the SHA-256 of its digits without the line end, copied to
`engine`'s `DATA/secrets/w1.digest`:

```
umask 077; s=$(od -An -N32 -tx1 /dev/urandom | tr -d ' \n'); printf '%s\n' "$s" > secrets/w1.secret; printf %s "$s" | sha256sum | cut -d' ' -f1 > w1.digest; unset s
```

Expect: `stat -c '%U %a' FILE` prints `temper 600` for each.

If not: temper refuses a secret readable by another user or a link:
give it to `temper`, mode 0600.

## 3. Start Forgejo, then the engine

Units under `temper`, each with `UMask=0077`, `ProtectSystem=strict`,
`PrivateTmp=yes`, `ReadWritePaths=/var/lib/temper`, `NoNewPrivileges=yes`.
`strict` leaves `/tmp` read-only without `PrivateTmp`. Forgejo's:

```
temper-box forge fetch --config CONFIG --data DATA --cache CACHE 1
temper-box forge prepare --config CONFIG --data DATA --cache CACHE 1
temper-box forge run --config CONFIG --data DATA --cache CACHE 1 forgejo
```

The engine connects kept forges at start; wait until
`http://127.0.0.1:3000/api/v1/version` answers:

```
temper-box serve --config CONFIG --data DATA --cache CACHE
```

Expect: `fetch` says how far each binary has gone, `fetched BINARY DONE
of TOTAL bytes`, and nothing for one already there; `prepare` prints
nothing; each exits 0. `run` is Forgejo's own output; `serve` prints its
sign-in link,
`https://temper.example.com/jig/v1/sign-in/link?secret=` and its
secret, and on stderr `listening clients 127.0.0.1:8367`.
With `application.sign_in.forges` on a new store, `serve`'s stderr says
`temper-box serve: forge 1: no sign-in through temper-box: no such forge is kept`
until step 6 connects it.

If not: a refused start names its member, file, address, or restore
refusal (`temper-box serve: restore KIND: REASON`,
`temper-box serve: restart refused during PHASE: REASON`, the reason naming
the store's cause and the record's family and key when it has one): keep
`DATA` unchanged and report that finding.

## 4. The TLS proxy

Caddy, the only listener facing the world, on 80 and 443:

```
git.example.com {
    reverse_proxy 127.0.0.1:3000
}
temper.example.com {
    @link path /jig/v1/sign-in/link /jig/v1/sign-in/link/*
    respond @link 403
    reverse_proxy 127.0.0.1:8367 {
        flush_interval -1
        transport http {
            read_timeout 10m
        }
    }
}
```

Keep the `Host` header: temper sets `Secure` on its cookies by it.
Add no retry.

Expect: `https://temper.example.com/` answers 200; a `POST` to
`/jig/v1/sign-in/link` answers 403; `/sign-in/google` answers 303 to
`accounts.google.com` with your redirect URI.

If not: 502 means the engine is down or listening elsewhere; read its journal.

## 5. The workers' host

- WireGuard: `engine` at `10.88.0.1/32` on UDP 51820, `worker` at
  `10.88.0.5/32` dialling it; `AllowedIPs` the other's /32 alone.
- Forwarders, `systemd-socket-proxyd`: on `engine`, `10.88.0.1:8369`
  and `10.88.0.1:3000` to the same ports on 127.0.0.1, with
  `FreeBind=yes`, admitted from `10.88.0.5` alone; on `worker`,
  `127.0.0.1:8369` and `127.0.0.1:3000` to `10.88.0.1`.
- The worker alone: `temper-box serve` on `worker`'s own three
  directories, its document the example's worker part, `workers[0]`
  with `engine` `{"socket": "127.0.0.1:8369"}`, `name` `"w1"`,
  `secret` `"secrets/w1.secret"`, `agent` naming the binary and
  `agent.json` by absolute paths; its unit as step 3's,
  `ReadWritePaths` its own home: without `PrivateTmp=yes` a Claude Code
  run fails `EROFS: read-only file system, mkdir '/tmp/claude-UID'`.
  `agent.json`'s `environment` is a list of entries,
  `[{"name": "PATH", "value": "/usr/bin:/bin"}]`, not a map
  (temper help box).
- The runner, Forgejo runner 13.2.1 under a user of its own, its
  `url` `http://127.0.0.1:3000`, its `uuid` and secret those of
  `DATA/forge-1/runner/`; its jobs in rootless podman.

```
temper-box serve --config CONFIG --data DATA --cache CACHE
```

Expect: `worker`'s stderr says `linked w1` and the engine's address;
Forgejo's site administration lists the runner online.

If not: a limit below what the stock's settings need is refused
before any listener opens, naming the limit, the setting and the size
it needs: `engine.limits.overrides.core.policy_bytes:
core.policy_bytes is 100; the stock's policy 1 needs 2390 bytes; raise
it in engine.limits.overrides`; raise that override and start again.

For a task held `out of tries: run failures`, the engine's journal names
each run's failure,
`temper-box serve: task TASK run RUN attempt ATTEMPT failed CLASS: DETAIL`:

```
journalctl -u temper -o cat | grep 'task TASK run'
```

## 6. Sign in, through a tunnel, and connect the forge

The proxy refuses the link's public route: give ctl the listener's
origin instead, on `engine` as `temper`:

```
journalctl -u temper -o cat | grep -o 'https://temper.example.com/jig/v1/sign-in/link?[^ ]*' | tail -1 | sed 's#^https://temper.example.com#http://127.0.0.1:8367#' | temper ctl DIR sign-in -
```

In a browser, open the same link through `ssh -L
8367:127.0.0.1:8367 engine`.

Connect step 3's Forgejo. Without a box it has no record yet:
`forge list` is empty and `forge setup` refuses. On a new store it is forge 1:

```
temper ctl DIR forge connect forgejo-admin DATA/forge-1/admin_token --api 127.0.0.1:3000 --git http://127.0.0.1:3000 --web https://git.example.com --key KEY
temper ctl DIR forge list --until connected:1
```

Keep `DATA/forge-1/admin_token` for the next redeploy's connect.

Expect: `party 1 · sam · ` and the party's name: the first of
`engine.owners`. `connecting`; then the wait prints the forges, forge
`1` `managed`, then `met`.
The page's `temper-forge-sign-in` meta appears only once `forge connect`
has kept the forge. If a page fetched after that still lacks the meta,
restart the engine once: a start makes it from the kept record.

If not: `temper ctl: the sign-in link was refused`: it was spent or
expired, and temper printed a fresh one: run the line again.
`connected already` (exit 2): a kept store keeps its forge, and there is
nothing to do. Other refusals: as temper help forgejo says in its step 2.

## 7. Redeploy from main

A redeploy resets the store, keeping Forgejo, secrets and members.
Before stopping, export each account whose source is
`sign_in` and `granted` to a file the reset keeps:

```
temper ctl DIR account list
temper ctl DIR account export ACCOUNT FILE
```

Then stop the units; install the new binary and documents; remove
`DATA` but `DATA/secrets/` and `DATA/forge-1/` and, with Forgejo's sign-in
through temper, the provider's key and the client's secret, and on
`worker` its data but `secrets/`; start Forgejo, the engine, the worker
and the runner; sign in and connect the forge again (step 6); then:

```
temper ctl DIR account import FILE --key KEY
```

Expect: `moved out: account ACCOUNT to FILE`; after the start,
`moved in: account ACCOUNT`, and `FILE` removed.

If not: an export answered `the engine holds its N answers of the last
day; room again at TIME` (exit 2) or `unsent` (exit 1) moved nothing:
keep `DATA`, export again after TIME or later, and remove `DATA` only
once it is out. An export
refused or lost: the engine kept the account, and the store loses it;
an owner signs in to it again. An import refused
keeps `FILE`: leave it as it is. A key's account does not cross: add
its key again. A worker's document from an older `temper-box init` stops the
worker's start at `agent.arguments`,
`"agent" is now "internal", "agent"`: write them
`["internal", "agent", PATH]`, `PATH` its `agent.configuration`.
Before 10-27, `account list` prints ctl's usage (exit 1): before stopping that build, use `accounts` and `export`.

## Clean up

- `temper ctl DIR sign-out`, then remove `DIR`.
- Check an exported `FILE` by `stat` alone; never print it or copy
  it off `engine`.

## Next

- temper help ctl: the loop, keys and exit statuses for every call.
- temper help forgejo: a Forgejo temper does not run.
