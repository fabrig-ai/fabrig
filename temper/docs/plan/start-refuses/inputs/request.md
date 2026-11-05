# start-refuses: a release at a start withdraws its stale proposals without a panic

From the deployment's redeploy on 2026-11-05. Restarting a build at `f9955bf` on a store written by a build at `3fe035b` panicked, in a crash loop:

```text
thread 'main' panicked at jig/crates/jig-domain-people/src/domain/start.rs:17:9: people projections require start_project before open
```

**It's not an older store being read wrongly: the store opened.** The backtrace, from a copy of that store:

`temper_engine_domain::restart::load_outputs` → `startup_page` → `route_into` → `jig_domain_core::start::script::start_done` → `write_restored_settings` → `routing::settings::dispatch::withdraw_stale` → `routing::children::append_tasks` → `task::tag_task_outputs` → `task::save::save` → `task_saved` → `routing::children::append_people` → `party::people::route_people` → `step_people` → `jig_domain_people::domain::step` → `assert_projection_event`.

## What that says, from reading (to confirm)

- `write_restored_settings` (`jig/crates/jig-domain-core/src/settings/restart.rs`, 410): the new binary's application defaults differ (`defaults_changed`), so the start makes a release, a new settings revision.
- A release withdraws every pending settings proposal (`withdraw_stale`, `routing/settings/dispatch.rs`, 441; "revision changes and release startup share the exact durable withdrawal path").
- The withdrawal saves the proposal's task, and `task_saved` routes to the people domain. But `start_settings_commit` runs before `self.people.start_sweep()` (`jig/crates/jig-domain-core/src/start/script.rs`, about 73 to 76), so people isn't ready, and `assert_projection_event` fires.

So **any restart with a pending settings proposal and changed defaults crash-loops,** on any build that changes the stock: the store is good, and the deployment can't start. A proposal comes from a configuring chat, so this is on the path of a person using temper. Nothing here refuses, or should.

## The task

1. **A failing test first,** in `jig/tests/core`: a store with a pending settings proposal, a start with changed application defaults, and the start finishes: the proposal is withdrawn as `engine.md`, 17 says, its task told, no panic. Prove it fails on the old code. Also: the same with a held revision, and a restart again after it, once a story of the settings restart tier exists for those.
2. **Fix it in the core,** the lowest tier. Settle by the design which is right:
   - the withdrawal's people work waits until people's start has ended (the sweep), or
   - the release's withdrawal runs after that;
   - or what else `jig/docs/design/domain/engine.md`, 6 and 17 say. Durability first: the withdrawal must survive a crash between the commit and what follows, so a second start gets the same result (a story for it).
3. **Sweep the same class:** any other event the start's commit phases send to a domain before its start has ended, such as `withdraw_project`, or the application release's other items. Name each in the report, and fix those on the same path.

Temper's start reports what it refuses through `temper-service/src/engine.rs`, `refused_start`; a panic there is not the promise `temper/docs/help/server.md` and `deployment.md` make. Formats change in place before v0.1.0, and a store of truly another format stays refused, as it does today (`jig/crates/jig-protocol-store/src/open.rs`). Don't change that.

jig is shared with hearth, whose manifests name no jig crate: name the change in `plan.md`.

## Two small additions to `temper/docs/help/server.md`

Each with its help test (`temper/docs/help` is read by tests):
- the agent's `environment` takes `[{"name": ..., "value": ...}]` entries, not a map (section 5 doesn't say; `box.md` shows it);
- with `application.sign_in.forges` set and a new store, the first start says `forge 1: no sign-in through temper: no such forge is kept`, and the page's meta appears only after `forge connect` and a restart. Say so in the walk.

## Size

One or two tasks. A stored format, a protocol or a public interface change is a `Question:`. Budgets and timeouts are the user's (skein's `docs/foundation/testing-strategy.md`, 8.1). fabrig/ui has priority: leave its files alone. ops keeps the store, and re-runs the fixed build on a copy of it as the real proof.
