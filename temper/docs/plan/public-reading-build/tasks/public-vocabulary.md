+++
id = "public-vocabulary"
part = "jig-public"
kind = "interface"
needs = []
files = ["jig/crates/jig-domain-core/src/public.rs", "jig/crates/jig-domain-core/src/lib.rs", "jig/crates/jig-domain-core/src/vocabulary.rs", "jig/crates/jig-domain-core/src/tests/public.rs"]
effort = "high"
minutes = 100
+++

# The core's public vocabulary: the read, the connector's ask and answer, the held answer

## Goal

jig's core gains the messages public reading crosses its boundaries with, and the domain's
public documents, so the core's, the component's and each application's tasks build their
halves at once (`notes/contracts.md`, section 4; `jig/docs/design/protocol/client.md`,
section 19; `jig/docs/design/domain/engine.md`, section 11). Nothing reads publicly yet: the
core answers every public read unknown, every connector answers the public ask with nothing,
and every root hands a public read back busy.

Acceptance:

- `jig_domain_core::public` (`src/public.rs`), re-exported at the crate root as the crate's
  other vocabulary is: `PublicRead`, `PublicAnswer`, `PublicDocument`, `PublicProjects`,
  `PublicBoard`, `PublicGoal`, `PublicCard`, `PublicLine`, `PublicCounts`,
  `PublicApplication`, `PublicExecutor`, `PublicPhase`, `PublicHold`, exactly as the
  contracts' section 4 gives them (`PublicPhase` one unit variant per
  `jig_domain_tasks::Phase` variant as the client contract's `Phase` names them;
  `PublicHold` as `public-wire`'s, one unit variant per kind of hold a held task's line names,
  or the one variant `Held` if it names none), each deriving what the vocabulary's neighbours
  derive; the module's doc cites `jig/docs/design/domain/people.md`, section 13.
- `vocabulary.rs` gains, appended last in each enum, each with a doc line citing its section:
  `Event::PublicRead { reply: ReplyTo, read: PublicRead }`,
  `Event::PublicAnswered { reply: ReplyTo, answer: Option<PublicApplication> }`,
  `Ask::Public { reply: ReplyTo, kind: u16, version: u16, project: u32,
  task: Option<u64>, public: Box<[u64]> }` and
  `Held::PublicRead { reply: ReplyTo, answer: PublicAnswer }`.
- The core answers `Event::PublicRead` at once with `Held::PublicRead { answer:
  PublicAnswer::Unknown }`, committing and marking nothing, and ignores an
  `Event::PublicAnswered` it holds no read for. Every other match on these enums in the core
  gains its arm (classifications such as the `Self::Read { .. }` list near `vocabulary.rs`'s
  end place `PublicRead` beside `Read`).
- **Every root and connector** gains its forced arms, edited only for them:
  - a connector answers `Ask::Public` with `Event::PublicAnswered { reply, answer: None }`
    through its root, as it answers a read it does not serve: `jig/testing/jig-test-domain/src/routes.rs`,
    `jig/examples/ops/domain/src/route.rs`, `temper/crates/temper-engine-domain/src/route/asks.rs`
    and `route/forge_route.rs`, `jig/tests/store/src/world.rs`,
    `jig/testing/jig-conformance/src/core.rs`, `temper/tests/one-process/src/machine.rs`;
  - a root hands `Held::PublicRead` to nothing yet (no exchange sends a public read until
    `public-component`), dropping it; and where a root receives an `Event::PublicRead` from
    outside, it answers it busy: `jig/testing/jig-test-domain/src/client_documents.rs`,
    `jig/testing/jig-test-service/src/route.rs`, `clients/translation.rs`,
    `jig/crates/jig-domain-hub-link/src/down.rs`, temper's root, and any other the compiler
    names.
- Step tests in `jig-domain-core`'s `src/tests/public.rs`:
  - `a_public_read_is_answered_unknown_and_commits_nothing`: for each of jig's kinds and an
    application's, the answer is `Unknown`, no write and no store load leave the core, and
    no party's state changes;
  - `an_unasked_public_answer_changes_nothing`.

## Inputs

- `temper/docs/plan/public-reading-build/notes/common.md`; `notes/contracts.md`, section 4.
- `jig/docs/design/domain/people.md`, 13.3, 13.7 and 13.10; `jig/docs/design/protocol/client.md`,
  section 19; `jig/docs/design/domain/engine.md`, section 11 (public reads).
- As they stand: `jig-domain-core/src/vocabulary.rs` (`Event`, `Ask`, `Held`), `reads.rs`
  (how `Ask::Read` is asked and answered), `lib.rs`'s re-exports; `jig-domain-tasks`'
  `Phase`; `jig-client-domain/src/contract/common.rs`'s `Phase`.

## Owns and may change

- Owns the files in the header. `vocabulary.rs` is shared, by hunk, with `public-limits`,
  which edits the core's `Limits` struct at the same time: edit only the `Event`, `Ask` and
  `Held` enums and their classifying functions there. `lib.rs`: the module's declaration and
  re-export alone.
- **Forced arms** in the files named above and any other in jig or temper the compiler names,
  each edited only for the arm; temper's are a required arm of jig's enum, edited only for
  it. `jig/testing/jig-test-domain/src/lib.rs` is not this task's (`public-limits` edits it);
  if an arm must go there, put the arm's body in `routes.rs` or `client_documents.rs` and
  call it.
- Pre-approved: if a root routes connector answers through an enum of its own (a
  `ReadAnswered`-like message) rather than `Event`, add the public answer's variant there,
  appended last, carrying `Event::PublicAnswered`'s fields, and say which. If a `Held`
  variant must name its destination component, `Held::PublicRead` goes where `Held::Read`
  goes.

## Steps

1. `public.rs` and the enums' variants; the core's arms;
   `heavy cargo check -p jig-domain-core --tests`.
2. The roots' and connectors' arms until
   `heavy cargo check -p jig-test-service -p jig-ops-domain -p temper-engine-domain -p temper-service --tests`
   passes (package names as their `Cargo.toml`s give them).
3. The step tests; run them alone
   (`heavy cargo nextest run -p jig-domain-core --lib -E 'test(/public/)'`).
4. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py` and
   `python3 -I -B temper/scripts/old_forms.py`; clippy on every package you changed; the
   focused tests of `jig-domain-core`, `jig-test-domain`, `jig-test-service`,
   `temper-engine-domain` and the other packages you touched; the citations check.
5. Commit (why: the public read's messages across the core's boundaries, which the core's,
   the component's and the forge's tasks build on at once); push your branch.

## Report

As the implementer's text says, and also: `PublicPhase`'s and `PublicHold`'s variants, and
every forced arm by file with what it does.
