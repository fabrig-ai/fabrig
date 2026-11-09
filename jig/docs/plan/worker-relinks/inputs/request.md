# worker-relinks: a worker's link that drops every two minutes, and an engine that stops taking it back

A real deployment at fabrig `2c8b9a3f7d` (one engine, one remote worker on another host over a forwarded loopback port, about eight goals' tasks at once) showed three faults on 2026-11-08. Evidence is outside this repository, so the words here are what you have. **Run, unless marked read.**

- **The worker relinks about every 2 to 2.5 minutes,** from at least 17:55Z, with nothing in either log between links: no error, no reason. The pattern came back right after a restart. The network path stayed up: pings fine, the forwarded port accepting on both hosts.
- **At 18:19Z the engine stopped taking the worker back** and logged nothing more for 3.5 hours. It still answered a client's `me`, but no worker connection was established at either end. Every run then waited for a host. Its last lines, 18:14 to 18:18Z, were a burst of forge reads refused as "read admission busy" and briefs failing `counts=capacity reason=claim: refused`. Restarting the worker didn't help; restarting the engine did.
- **The engine didn't stop on SIGTERM:** systemd killed it after 90 seconds. Right after the restart, briefs failed for about a minute, about 23 times, with `transcript: its window could not be loaded`, which held five tasks until released.

## What's wanted, ranked. Drop from the bottom if it passes four tasks, and say what you dropped

1. **Why a link drops every two minutes,** with the failing test first, at the lowest tier holding the engine's and the worker's link layers (jig's `protocol/channel.md` section 9: pings, the silence deadline and the stall deadline). Candidates to rule in or out by test, not by assumption:
   - a ping interval or silence deadline that the engine's own loop misses under load, such as a long step on a burst of forge reads or briefs;
   - output queued past the stall deadline because the engine stops draining it;
   - a deadline that fires on a busy but healthy link.

   Fix the cause. A link that drops should say why, in a log line on each side.
2. **Why the engine stops taking the worker back after many relinks,** with the failing test first: a worker that links, loses its link and links again many times, say 50, is taken back every time. Look first for state a lost link leaves behind: a worker row, a claim, a slot, a watch or a listener's accept budget not freed, or a fleet bound (`fleet.workers`) counted per link rather than per worker. Fix it, and have an engine that refuses a link log why.
3. **A clean stop:** the engine stops on SIGTERM within its stop grace, with its store consistent, and a restarted engine loads every task's transcript window at once. The failing row first, in the existing stop or restart tests.
4. **The design says so:** channel.md section 9 and jig's `domain/hosts.md` wherever a link's loss and a worker's return are written.

## Not in this plan

Any figure: intervals and deadlines stay as configured, unless a test shows one is wrong, and then it's a question first.

**Tests stay cheap:** a new case of one behaviour is a row of an existing table test, and a story goes in a world below the real loop on a simulated clock. Failing tests first; no test reaches a real host. **A plan runs the focused and fuzzy suites alone and is complete when it lands (`AGENTS.md`, 5).** An interface change in skein's `io`, `sim` or `world` goes to hearth/core first. jig never names temper. `walk = false`.
