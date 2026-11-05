# start-refuses: an older store is refused, not a panic

From the deployment's redeploy on 2026-11-05. Restarting a build at `f9955bf` on a store written by a build at `3fe035b` panicked, in a crash loop, instead of refusing:

```text
thread 'main' panicked at jig/crates/jig-domain-people/src/domain/start.rs:17:9: people projections require start_project before open
```

`temper help server` and `temper/docs/help/deployment.md` promise a refused start that names the reason, and the store's checks at open (`jig/crates/jig-protocol-store/src/open.rs`: levels, family versions, layout fingerprints) exist for it. The earlier binary still opened that store. Formats change in place before v0.1.0, so the store need not open: it must be refused.

## The task

1. **Find how the start got past the checks and reached the panic.** Candidates, from reading only:
   - `assert_projection_event` asserts `self.ready()`. So an event reached the people domain while its start had failed or was still loading: perhaps a `restore_failure`, whose fit failure the core doesn't stop on before delivering the next event, with a limit or value the newer build bounds more tightly;
   - or the families' layouts matched, while what the rows mean moved: a value the newer build reads differently.
   
   Between the two builds, `temper/crates/temper-protocol-store/` and jig's store schemas both changed; see `git diff 3fe035b f9955bf`. The store itself isn't available to a task. Build an older store in a test where you can, or write the state directly at the lowest tier.
2. **A failing test first,** in the lowest tier that shows it: `jig/tests/core`, if it's jig's.
3. **Make it a refusal** that names the reason, through the existing refusal path (`temper-service/src/engine.rs`, `refused_start`), exit non-zero, not a panic. Any other way an older store's content can reach an assert during a start: name it in the report, and fix it if it's the same path.

jig is shared with hearth, whose manifests name no jig crate: name the change in `plan.md`.

## Two small additions to `temper/docs/help/server.md`

Each with its help test (`temper/docs/help` is read by tests):
- the agent's `environment` takes `[{"name": ..., "value": ...}]` entries, not a map (section 5 doesn't say; `box.md` shows it);
- with `application.sign_in.forges` set and a new store, the first start says `forge 1: no sign-in through temper: no such forge is kept`, and the page's meta appears only after `forge connect` and a restart. Say so in the walk.

## Size

One or two tasks. A stored format, a protocol or a public interface change is a `Question:`. Budgets and timeouts are the user's (skein's `docs/foundation/testing-strategy.md`, 8.1). fabrig/ui has priority: leave its files alone.
