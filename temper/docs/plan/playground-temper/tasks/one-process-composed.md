+++
id = "one-process-composed"
part = "composition"
kind = "migration"
needs = ["playground-processes", "one-process-documents"]
files = [
  "temper/tests/one-process/**",
  "temper/docs/design/domain/testing.status.md",
  "temper/docs/design/playground.status.md",
  "Cargo.lock",
]
effort = "high"
minutes = 150
+++

# The one-process world composes `temper-playground` with its harness

## Goal

"The one-process world composes it with its harness, so the gate runs the world the browser runs"
(`temper/docs/design/playground.md`, 1 and 3). The one-process world's composition goes: its world
is `temper_playground`'s processes and documents, and what stays is its harness, "its referee,
its fake person in front of the terminal client, its captures, its faults, its kills and power
losses, its fixtures' scenarios, all over the composition" (3, "What stays in
`tests/one-process`"), through C2's seam. No story changes meaning.

Acceptance:

- The one-process world's process type is `temper_playground::Process<OneProcess>` (a type alias
  `Proc` may stay), where `OneProcess: Harness` holds its own processes (the hosted ctl, its
  callers, the person in front of the terminal client) as `H::Process`, and its service-side
  captures (capture, store, commits, wire, cut, reserve, codec, families, hooks) as
  `H::Service: Observe`.
- Its worlds register `temper_playground::programs::<OneProcess>()` and add `/world/temper-ctl`
  themselves, at each place a world is built today (`world.rs` ~1082, `oracle.rs` ~145 and
  `spawn` ~167, `lower.rs` ~259, `lower/github.rs` ~446, `m15.rs` ~151, `m15/run.rs` ~294).
- No copy of a composition item is left in `temper/tests/one-process/src/` (`machine.rs`'s
  `SignalHost`, `Rooted`, `Machine`, `fresh`; `stories.rs`'s `Shell`; `world.rs`'s factories,
  `share`/`transport`, `peer`, `host_programs`, the handover drain): each is the playground's.
- `heavy cargo nextest run -p temper-one-process-world` passes with the trunk's count, and its
  fuzzy suite (`--profile fuzzy`) passes as on the trunk.
- The status entries below are edited.

## Inputs

- `temper/docs/plan/playground-temper/notes/common.md` and `notes/contracts.md`, C2 and C3.
- `temper/docs/design/playground.md`, 3; `temper/docs/design/domain/testing.md`, 2.5.
- The reports of `playground-processes` (its hooks, each with the call site it serves) and
  `one-process-documents`.

## Owns and may change

- All of `temper/tests/one-process/` (sources, tests, `Cargo.toml`). The `Proc::...` arms the
  harness matches on (about 80 sites: referee, oracle, m15, lower, ctl) become
  `Process::...` or `Process::Other(...)`, edited only for that.
- `temper_playground` is not yours: if a hook it lacks is needed, or one it has cannot serve its
  call site, stop and ask, naming the call site.
- `.config/fixture-readers.toml` names `temper/tests/one-process/src/fixtures/*.rs`: keep
  `src/fixtures/durable.rs` where it is.
- Status, as `notes/common.md` says:
  - `temper/docs/design/domain/testing.status.md`: in its first **Not yet** entry, remove
    `2.4, 2.5, 7: the twins' documents and the one-process world's composition in
    \`temper-playground\`, which does not exist; \`tests/playground\`.` and put in its place
    `2.4, 7: the twins' documents in \`temper-playground\`; \`tests/playground\`.`; nothing else.
  - `temper/docs/design/playground.status.md`: in **Not yet**, `3: \`temper-playground\`,
    \`temper-playground-world\`` becomes `3: \`temper-playground-world\`` (and the rest of that
    item word for word).

## Steps

1. `OneProcess` and its `Observe`; `world.rs`'s `ServiceHost` and `Proc` replaced; `heavy cargo
   check -p temper-one-process-world --tests` while errors last.
2. The world builders and the harness's match arms.
3. Delete the copies; `heavy cargo nextest run -p temper-one-process-world`, count against the
   trunk's; then `heavy cargo nextest run -p temper-one-process-world --profile fuzzy`. A story
   that now fails or runs past its budget is read from its log first; a change of meaning is a
   question.
4. The status edits; the citations check.
5. `cargo fmt`; old forms for temper (`--write` for what the deletions lower); `heavy cargo clippy
   -p temper-one-process-world --all-targets -- -D warnings`. Commit the rewire, then the status;
   push.

## Report

As the implementer's text says, and: `OneProcess`'s shape; each world builder; the stories'
counts and times before and after.

## Answers

2026-11-03. **GitHub selection is runtime, not a constant: one `OneProcess`, a runtime hook.**
`Harness::GITHUB` is now `fn github() -> bool` (default `false`, the playground's `()` keeps it),
read by `programs::git`, `programs::agent_root` and `machine::agent_roots` at each spawn; this
answer made that change in `temper_playground`. `OneProcess` overrides it to return
`lower::github::active()`, so its GitHub worlds and the others share one `Process<OneProcess>`;
do not use distinct harness types and keep no local factories. The other hooks are the
`Harness` trait's in `temper/crates/temper-playground/src/programs.rs` (`configure`, `spawned`,
`complete`, `git_limits`, `seeds`, `agent_seed`, `files`, `given`): read them there in place of
`playground-processes`' report. No root exports are added: the acceptance's paths mean
`temper_playground::programs::{Process, Harness}` and `temper_playground::programs::programs::<OneProcess>()`.
If another hook proves missing, stop and ask again, naming its call site.

2026-11-03. **The restart factories at `world.rs` ~1097–1098 are the playground's.** This answer
added to `temper_playground` `programs::llm::<H>` and `programs::forge::<H>` (both
`fn(&Inherited) -> Process<H>`, for `spawn_restartable`), two hooks `Harness::provider(one) ->
llm::Peer` and `Harness::forge_seed(one) -> u64`, and `fakes::forge_peer(&Typed, seed)`, the peer
`Fakes::new` builds. `programs::forge` reads the forge from the scoped `Parameters` (whose `forge`
the harness sets to its run's transport), so a restarted peer shares it with git. `OneProcess`
overrides `provider` to return `fixtures::provider()` and `forge_seed` to return `fixtures::SEED`,
and the `Phase::Fault` arm spawns `programs::llm::<OneProcess>` and `programs::forge::<OneProcess>`.
The non-restartable spawns (`Phase::Restart`, `Story`, ...) build `Process::Peer` and
`Process::Forge` in their closures from `stories::provider(scenario)` and
`fakes::forge_peer(&transport, fixtures::SEED)`, with the transport they already hold. So
`share`/`transport`/`peer` and the local `llm_peer`/`fake_forge` go, as the acceptance says; no
`parameters()` accessor is exported. The answerer could not build this change (heavy had no slots
directory): build `temper-playground` first and, if it fails, fix it there; this task may edit
those three items in `temper/crates/temper-playground/src/{programs,fakes,story}.rs` for that alone.
