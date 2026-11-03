# playground-skein: skein's playground kit

At the user's request (11-03), carry out the playground in the browser, part one: skein's kit, as `skein/docs/design/playground.md` designs it, sections 2 to 8. The world runs in a Web Worker and carries the client's transport to it.

## The scope

From `skein/docs/design/playground.status.md`, all not built yet:
- `skein-playground`: the worker's loop (section 3, the slices), the visitor (4), the bridge (5), the session log (6), and the page, the worker and the bundle (7);
- `skein-ui-web`'s bridge messages, session log and worker platform (5, 6; `skein/docs/design/ui-web.md`, 18);
- `skein-ui-bundle`'s worker module (7).

Its tests are section 8's. The design exists; its designers write only where a cited section is incomplete (`DESIGN.md`).

## Context

The whole feature is temper's `temper/docs/design/playground.md` ("The playground in the browser": temper's engine, worker, jig's store, smith's agents with the fake LLM, forge and git, all on skein's simulator in a Web Worker, with temper's web client on the page). Its draft (`temper/docs/design/draft/browser-playground.md`, deleted on 11-02 because the design carries it whole) is readable in git at `13e7c2a649^`. `v0.1.0.md`, section 5 lists it among v0.1.0's parts. A second plan, `playground-temper`, builds temper's composition beside this one. skein names nothing of temper (`AGENTS.md`, 1).

## Bar

A desktop browser; phones are Later. skein's browser tier measures a slice's latency against the bar the application sets. Budgets and timeouts are the user's (skein's `docs/foundation/testing-strategy.md`, 8.1).

## Size

One cut is at most 90 minutes. Release early. If the whole cannot be designed in one cut, release what holds together and end with a `Question:` naming the rest.

## Walk

None of its own. It is walked with temper's page, in a later plan.
