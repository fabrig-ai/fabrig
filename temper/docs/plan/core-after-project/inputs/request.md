# core-after-project: a deployment's core keeps answering after a project is made

A real deployment at fabrig `7638040201` stopped answering every call that needs its core, right after `project make`, and stayed so across a restart on the same store. Its evidence is outside this repository, and its store holds a login, so nobody copies it: the words here are what you have, and the cause must be reproduced from the acts. **Run, unless marked read.**

**What happened, in order, on one store:**
1. A goal ran whole on it earlier: a project (2), a repository adopted, changes, a landing into main and a close-out, then the goal ended.
2. A program account was forgotten ("Signed out on every machine").
3. Settings revision 4 was applied against 3, setting all eleven items again, two of them changed.
4. `account declare` for that forgotten account's name was refused, "This act is not offered" (another plan repairs that).
5. `account list` answered, at 23:50:20.
6. `project make proof-5-clearer-words` returned project 3, at 23:50:25.892.
7. From the next call, 0.1 s later, every call needing the core failed: `project show 3`, `repository adopt`, `me`, `account list`, `settings show`, and a fresh client's `sign-in` and `me`. The client's link "did not come up within 30 s", or the engine "did not answer within 30 s".

**What was seen of the engine:** `temper serve` logged nothing after it. Its main thread slept in `io_cqring_wait`, not spinning, and an io worker was idle. The HTTP router answered at once: unauthenticated `me` 404, `stream` 401, the private public read 503, and a spent sign-in link refused at once. The store took writes: a new log file began during a sign-in. A restart on the same binary, config and store came up, the worker relinked, and the same calls hung again.

## What's wanted, ranked. Drop from the bottom if it passes five tasks, and say what you dropped

1. **Reproduce it, the failing story first, in the lowest world that holds a store, a restart and the root's routes:** the acts above, in order, then a core call that must answer within the world's bound, before and after a restart on the same store. Find the smallest sequence that still hangs. Step 6 is the likeliest trigger; steps 1 to 3 its likeliest preconditions.
2. **Find what the core waits on:** a load that never completes, a startup continuation, a held lane, an admission that never frees, or a record the core can't decode and parks on. Say it in the design's terms, with the code.
3. **Fix the cause,** so the core answers, and a store already in that state opens and answers too, as formats change in place before v0.1.0.
4. **A core that can't answer says why:** whatever the core waits on is named in the log, and a client's 30 s wait says the core is busy with what, not only that the link didn't come up.
5. **Read, not run:** whether proof-4's earlier goal in the same store, or the forget, is needed. Say which preconditions mattered.

## Not in this plan

Re-declaring a forgotten account, which another plan repairs.

**Tests stay cheap:** a new case of one behaviour is a row of an existing table test, and a new story goes in a world below the real loop. Failing tests first; no test reaches a real host. **A plan runs the focused and fuzzy suites alone and is complete when it lands (`AGENTS.md`, 5).** A stored format or protocol change is a question first; both main areas write `temper-engine-domain` and `forge_route.rs`, and skein's interface changes get a heads-up. Formats change in place before v0.1.0. `walk = false`.
