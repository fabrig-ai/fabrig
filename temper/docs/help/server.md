# Run temper on a server

Kind: walk

Run temper long-term: engine behind a TLS proxy with Google sign-in and owned Forgejo, workers on a second host; redeploy keeping model accounts.

## Before you start

- **WireGuard hosts:** `engine` (public proxy, engine, Forgejo), `worker`
  (worker, Forgejo runner). `engine`: `temper`, `/var/lib/temper` (0700,
  no login shell); `CONFIG`, `DATA`, `CACHE`: its `config/`, `data/`, `cache/`.
- **Names:** temper: `temper.example.com`; Forgejo: `git.example.com`.
- **Google client:** OAuth consent: Testing, members: test users;
  scopes `openid`, `email`, `profile`; Web application, sole redirect URI
  `https://temper.example.com/sign-in/google/return`.
- `HEX`: `w1.digest`'s hex, not its path.
- `COMMIT`: build's twelve hex digits, same on both Linux x86-64 hosts.
  `engine`: `binaries='temper-engine temper'` (`temper`: ctl's client);
  `worker`: `binaries='temper-worker temper-agent'`. Each:

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

Binaries: suffixless gzip; keep checksum layout. For `owned.json`, strip
`temper/docs/examples/`.

## 1. Write the engine's configuration

`CONFIG/temper.toml`, from `temper-engine help config`:

```toml
# Engine: lines explained by `temper-engine help config`.
# Ask temper to change everything else.

listen = "127.0.0.1:8367"   # clients' listener behind proxy
public = "https://temper.example.com"   # public origin
workers = "127.0.0.1:8369"   # workers on loopback
owner = "sam@example.com"   # first owner; read only at first start
webhooks = "127.0.0.1:8368"

[sign_in.google]
client = "CLIENT_ID"
secret = "secrets/google-client.secret"   # data directory, mode 0600

[[sign_in.members]]   # each member
address = "sam@example.com"
login = "sam"
```

Google owners must be members; to add: edit, restart.

Forgejo sign-in: `[openid]`: `key = "provider.key"`,
`addresses = [3]`; `[[openid.clients]]`: `id = "forgejo"`,
`secret = "forgejo-client.secret"` and
`redirect = "https://git.example.com/user/oauth2/temper/callback"`.
`[[sign_in.forges]]`: `forge = 1`, `client = "forgejo"`.
Step 3 creates key and secret, prints this command; run once after step 4:

```
temper-engine forge sign-in --config CONFIG --data DATA --cache CACHE 1
```

Held transcripts/turns, not process memory: `transcript_memory` in
`temper-limits` (1 GiB: `1073741824`). At the bound, due tasks await
earlier turns' saves, even after restart. Read: `temper ctl settings show`; apply: `temper ctl settings apply`.

More runs: `temper ctl DIR worker places w1 PLACES --key KEY`.
Places: worker setting, taken at next opening from engine figures.
Engine limits: `limits`; starts name short ones. Each place: an agent
process, up to `agents`' `declared.memory`.

Expect: valid TOML; not running.

If not: step 3 names bad keys.

## 2. Keep the secrets

As `temper`, enter Google's client secret: no echo, chat or copying:

```
sudo -u temper bash -c 'umask 077; mkdir -p DATA/secrets; IFS= read -rsp "Google client secret: " s; echo; printf %s "$s" > DATA/secrets/google-client.secret; unset s'
```

On `worker`, make data's `secrets/worker.secret`; engine's worker setting
takes SHA-256 of its digits without the line end:

```
umask 077; s=$(od -An -N32 -tx1 /dev/urandom | tr -d ' \n'); printf '%s\n' "$s" > secrets/worker.secret; printf %s "$s" | sha256sum | cut -d' ' -f1 > w1.digest; unset s
```

After engine sign-in, add without restart:

```
temper ctl DIR worker add w1 --places 4 --digest HEX --key KEY
```

Workers are store settings; `engine.hosts.workers` is refused.

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

Wait for `http://127.0.0.1:3000/api/v1/version`; start engine to connect
kept forges:

```
temper-engine serve --config CONFIG --data DATA --cache CACHE
```

Engine: `TimeoutStopSec=660s`: its 602-second stop bound
(`temper-engine versions`, `bound stop`) plus 58 seconds. If you raised
`forge.client.lifetime_ms` or `forge.client.clock_margin_ms`, take twice
their sum, plus at least 30 seconds, rounded up to a whole minute.
No `TimeoutStartSec`: start lasts as long as the store's open and its
steps; a step past 30 seconds says what holds it in the journal.

Expect: `prepare` exits 0: missing pinned binaries print `fetched BINARY
DONE of TOTAL bytes`; present ones print nothing. `run`: Forgejo's output.
`serve`: sign-in link `https://temper.example.com/sign-in?secret=` and
secret; stderr: `listening clients 127.0.0.1:8367`.
With `[[sign_in.forges]]` on a new store, `serve`'s stderr says
`temper-engine serve: forge 1: no sign-in through temper-engine: no such forge is kept`
until step 6 connects it.

If not: start refusal names member, file, address or restore:
`temper-engine serve: restore KIND: REASON` or
`temper-engine serve: restart refused during PHASE: REASON`.
Reason: store cause, record family/key if available. Keep `DATA`; report it.

## 4. The TLS proxy

Caddy alone faces 80/443:

```
git.example.com {
    reverse_proxy 127.0.0.1:3000
}
temper.example.com {
    @link path /jig/v1/sign-in/link /jig/v1/sign-in/link/* /sign-in
    respond @link 403
    reverse_proxy 127.0.0.1:8367 {
        flush_interval -1
        transport http {
            read_timeout 10m
        }
    }
}
```

Keep `Host` for cookies' `Secure`; no retries. No `trusted_proxies`: Caddy
overwrites client-sent `X-Forwarded-For` with client's address. Leave it:
temper uses the last entry as terminal approval source.

Expect: `https://temper.example.com/` answers 200;
`POST /jig/v1/sign-in/link` and `GET /sign-in` answer 403;
`/sign-in/google` answers 303 to
`accounts.google.com` with your redirect URI.

If not: 502: engine down or listening elsewhere; read its journal.

## 5. The workers' host

- WireGuard: `engine` at `10.88.0.1/32` on UDP 51820, `worker` at
  `10.88.0.5/32` dialling it; `AllowedIPs` the other's /32 alone.
- Forwarders, `systemd-socket-proxyd`: on `engine`, `10.88.0.1:8369`
  and `10.88.0.1:3000` to the same ports on 127.0.0.1, with
  `FreeBind=yes`, admitted from `10.88.0.5` alone; on `worker`,
  `127.0.0.1:8369` and `127.0.0.1:3000` to `10.88.0.1`.
- Worker `CONFIG/temper.toml` (`temper-worker help config`):

```toml
# Worker: lines explained by `temper-worker help config`.

engine = "127.0.0.1:8369"   # engine's workers listener
name = "w1"   # name engine knows, if not hostname

[environment]   # one line for each variable
PATH = "/usr/bin:/bin"
```

  Secret: data's `secrets/worker.secret`. No worker
  `limits`, `agent` or configuration `agent.json`: starts fetch engine
  figures, apply limits, write private `ROOT/agent/agent.json`, and run
  sibling `temper-agent`. No `claude` member: worker PATH's `claude`, else
  fetched; `[programs]` can name its absolute path.
  Step 3 unit: `KillMode=control-group` (no agent or command outlives it),
  `TimeoutStopSec=360s`: its 326-second stop bound plus 34 seconds.
  Raise this timeout with the worker's bound.
  `ReadWritePaths` its home; without `PrivateTmp=yes`, Claude Code fails
  `EROFS: read-only file system, mkdir '/tmp/claude-UID'`.
  `[environment]`: optional table of strings; unique names (temper help box).
  Absent: the worker's own `HOME` and `PATH`. Otherwise: exactly its entries, with the worker's own `HOME` and `PATH` added where it omits them; no other process variables.
- Forgejo runner 13.2.1: separate user, `url` `http://127.0.0.1:3000`,
  `uuid`/secret: `DATA/forge-1/runner/`; rootless podman jobs.

```
temper-worker serve --config CONFIG --data DATA --cache CACHE
```

Expect: stderr: `figures taken from ENGINE: P places, L limits, E endpoints`,
then `linked w1` and engine address; Forgejo site administration: runner online.

If not: short limits refuse before listening: limit, setting, required
size. `core.policy_bytes` in `limits` must fit stock policy; raise with
`temper ctl settings apply`, restart.

For `out of tries: run failures`, read the engine's journal,
`temper-engine serve: task TASK run RUN attempt ATTEMPT failed CLASS: DETAIL`:

```
journalctl -u temper -o cat | grep 'task TASK run'
```

## 6. Sign in, through a tunnel, and connect the forge

Proxy refuses the link's public page and route. On `engine` as `temper`,
give ctl the listener's origin:

```
journalctl -u temper -o cat | grep -o 'https://temper.example.com/sign-in?[^ ]*' | tail -1 | sed 's#^https://temper.example.com#http://127.0.0.1:8367#' | temper ctl DIR sign-in -
```

Browser: open that link through `ssh -L 8367:127.0.0.1:8367 engine`.

After sign-in (store resets too), use step 2's `worker add`. Until then:
redials with `not-admitted`; no engine restart.

Connect Forgejo: without a record, `forge list` is empty; `forge setup`
refuses. New store: forge 1:

```
temper ctl DIR forge connect forgejo-admin DATA/forge-1/admin_token --api 127.0.0.1:3000 --git http://127.0.0.1:3000 --web https://git.example.com --key KEY
temper ctl DIR forge list --until connected:1
```

Keep `DATA/forge-1/admin_token` for redeploy's connect.

Expect: `party 1 · sam · ` and the first `owner`'s party name.
`connecting`; the wait prints forges, `1` `managed`, then `met`.
The page's `temper-forge-sign-in` meta needs a kept `forge connect`
record. If a fresh page lacks it, restart once to make it from that record.

If not: signed-in directory refuses before sending; link stays unspent.
`temper ctl DIR sign-out` first, or use another directory.
Sign-out empties even engine-refused tokens; no answer keeps it (exit 1).
`temper ctl: the sign-in link was refused` distinguishes `it was already
used`, `it has expired`, and `temper does not know it, as another
deployment's link or one it no longer remembers`; each asks for a fresh
`temper-box serve` link: rerun with one. Other refusals: line only.
`connected already` (exit 2): store keeps its forge; do nothing.
Other refusals: temper help forgejo, step 2.

## 7. Redeploy from main

Redeploy resets store, keeps Forgejo, secrets, members.
Before stop, export each `sign_in` or `granted` account:

```
temper ctl DIR account list
temper ctl DIR account export ACCOUNT FILE
```

Stop units. Both hosts: fetch, check, install, unpack new commit as above;
edit documents. Keep `DATA/secrets/`, `DATA/forge-1/`; for Forgejo sign-in
through temper, keep provider's key and client's secret. Delete other
`DATA`; `worker` keeps only `secrets/`. Start Forgejo, engine, worker,
runner; sign in, reconnect forge (step 6); then:

```
temper ctl DIR account import FILE --key KEY
```

Expect: `moved out: account ACCOUNT to FILE`; after start:
`moved in: account ACCOUNT`; `FILE` removed.

If not: `the engine holds its N answers of the last day; room again at
TIME` (exit 2) or `unsent` (exit 1) moved nothing: keep `DATA`, export
later, then remove `DATA`. Refused or lost export keeps the account;
resetting loses it. Refused import keeps `FILE`. Add keys after reset.

## Clean up

- `temper ctl DIR sign-out`, then remove `DIR`.
- `stat` an exported `FILE` only; never print it or copy it off `engine`.

## Next

- temper help ctl: calls and exit statuses.
- temper help forgejo: a Forgejo temper does not run.
