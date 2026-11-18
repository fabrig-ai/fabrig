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
- `HEX`: `w1.digest`'s hex, not its path.
- `COMMIT`: the build's twelve hex digits, the same on both Linux x86-64
  hosts. On `engine`, set `binaries='temper-engine temper'` (`temper` is
  ctl's client); on `worker`, `binaries='temper-worker temper-agent'`.
  Each:

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

Copy `owned.json` to `CONFIG/temper.json`; change:

- `engine.hosts`: `{"listener":[{"socket":"127.0.0.1:8369"}]}`;
  `workers`: `[]`, an engine alone;
- `application.clients`: `listener` `{"socket":"127.0.0.1:8367"}`,
  `public` `"https://temper.example.com"`;
  `application.webhooks.listener`: `{"socket":"127.0.0.1:8368"}`;
- `engine.owners`: `[{"provider":3,"subject":"sam@example.com"}]`;
  provider 3 is Google's, fixed by temper;
- `application.sign_in`: `{"google":{"provider":3,"client":
  "CLIENT_ID","secret":"secrets/google-client.secret","endpoints":
  null},"members":[{"address":"sam@example.com","login":"sam"}]}`.

Provider 3 owners must be members; edit and restart to add one.

For Forgejo sign-in through temper, set
`engine.clients.provider`: `{"key":"provider.key","clients":[{"id":
"forgejo","secret":"forgejo-client.secret","redirect_uri":
"https://git.example.com/user/oauth2/temper/callback"}],"addresses":
[3]}`, and in `application.sign_in`, `"forges":[{"forge":1,"client":
"forgejo"}]`. Step 3's forge preparation creates key and secret and prints this command;
run once after step 4:

```
temper-engine forge sign-in --config CONFIG --data DATA --cache CACHE 1
```

`engine.memory_bytes` budgets held transcripts/turns, not process memory
(example: 1 GiB, `1073741824`). At the bound, due tasks await earlier turns'
saves, even after restart. `0` alone refuses start:
`engine.memory_bytes: past its bound` (temper help configuration).

More runs: `temper ctl DIR worker places w1 PLACES --key KEY`;
keep the worker's `workers[0].slots` at least that large. Places apply at
its next start, from its engine's figures. Set engine limits;
starts name short ones. Each slot is an agent process, up to `agents`'
`declared.memory`.

Expect: valid JSON; not running yet.

If not: step 3 names refused members.

## 2. Keep the secrets

As `temper`, enter Google's client secret without echo, chat or copying:

```
sudo -u temper bash -c 'umask 077; mkdir -p DATA/secrets; IFS= read -rsp "Google client secret: " s; echo; printf %s "$s" > DATA/secrets/google-client.secret; unset s'
```

On `worker`, make data's `secrets/w1.secret`; copy SHA-256 of its digits
without the line end for the engine's worker setting:

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

## 3. Start Forgejo, then engine

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
- `temper-worker serve`: three dirs. Example's `workers[0]`:
  `engine` `{"socket":"127.0.0.1:8369"}`, `name` `"w1"`,
  `secret` `"secrets/w1.secret"`; top-level only `workers` and
  `application`. Remove
  `application.workers`, worker `limits` and `agent`,
  `application.limits` and config's `agent.json`: refused as moved.
  Each start: fetch engine figures, apply limits, write private
  `ROOT/agent/agent.json`, runs executable sibling `temper-agent`.
  No `claude` member: worker PATH's `claude`, else fetched.
  Step 3 unit: `KillMode=control-group` (no agent or command outlives it),
  `TimeoutStopSec=360s`: its 326-second stop bound plus 34 seconds.
  Raise this timeout with the worker's bound.
  `ReadWritePaths` its home; without `PrivateTmp=yes`, Claude Code fails
  `EROFS: read-only file system, mkdir '/tmp/claude-UID'`.
  `application.environment`: optional list, not a map; unique names:
  `[{"name":"PATH","value":"/usr/bin:/bin"}]` (temper help box).
  Absent: the worker's own `HOME` and `PATH`. A list gives exactly
  its entries, with the worker's own `HOME` and `PATH` added where it omits
  them; no other process variables.
- Forgejo runner 13.2.1: separate user, `url` `http://127.0.0.1:3000`,
  `uuid`/secret: `DATA/forge-1/runner/`; rootless podman jobs.

```
temper-worker serve --config CONFIG --data DATA --cache CACHE
```

Expect: stderr: `figures taken from ENGINE: P places, L limits, E endpoints`,
then `linked w1` and engine address; Forgejo site administration: runner online.

If not: short limits refuse before listening, naming limit, setting and
required size: `engine.limits.overrides.core.policy_bytes:
core.policy_bytes is 100; the stock's policy 1 needs 2390 bytes; raise
it in engine.limits.overrides`; raise it and restart.

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

Add the worker with step 2's `worker add` command after signing in,
including after a store reset. Until its setting exists, the worker
redials with `not-admitted`; adding it needs no engine restart.

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
Before stopping, export each `sign_in` or `granted` account:

```
temper ctl DIR account list
temper ctl DIR account export ACCOUNT FILE
```

Stop units; on both hosts fetch, check, install and unpack the new commit
as above; edit documents. Keep `DATA/secrets/`, `DATA/forge-1/` and, for
Forgejo's sign-in through temper, the provider's key and client's secret;
delete the rest of `DATA`. On `worker`, keep `secrets/` alone.
Start Forgejo, engine, worker and runner; sign in and connect the forge
again (step 6); then:

```
temper ctl DIR account import FILE --key KEY
```

Expect: `moved out: account ACCOUNT to FILE`; after start,
`moved in: account ACCOUNT`; `FILE` removed.

If not: `the engine holds its N answers of the last day; room again at
TIME` (exit 2) or `unsent` (exit 1) moved nothing: keep `DATA`, export
later, then remove `DATA`. A refused or lost export keeps the account; resetting loses it.
A refused import keeps `FILE`. Add keys again after reset.

## Clean up

- `temper ctl DIR sign-out`, then remove `DIR`.
- `stat` an exported `FILE` only; never print it or copy it off `engine`.

## Next

- temper help ctl: calls and exit statuses.
- temper help forgejo: a Forgejo temper does not run.
