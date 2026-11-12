# Run temper on a server

Kind: walk

Run a long-lived deployment on a server: the engine behind a TLS proxy with Google sign-in, an owned Forgejo beside it, the workers on a second host, and a redeploy that keeps the model's accounts.

## Before you start

- **Two hosts:** `engine` (public proxy, engine, Forgejo) and `worker`
  (worker, Forgejo's runner), joined by WireGuard. On `engine`, user
  `temper`, home `/var/lib/temper` (0700, no login shell): `CONFIG`,
  `DATA`, `CACHE` are its `config/`, `data/`, `cache/`.
- **Names:** `temper.example.com` for temper, `git.example.com` for Forgejo.
- **Google's client:** OAuth consent in Testing, members its test users,
  scopes `openid`, `email`, `profile`; a Web application client with one redirect URI
  `https://temper.example.com/sign-in/google/return`.
- `COMMIT`: the build's twelve hex digits. Use one commit's set on both
  Linux x86-64 hosts: on `engine`, `temper-engine` and `temper` (the
  client, for ctl); on `worker`, `temper-worker` and `temper-agent`.
  Set `binaries='temper-engine temper'` on `engine`, or
  `binaries='temper-worker temper-agent'` on `worker`, then on each:

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

Binaries are gzipped without a suffix; keep this layout for checksums. Stripping the archive's
`temper/docs/examples/` prefix leaves `owned.json` and `owned/`.

## 1. Write the engine's configuration

Copy the unpacked `owned.json` to `CONFIG/temper.json` and `owned/`'s
files to `CONFIG/`; change these:

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
temper-engine forge sign-in --config CONFIG --data DATA --cache CACHE 1
```

`engine.memory_bytes` (1 GiB, `1073741824` in the example) budgets held
transcripts and turns, not process memory. At its bound, due tasks wait
for earlier turns to be saved, also after restart. Only `0` refuses a
start: `engine.memory_bytes: past its bound` (temper help configuration).

For more runs, raise `workers[0].slots`, `hub.slots`, `fleet.slots`,
`checkout.workspaces` and `git_component.operations`. A start names short
limits. A slot is an agent process, up to `agent.json`'s `declared.memory`.

Expect: valid JSON; nothing runs yet.

If not: step 3 names the refused member.

## 2. Keep the secrets

As `temper`, type Google's client secret without echo, never through
chat or a copy:

```
sudo -u temper bash -c 'umask 077; mkdir -p DATA/secrets; IFS= read -rsp "Google client secret: " s; echo; printf %s "$s" > DATA/secrets/google-client.secret; unset s'
```

On `worker`, make its data's `secrets/w1.secret`; copy its digest,
SHA-256 of the digits without the line end, to `engine`'s
`DATA/secrets/w1.digest`:

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
temper-engine forge fetch --config CONFIG --data DATA --cache CACHE 1
temper-engine forge prepare --config CONFIG --data DATA --cache CACHE 1
temper-engine forge run --config CONFIG --data DATA --cache CACHE 1 forgejo
```

Wait for `http://127.0.0.1:3000/api/v1/version` before starting the
engine, which connects kept forges:

```
temper-engine serve --config CONFIG --data DATA --cache CACHE
```

Expect: `fetch` prints `fetched BINARY DONE of TOTAL bytes`, nothing for
one already there; `prepare` prints nothing; both exit 0. `run` prints
Forgejo's output; `serve` prints its sign-in link,
`https://temper.example.com/jig/v1/sign-in/link?secret=` and its
secret, and on stderr `listening clients 127.0.0.1:8367`.
With `application.sign_in.forges` on a new store, `serve`'s stderr says
`temper-engine serve: forge 1: no sign-in through temper-engine: no such forge is kept`
until step 6 connects it.

If not: a refused start names its member, file, address or restore:
`temper-engine serve: restore KIND: REASON` or
`temper-engine serve: restart refused during PHASE: REASON`.
The reason names the store's cause, record family and key when available.
Keep `DATA` unchanged and report it.

## 4. The TLS proxy

Caddy alone faces the world, on 80 and 443:

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

Keep `Host`: it sets cookies' `Secure`. Add no retry.

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
- The worker alone: `temper-worker serve` on `worker`'s own three
  directories, its document the example's worker part, `workers[0]`
  with `engine` `{"socket": "127.0.0.1:8369"}`, `name` `"w1"`,
  `secret` `"secrets/w1.secret"`; omit `engine`, keep only `workers`
  and `application`'s `workers` and `limits`. Set `agent.program` to
  `"/usr/local/bin/temper-agent"`, `agent.configuration` to the absolute
  path of its `agent.json`, and `agent.arguments` to a one-element list
  holding that same path. Its unit as step 3's, with
  `KillMode=control-group` so no agent or command outlives the worker,
  and `TimeoutStopSec=240s`: above its 186-second stop bound with a
  54-second margin. Raising the worker's bound raises this timeout too.
  `ReadWritePaths` its own home: without `PrivateTmp=yes` a Claude Code
  run fails `EROFS: read-only file system, mkdir '/tmp/claude-UID'`.
  `agent.json`'s `environment` is a list of entries,
  `[{"name": "PATH", "value": "/usr/bin:/bin"}]`, not a map
  (temper help box).
- The runner, Forgejo runner 13.2.1 under a user of its own, its
  `url` `http://127.0.0.1:3000`, its `uuid` and secret those of
  `DATA/forge-1/runner/`; its jobs in rootless podman.

```
temper-worker serve --config CONFIG --data DATA --cache CACHE
```

Expect: `worker`'s stderr says `linked w1` and the engine's address;
Forgejo's site administration lists the runner online.

If not: a limit short of stock settings refuses before listeners open,
naming limit, setting and required size: `engine.limits.overrides.core.policy_bytes:
core.policy_bytes is 100; the stock's policy 1 needs 2390 bytes; raise
it in engine.limits.overrides`; raise that override and start again.

For `out of tries: run failures`, read the engine's journal,
`temper-engine serve: task TASK run RUN attempt ATTEMPT failed CLASS: DETAIL`:

```
journalctl -u temper -o cat | grep 'task TASK run'
```

## 6. Sign in, through a tunnel, and connect the forge

The proxy refuses the link's public route: give ctl the listener's
origin instead, on `engine` as `temper`:

```
journalctl -u temper -o cat | grep -o 'https://temper.example.com/jig/v1/sign-in/link?[^ ]*' | tail -1 | sed 's#^https://temper.example.com#http://127.0.0.1:8367#' | temper ctl DIR sign-in -
```

In a browser, open that link through `ssh -L 8367:127.0.0.1:8367 engine`.

Connect Forgejo: `forge list` is empty and `forge setup` refuses until
it has a record. On a new store it is forge 1:

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

Redeploy resets the store, keeping Forgejo, secrets and members.
Before stopping, export each `sign_in` or `granted` account to a file
the reset keeps:

```
temper ctl DIR account list
temper ctl DIR account export ACCOUNT FILE
```

Then stop the units; fetch, check, install and unpack the new commit's
set on both hosts as in Before you start; edit its documents as above; remove
`DATA` but `DATA/secrets/` and `DATA/forge-1/` and, with Forgejo's sign-in
through temper, the provider's key and the client's secret, and on
`worker` its data but `secrets/`; start Forgejo, the engine, the worker
and the runner; sign in and connect the forge again (step 6); then:

```
temper ctl DIR account import FILE --key KEY
```

Expect: `moved out: account ACCOUNT to FILE`; after the start,
`moved in: account ACCOUNT`, and `FILE` removed.

If not: `the engine holds its N answers of the last day; room again at
TIME` (exit 2) or `unsent` (exit 1) moved nothing: keep `DATA`, export
after TIME or later, remove `DATA` only once it is out. An export refused
or lost kept the account in the engine; resetting loses it, so an owner
signs in again. A refused import keeps `FILE`: leave it. A key's account
does not cross: add its key again.

## Clean up

- `temper ctl DIR sign-out`, then remove `DIR`.
- `stat` an exported `FILE` only; never print it or copy it off `engine`.

## Next

- temper help ctl: the loop, keys and exit statuses for every call.
- temper help forgejo: a Forgejo temper does not run.
