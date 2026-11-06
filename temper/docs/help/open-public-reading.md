# Open public reading

Kind: walk

Enable anonymous reading at the deployment's entrance, then make a project public and check its public pages without exposing its private words.

## Before you start

- **A running deployment** with its configuration at `CONFIG/temper.json`,
  and a signed-in project owner using `DIR` (temper help box).
- **Control of its configuration and restart.** Visibility is a live owner
  operation; admission limits are startup configuration. Opening visibility
  alone does not enable reading while admission limits remain zero.
- `PROJECT`: the project's number, printed by `temper ctl DIR me`.
- `PUBLIC`: the public identifier in the page path printed after opening.
- `TASK`: a task number in that project's public board.
- **An example capacity,** below: four concurrent reads, two reads per
  second, a four-read burst, and answers of at most one MiB. Choose figures
  for your deployment; temper help limits-engine describes their bounds.

## 1. Set admission and index limits

In `CONFIG/temper.json`, merge these members into `engine.limits.overrides`,
keeping the other overrides. These are engine limits, not
`application.limits.overrides` or `workers[].limits.overrides`.

```json
{
  "clients.public.reads": 4,
  "clients.public.rate": 2,
  "clients.public.rate_burst": 4,
  "clients.public.bytes": 2097152,
  "clients.public.bytes_burst": 4194304,
  "clients.public.answer_bytes": 1048576,
  "clients.public.max_age": 30,
  "clients_http.public_calls": 4,
  "core.public.goals": 16,
  "core.public.ended": 16,
  "core.public.lines": 32,
  "core.public.depth": 8
}
```

The clients' figures bound concurrent reads, reads per second and burst,
answer bytes per second and burst, one answer's bytes, and freshness in
seconds. The HTTP figure bounds concurrent anonymous HTTP calls. The core
figures bound live goals per board, ended goals kept per project, lines per
goal and delegates per card, and a card's lineage.

Stop and restart the deployment using its existing start command (temper
help box or temper help server). All configuration changes need a restart.

Expect: the deployment starts with the edited configuration.

If not: read the startup error's path. A burst must cover its rate;
`clients.public.answer_bytes` must fit the byte burst and every declared
public answer kind must fit that answer bound. HTTP calls must fit public
reads; freshness is at most 60 seconds. A process fit refusal needs smaller
index bounds or fewer projects. See temper help configuration-engine.

## 2. Open the project as its owner

```
temper ctl DIR me
temper ctl DIR project amend PROJECT --visibility public --key KEY
temper ctl DIR project show PROJECT
```

Expect: `project PROJECT: public` and `page: /public/projects/PUBLIC`.
Keep the identifier from that path; the private project's number is not
its public identifier. Only an owner changes visibility.

If not: sign in as a project owner. If the page is busy, check every admission
figure from step 1 and that the deployment restarted with the edited file.

## 3. Read without signing in

Open `/public/projects/PUBLIC` under the deployment's browser address in a
browser without a sign-in. Follow a task to
`/public/projects/PUBLIC/tasks/TASK`; landing pages are
`/public/projects/PUBLIC/landings` and
`/public/projects/PUBLIC/landings/TASK`.

Expect: the project's name, task structure and titles, and public landing
summaries. Specifications beyond titles, instructions, results,
conversations and people's names stay private.

If not: check the path from `project show`, the listener and browser origin
in `application.clients` (temper help configuration), then step 1's limits.
A busy response can mean the read or byte allowance is spent; wait for its
rate to refill before retrying.

## Clean up

To close the project again, use a fresh key:

```
temper ctl DIR project amend PROJECT --visibility private --key KEY
```

Expect: `project PROJECT: private`. To close anonymous admission for the
whole deployment, set `clients.public.rate` and `clients.public.reads` to
zero and restart it.

## Next

Read temper help project for visibility's contract and temper help
limits-engine for each public figure.
