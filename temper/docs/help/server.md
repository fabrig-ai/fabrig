# Run temper on a server

Kind: walk

Run temper long-term: engine behind a TLS proxy with Google sign-in and owned Forgejo, workers on a second host; redeploy keeping model accounts.

## Before you start

- **WireGuard hosts:** `engine` (public proxy, engine, Forgejo), `worker`
  (worker, Forgejo runner). On `engine`: user `temper`, home
  `/var/lib/temper` (0700, no login shell); `CONFIG`, `DATA`, `CACHE`:
  its `config/`, `data/`, `cache/`.
- **Names:** `temper.example.com` for temper, `git.example.com` for Forgejo.
- **Google client:** OAuth consent in Testing, members its test users,
  scopes `openid`, `email`, `profile`; a Web application client with one redirect URI
  `https://temper.example.com/sign-in/google/return`.
<<<<<<< HEAD
- `COMMIT`: same build's 12 hex digits on both Linux x86-64 hosts. On `engine`, set `binaries='temper-engine temper'` (`temper` is
=======
- `HEX`: `w1.digest`'s hex, not its path.
- `COMMIT`: the build's twelve hex digits, the same on both Linux x86-64
  hosts. On `engine`, set `binaries='temper-engine temper'` (`temper` is
>>>>>>> f29ee15eb4ff4fc3e8054b45ce86679e1e771fb2
  ctl's client); on `worker`, `binaries='temper-worker temper-agent'`.
  On each:

```
set -e; set -o pipefail
mkdir -p COMMIT/Linux-x86_64
cd COMMIT
curl -fLO https://fabrig.ai/temper/v0/COMMIT/SHA256SUMS
curl -fLO https://fabrig.ai/temper/v0/COMMIT/examples.tar.gz
for name in $binaries; do curl -fL https://fabrig.ai/temper/v0/COMMIT/Linux-x86_64/"$name" -o Linux-x86_64/"$name"; done
sha256sum -c --ignore-missing SHA256SUMS
for name in $binaries; do cat Linux-x86_64/"$name" | gunzip | sudo tee /usr/local/bin/"$name" > /dev/null; sudo chmod 0755 /usr/local/bin/"$name"; done
tar -xzf examples.tar.gz --strip-components=3
```

Binaries are suffixless gzip; keep the checksum layout. Strip
`temper/docs/examples/` for `owned.json`.

## 1. Write the engine's configuration

<<<<<<< HEAD
Copy `owned.json` to `CONFIG/temper.json`; change:
=======
Copy `owned.json` to `CONFIG/temper.json`, `owned/` files to `CONFIG/`; edit:
>>>>>>> f29ee15eb4ff4fc3e8054b45ce86679e1e771fb2

- `engine.hosts`: `{"listener":[{"socket":"127.0.0.1:8369"}]}`;
  `workers` and `application.workers`: `[]`, an engine alone;
- `application.clients`: `listener` `{"socket": "127.0.0.1:8367"}`,
  `public` `"https://temper.example.com"`;
  `application.webhooks.listener`: `{"socket": "127.0.0.1:8368"}`;
- `engine.owners`: `[{"provider": 3, "subject": "sam@example.com"}]`;
  provider 3 is Google's, fixed by temper;
- `application.sign_in`: `{"google": {"provider": 3, "client":
  "CLIENT_ID", "secret": "secrets/google-client.secret", "endpoints":
  null}, "members": [{"address": "sam@example.com", "login": "sam"}]}`.

Provider 3 owners must be members; edit and restart to add one.

For Forgejo sign-in through temper, set
`engine.clients.provider`: `{"key": "provider.key", "clients": [{"id":
"forgejo", "secret": "forgejo-client.secret", "redirect_uri":
"https://git.example.com/user/oauth2/temper/callback"}], "addresses":
[3]}`, and in `application.sign_in`, `"forges": [{"forge": 1, "client":
"forgejo"}]`. After preparing the forge in step 3, this creates key and secret and prints the command
to run once after step 4:

```
temper-engine forge sign-in --config CONFIG --data DATA --cache CACHE 1
```

`engine.memory_bytes` budgets held transcripts/turns, not process memory
(example: 1 GiB, `1073741824`). At the bound, due tasks await earlier turns'
saves, even after restart. `0` alone refuses start:
`engine.memory_bytes: past its bound` (temper help configuration).

<<<<<<< HEAD
For more runs, raise `workers[0].slots`, `hub.slots`, `fleet.slots`,
`checkout.workspaces` and `git_component.operations`. A start names short
limits. A slot is an agent process, up to the `agents` setting's `declared.memory`.
=======
More runs: raise `workers[0].slots`, `hub.slots`, `fleet.slots`,
`checkout.workspaces` and `git_component.operations`. Starts name short limits.
Each slot: an agent process, at most `agent.json`'s `declared.memory`.
>>>>>>> f29ee15eb4ff4fc3e8054b45ce86679e1e771fb2

Expect: valid JSON; nothing runs yet.

If not: step 3 names refused members.

## 2. Keep the secrets

<<<<<<< HEAD
As `temper`, type Google's secret without echo, chat or copying:
=======
As `temper`, enter Google's client secret without echo, chat or copying:
>>>>>>> f29ee15eb4ff4fc3e8054b45ce86679e1e771fb2

```
sudo -u temper bash -c 'umask 077; mkdir -p DATA/secrets; IFS= read -rsp "Google client secret: " s; echo; printf %s "$s" > DATA/secrets/google-client.secret; unset s'
```

On `worker`, make data's `secrets/w1.secret`; copy SHA-256 of its digits
<<<<<<< HEAD
(no line end) to `engine`'s `DATA/secrets/w1.digest`:
=======
without the line end for the engine's worker setting:
>>>>>>> f29ee15eb4ff4fc3e8054b45ce86679e1e771fb2

```
umask 077; s=$(od -An -N32 -tx1 /dev/urandom | tr -d ' \n'); printf '%s\n' "$s" > secrets/w1.secret; printf %s "$s" | sha256sum | cut -d' ' -f1 > w1.digest; unset s
```

After engine sign-in, add without restart:

```
temper ctl DIR worker add w1 --places 4 --digest HEX --key KEY
```

Workers are store settings; `engine.hosts.workers` is refused.
Places apply at next opening.

Expect: `stat -c '%U %a' FILE` prints `temper 600` for each.

If not: temper refuses a secret readable by another user or a link:
give it to `temper`, mode 0600.

## 3. Start Forgejo, then the engine

Units run as `temper` with `UMask=0077`, `ProtectSystem=strict`,
`PrivateTmp=yes`, `ReadWritePaths=/var/lib/temper`, `NoNewPrivileges=yes`.
Without `PrivateTmp`, `strict` makes `/tmp` read-only. Forgejo:

```
temper-engine forge prepare --config CONFIG --data DATA --cache CACHE 1 --listen 127.0.0.1:3000 --public https://git.example.com --runner-label w1-jobs:docker://docker.io/library/debian:trixie-slim --runner-path /usr/bin:/bin --person sam sam@example.com
temper-engine forge run --config CONFIG --data DATA --cache CACHE 1 forgejo
```

Wait for `http://127.0.0.1:3000/api/v1/version` before starting the
engine, which connects kept forges:

```
temper-engine serve --config CONFIG --data DATA --cache CACHE
```

Expect: `prepare` fetches pinned binaries, printing `fetched BINARY DONE of
TOTAL bytes`, nothing if present, and exits 0. `run` prints Forgejo's
output; `serve` prints its sign-in link,
`https://temper.example.com/jig/v1/sign-in/link?secret=` and its
secret, and on stderr `listening clients 127.0.0.1:8367`.
With `application.sign_in.forges` on a new store, `serve`'s stderr says
`temper-engine serve: forge 1: no sign-in through temper-engine: no such forge is kept`
until step 6 connects it.

If not: refused start names member, file, address or restore:
`temper-engine serve: restore KIND: REASON` or
`temper-engine serve: restart refused during PHASE: REASON`.
The reason names store cause, record family and key if available.
Keep `DATA` unchanged; report it.

## 4. The TLS proxy

Caddy alone faces the world on 80/443:

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

Keep `Host` for cookies' `Secure`; no retries.

Expect: `https://temper.example.com/` answers 200; a `POST` to
`/jig/v1/sign-in/link` answers 403; `/sign-in/google` answers 303 to
`accounts.google.com` with your redirect URI.

If not: 502: engine down or listening elsewhere; read its journal.

## 5. The workers' host

- WireGuard: `engine` at `10.88.0.1/32` on UDP 51820, `worker` at
  `10.88.0.5/32` dialling it; `AllowedIPs` the other's /32 alone.
- Forwarders, `systemd-socket-proxyd`: on `engine`, `10.88.0.1:8369`
  and `10.88.0.1:3000` to the same ports on 127.0.0.1, with
  `FreeBind=yes`, admitted from `10.88.0.5` alone; on `worker`,
  `127.0.0.1:8369` and `127.0.0.1:3000` to `10.88.0.1`.
<<<<<<< HEAD
- `temper-worker serve`: three dirs. Use example's
  `workers[0]` with `engine` `{"socket":"127.0.0.1:8369"}`, `name` `"w1"`,
  `secret` `"secrets/w1.secret"`; top-level only `workers` and
  `application.workers`. Remove worker `limits` and `agent`,
  `application.limits` and config's `agent.json`: refused as moved.
  Each start: fetch engine figures, apply limits, write private
  `ROOT/agent/agent.json`, runs executable sibling `temper-agent`.
  `claude`: optional absolute executable; else PATH `claude` or fetched.
  Step 3 unit: `KillMode=control-group` (no agent or command outlives it),
  `TimeoutStopSec=360s`: its 326-second stop bound plus 34 seconds.
  Raise this timeout with the worker's bound.
  `ReadWritePaths` its home; without `PrivateTmp=yes`, Claude Code fails
  `EROFS: read-only file system, mkdir '/tmp/claude-UID'`.
  Entry `environment`: optional list, not a map; unique names:
  `[{"name":"PATH","value":"/usr/bin:/bin"}]` (temper help box).
- Forgejo runner 13.2.1: separate user, `url` `http://127.0.0.1:3000`,
  `uuid`/secret: `DATA/forge-1/runner/`; rootless podman jobs.
=======
- Worker: three directories for `temper-worker serve`. Keep only top-level
  `workers` and `application`'s `workers` and `limits`; omit top-level `engine`.
  From the example's `workers[0]`, set `engine`: `{"socket":"127.0.0.1:8369"}`,
  `name`: `"w1"`, `secret`: `"secrets/w1.secret"`,
  `agent.program`: `"/usr/local/bin/temper-agent"`, `agent.configuration`: absolute
  `agent.json` path, `agent.arguments`: `[that path]`.
  Use step 3's unit with `KillMode=control-group` (no surviving agent/command)
  and `TimeoutStopSec=360s` (326-second stop bound plus 34 seconds).
  Raise this timeout with the bound. `ReadWritePaths`: its own home.
  Without `PrivateTmp=yes`, Claude Code fails
  `EROFS: read-only file system, mkdir '/tmp/claude-UID'`.
  `agent.json`'s `environment` is a list, not a map:
  `[{"name":"PATH","value":"/usr/bin:/bin"}]` (temper help box).
- The runner, Forgejo runner 13.2.1 under a user of its own, its
  `url` `http://127.0.0.1:3000`, its `uuid` and secret those of
  `DATA/forge-1/runner/`; its jobs in rootless podman.
>>>>>>> f29ee15eb4ff4fc3e8054b45ce86679e1e771fb2

```
temper-worker serve --config CONFIG --data DATA --cache CACHE
```

<<<<<<< HEAD
Expect: stderr: `figures taken from ENGINE: P places, L limits, E endpoints`,
then `linked w1` and engine address; Forgejo site administration: runner online.
=======
Expect: `worker` stderr: `linked w1` and the engine address;
runner online in Forgejo site administration.
>>>>>>> f29ee15eb4ff4fc3e8054b45ce86679e1e771fb2

If not: limits below stock refuse before listening, naming limit,
setting and required size: `engine.limits.overrides.core.policy_bytes:
core.policy_bytes is 100; the stock's policy 1 needs 2390 bytes; raise
it in engine.limits.overrides`; raise that override and start again.

For `out of tries: run failures`, read the engine's journal,
`temper-engine serve: task TASK run RUN attempt ATTEMPT failed CLASS: DETAIL`:

```
journalctl -u temper -o cat | grep 'task TASK run'
```

## 6. Sign in, through a tunnel, and connect the forge

The proxy refuses the link's public route. Give ctl the listener's
origin, on `engine` as `temper`:

```
journalctl -u temper -o cat | grep -o 'https://temper.example.com/jig/v1/sign-in/link?[^ ]*' | tail -1 | sed 's#^https://temper.example.com#http://127.0.0.1:8367#' | temper ctl DIR sign-in -
```

Browser: open that link through `ssh -L 8367:127.0.0.1:8367 engine`.

Connect Forgejo: without a record, `forge list` is empty; `forge setup`
refuses. A new store's forge is 1:

```
temper ctl DIR forge connect forgejo-admin DATA/forge-1/admin_token --api 127.0.0.1:3000 --git http://127.0.0.1:3000 --web https://git.example.com --key KEY
temper ctl DIR forge list --until connected:1
```

Keep `DATA/forge-1/admin_token` for redeploy's connect.

Expect: `party 1 · sam · ` and the first `engine.owners` party's name.
`connecting`; the wait prints forges, `1` `managed`, then `met`.
The page's `temper-forge-sign-in` meta needs a kept `forge connect`
record. If a fresh page lacks it, restart once to make it from that record.

If not: a signed-in directory refuses before sending; the link stays unspent.
Run `temper ctl DIR sign-out` first, or use another directory.
Sign-out empties even an engine-refused token; no answer keeps it (exit 1).
`temper ctl: the sign-in link was refused` distinguishes `it was already
used`, `it has expired`, and `temper does not know it, as another
deployment's link or one it no longer remembers`; each asks for a fresh
link from `temper-box serve`: get one and rerun. Other refusals keep the
line without a reason.
`connected already` (exit 2): the store keeps its forge; nothing to do.
Other refusals: temper help forgejo, step 2.

## 7. Redeploy from main

Redeploy resets the store, keeping Forgejo, secrets and members.
Before stopping, export every `sign_in` or `granted` account to a file
retained across reset:

```
temper ctl DIR account list
temper ctl DIR account export ACCOUNT FILE
```

Stop units; fetch, check, install and unpack the new commit on both hosts
as in Before you start; edit documents as above; remove
`DATA` but `DATA/secrets/` and `DATA/forge-1/` and, with Forgejo's sign-in
through temper, the provider's key and the client's secret, and on
`worker` its data but `secrets/`; start Forgejo, the engine, the worker
and the runner; sign in and connect the forge again (step 6); then:

```
temper ctl DIR account import FILE --key KEY
```

Expect: `moved out: account ACCOUNT to FILE`; after start,
`moved in: account ACCOUNT`; `FILE` removed.

If not: `the engine holds its N answers of the last day; room again at
TIME` (exit 2) or `unsent` (exit 1) moved nothing: keep `DATA`, export
after TIME or later, then remove `DATA`. A refused or lost export keeps
the account in the engine; resetting loses it, requiring owner sign-in.
A refused import keeps `FILE`: leave it. A key's account does not cross:
add its key again.

## Clean up

- `temper ctl DIR sign-out`, then remove `DIR`.
- `stat` an exported `FILE` only; never print it or copy it off `engine`.

## Next

- temper help ctl: the loop, keys and exit statuses for every call.
- temper help forgejo: a Forgejo temper does not run.
