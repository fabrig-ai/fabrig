# Spot review of debt-13 (2026-11-05, merge fcb84eaa3e)

Read-only reviewer, using the user's quality criteria. Nothing was run. Line numbers are at the merge.

skein: D013 paid; one entry per forge, `get(at)` 155 to 0, `unreachable!` 54 to 18, none naming a dialect. Production
+95, mostly earned given step code bans generics. About 55 production and 60 test lines could still go.
jig: partly earned. `ops` 3,835 to 3,627, mostly clearer; non-test code -97 at the cost of 499 new test lines.
No correctness bug and no credential exposure.

2. Risk: the dialect is still asked at 23 sites (`skein-forge-connection/src/component/github_calls.rs:180`, 5 in `calls.rs`,
   1 in `github_logs.rs`): `let Some(github) = waiting.github() else { return };`. A panic became a silent stall to the
   deadline, against forge-connection.md 7.1. Take `Side` out of `Waiting` in `drive_calls` and pass `&mut GitHubCall` or
   `&mut ForgejoCall`, or one accessor with one `expect`. About -35. `a_call_of_the_other_dialect_is_unavailable` goes.
3. `hub_link::now` costs more than it saves (`jig-domain-hub-link/src/down.rs:86`): +47 production, +75 tests to save
   23 root lines, and `ops` reads worse. `core_now_returns_a_roots_output_without_consuming_its_bytes` can't fail on its
   pointer assertion. Nest the six in `Now::Link(..)`, touching 13 core construction sites.
4. Risk: an own `Exchange` outlives its connection (`skein-forge-connection/src/component.rs:346`, `:409`, `:551`):
   `done`, `heard` or the decoder can hold a `Minted` token after `state.forget()`. Memory lifetime only; `Secret` redacts.
   Reset the exchange wherever `own` clears, and remove the `h.pump()` added at `connection/tests.rs:96`.
5. `Env::wall_due` is a roundabout comparison, and `tests/env.rs:24` pins a wrong answer: use `expiry <= env.wall` at call
   sites, delete the method and its test. About -19.
6. Not earned: `settled_answer` (return `Option`), `no_workspace` and `complete_brief` (keep in the roots; `root.md`
   line 526 names `no_workspace`). `leaves.rs:48` has one caller. Temper uses none of this yet, so its copies remain.
7. `hosted` (`jig-protocol-charter/src/hosted.rs:23`): take the `Assignment` by value, drop `Given.answered` and `messages`,
   rename `Given`; the charter crate's new dependency on `jig-domain-hub-link` for one struct should go.
8. `skein-forge-connection/src/tests/fixtures.rs:528`, `:554`, `:848` rebuild the dialect mismatch in tests; `github/tests.rs:11,18`
   keep ignored `_at`s; three `#[path]` tricks. Add `start_forgejo` and `start_github`, plain layout. About -50.
9. `connection/forgejo.rs:157-203` matches `connection/github.rs:145-191`: `Payload::closes` and `Exchange::release`, about -12.
10. Small: `Core::cold_header` and `Core::commit_header` have no outside production caller (`routing/timers.rs:75,84`);
    `jig-domain-tasks/src/boundary.rs:319` has a test module mid-file; `component/runtime.rs:30` `remove` uses a closure under an
    `expect`, while `find` says closures are excluded.
