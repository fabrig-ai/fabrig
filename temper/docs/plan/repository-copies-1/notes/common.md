# repository-copies-1: standing rules

Every task of this plan names this note.

- Read `AGENTS.md`, then the `AGENTS.md` of the tree you work in. A task stays in its tree. The one
  exception is a required field or variant: the producer's branch carries the other tree's lines that
  name it, each edited only for that, as its spec lists them.
- **skein stays generic.** skein names no temper: no project, no `Making`, no adoption. Its plan copies
  a repository for an owner, and that is all it knows (`AGENTS.md`, 1).
- **The design is written.** Each task's `## Inputs` names the sections it builds; read those, never a
  whole design. The one `docs/design/` edit a task makes is the removal of its own **Not yet** text
  from the status file its spec names (a stated departure from the implementer's text, whose "Not
  yours" otherwise holds). Where the code teaches that the design is wrong, say so in the report;
  never change the design to fit.
- **The shapes that cross tasks** are `temper/docs/plan/repository-copies-1/notes/contracts.md`,
  binding. A shape a task needs that it does not give is the spec's, or a question.
- **Changed in place** (`temper/docs/design/draft/repository-copies.md`, 1): no supported deployment
  exists before v0.1.0, so the store's families, the FORGE records and `CLIENT_VERSION` change in
  place, with no migration and no version kept for an older side.
- **No test reaches a real host.** Sources are the fake's catalogue, under `.invalid` hosts, or a
  second throwaway Forgejo on loopback. No `github.com` in a test's request.
- **Build no welcome choice, no Ready's copies, no "Add a repository" sheet** (the draft's sections 7
  and 9). No page, view, word, gallery sample or CSS changes for this plan.
- **Test worlds:** a crate's `src/tests/` and its tree's `tests/...`. Fakes, never real services. Each
  test is one thread and one loop, and a seed replays (`AGENTS.md`, 2).
- **Seeding a source (`fake-box`):** if a second branch cannot be seeded without a new helper in skein's
  domain, seed the default branch alone, and say so in the report.
- **A budget or timeout of a test is the user's**
  (`skein/docs/foundation/testing-strategy.md`, 8.1): no task raises one.
- **Beside this plan:** `macos-cheap` moved the Forgejo pin and holds
  `skein/tests/forge/conformance/{facts.txt,transcripts/**,deliveries/**}`; `to-begin-store` takes
  store byte 34 and is on `temper-engine-domain-forge` and `temper-protocol-store`; `look-round-3` is
  on `temper-client-domain`'s and `temper-client-view`'s `pages/`; `debt-11` and `debt-10` sweep
  `temper/crates/temper-service/src/**` and `temper/crates/temper-engine-domain/src/**`;
  `kv-chunk-target` is on `skein-kv`. Keep each edit in a file they touch to the lines the task needs:
  no reformatting, no moved code. Expect catch-ups; never rewrite their work.
