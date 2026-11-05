+++
id = "server-page"
part = "server-page"
kind = "docs"
needs = []
files = ["temper/docs/help/server.md"]
effort = "medium"
minutes = 30
+++

# `temper help server` says what the walk needs

## Goal

The page's walk leaves out what its reader must do. Add the connect (finding 3), `PrivateTmp=yes` and where a run's failure is read (finding 4), and correct what the design and code contradict. This task's edits are given below: apply them; the page's voice is kept. This departs from your text, which leaves `docs/help/` alone: this page is yours.

This task waits for no code: `kept-metas` and `no-group-file` change what the page describes, and the page is written for when they have landed. The held task's own line does not change here: the plan's question `hold-last-failure` is the user's.

Acceptance: the page says each edit below; `every_command_line_in_the_pages_parses` still counts the page's lines and passes; `heavy cargo nextest run -p temper-docs` and `heavy cargo nextest run -p temper --lib -E 'test(~help)'` pass.

## Inputs

- `temper/docs/design/protocol/setup.md`, 5.5, 6.6 and 7.4; `temper/docs/design/protocol/sign-in.md`, 9.7; `temper/docs/design/domain/worker.md`, 4.3.
- `temper/docs/help/forgejo.md`, step 2: the connect's output, which this page's words follow.
- `temper/docs/plan/help-server/inputs/request.md`, findings 3 and 4.
- `temper/docs/plan/help-server/notes/common.md`.

## Owns and may change

Owns: `temper/docs/help/server.md`. No other file. If a test reads a sentence removed here, change that test's expected words and say so in the report.

## Steps

1. **Step 1,** after `Every owner of provider 3 is a member. Adding a member is a change of this file and a restart.`, a new paragraph and block:

   ```text
   To sign your people in to Forgejo through temper too, add
   `engine.clients.provider`: `{"key": "provider.key", "clients": [{"id":
   "forgejo", "secret": "forgejo-client.secret", "redirect_uri":
   "https://git.example.com/user/oauth2/temper/callback"}], "addresses":
   [3]}`, and in `application.sign_in`, `"forges": [{"forge": 1, "client":
   "forgejo"}]`. Before step 3, this makes the key and the client's secret
   and prints the Forgejo command to run once, after step 4:
   ```

   then, fenced, `temper forge sign-in --config CONFIG --data DATA --cache CACHE 1`.

2. **Step 3's units:** `each with `UMask=0077`, `ProtectSystem=strict`, `PrivateTmp=yes`, `ReadWritePaths=/var/lib/temper`, `NoNewPrivileges=yes``, and after them: `strict` leaves `/tmp` read-only without `PrivateTmp`.

3. **Step 3's engine:** `since it takes Forgejo's handover at its start` becomes `since it connects a forge it keeps at its start`.

4. **Step 3's Expect:** `fetch` and `prepare` printing nothing is not what fetch does (setup.md, 6.6). It becomes: `fetch` says how far each binary has gone, `fetched BINARY DONE of TOTAL bytes`, and nothing for one already there; `prepare` prints nothing; each exits 0. Keep `run` and `serve`'s words. Add: with `sign_in.forges`, `serve`'s stderr says `temper serve: forge 1: no sign-in through temper: no such forge is kept` until step 6 connects it.

5. **Step 3's If not:** remove `A forge not prepared: ... then start the engine again.` No owned forge's record is made on this walk (setup.md, 7.4), so it cannot be met.

6. **Step 5's `The worker alone`** bullet ends: its unit as step 3's, `ReadWritePaths` its own home: without `PrivateTmp=yes` a Claude Code run fails `EROFS: read-only file system, mkdir '/tmp/claude-UID'`.

7. **Step 5's If not** gains: a task held `out of tries: run failures`: its runs failed, each for the reason the engine's journal names, `temper serve: task TASK run RUN attempt ATTEMPT failed CLASS: DETAIL`; then, fenced, `journalctl -u temper -o cat | grep 'task TASK run'`.

8. **Step 6:** its heading becomes `## 6. Sign in, through a tunnel, and connect the forge`. After the browser's sentence:

   ```text
   Then connect the Forgejo of step 3. Without a box temper keeps no record
   of it until this, so `forge list` is empty and `forge setup` is refused;
   on a new store it is forge 1:
   ```

   then, fenced:

   ```text
   temper ctl DIR forge connect forgejo-admin DATA/forge-1/admin_token --api 127.0.0.1:3000 --git http://127.0.0.1:3000 --web https://git.example.com --key KEY
   temper ctl DIR forge list --until connected:1
   ```

   then: Keep `DATA/forge-1/admin_token`: the next redeploy's connect reads it.

   **Expect** gains: `connecting`; then the wait prints the forges, forge `1` `managed`, then `met`. **If not** gains: `connected already` (exit 2): a kept store keeps its forge, and there is nothing to do. Other refusals: as temper help forgejo says in its step 2.

9. **Step 7:** `remove `DATA` but `DATA/secrets/` and `DATA/forge-1/`` gains: and, with Forgejo's sign-in through temper, the provider's key and the client's secret. `sign in again (step 6)` becomes `sign in and connect the forge again (step 6)`.

10. Run the acceptance's tests, push.

## Report

As the implementer's text says. Name any test whose expected words changed.
