# goal-call-faults: three faults a goal's coordinator met in its host calls

A goal on a real deployment at fabrig `b4555e6785`, cutting a ten-item request, met three faults in its host tools. None stopped it landing, but together they cost about an hour and a half of active time. Evidence is outside this repository, so the words here are what you have. **Run, unless marked read.**

## What's wanted, ranked. Drop from the bottom if it passes five tasks, and say what you dropped

1. **A batched `change` loses its options.** The goal's first `change` call held a `changes` array of four members, each with names and words, and `"landing": "goal"`, `"producer": 2`, `"questions": "foreman"` and `llm` given once at the call's outer level. The call was accepted, and the four changes it made targeted main, required CI and routed their questions to none. Later single-change calls with the same options inside the change targeted the goal's branch correctly. The batch's members should take the outer options as the single form does, or the call should refuse the placement before it makes any work, naming the option. The protocol says which (`temper/docs/design/protocol/forge-tools.md`, or wherever `change`'s batch is written). The failing row first, in `change`'s existing table.

2. **A coordinator can't subscribe while it runs.** `subscribe` with `kind: task` naming one of the goal's own live children (a change, or a person task it just made), and the goal's timer subscriptions, return exactly `Refused: task 4 busy`, task 4 being the goal itself. The goal's charter tells it to subscribe to a person task before waiting on it, and to the repairs it makes before releasing a held landing. A task subscribing to its own children from its own run is the designed use (jig's `domain/tasks.md`, wherever `subscribe` is written). Find what makes the asking task "busy" to itself, and fix it. The failing row first.

3. **A whole spec is read 128 bytes a page.** `read_task` with `field: result` on a delivered design report of about 30.5 KB advanced 128 bytes a call, so the goal took about 2 minutes 40 seconds of calls to read one spec it then handed on whole. Find the bound that sets the page (a reply budget left after framing, a default page, a per-field cap) and make a page as large as the reply's room allows, by the existing figures. Say what the page was and why. The failing row first.

4. **The design says so,** wherever each is written.

## Not in this plan

A charter's words, a model or effort, and any figure.

**Tests stay cheap:** a new case of one behaviour is a row of an existing table test; no test reaches a real host or a real LLM. Failing tests first. **A plan runs the focused and fuzzy suites alone and is complete when it lands (`AGENTS.md`, 5).** A stored format or protocol change is a question first. jig never names temper. `walk = false`.
