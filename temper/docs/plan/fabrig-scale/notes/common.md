# fabrig-scale: standing rules

Every task of this plan names this note.

- Read `AGENTS.md`, then the `AGENTS.md` of each tree the task changes (`skein/AGENTS.md`,
  `smith/AGENTS.md`, `jig/AGENTS.md`, `temper/AGENTS.md`). Dependencies point one way: smith
  names neither jig nor temper, jig never names temper, and jig's core never links smith.
- **The design is written.** Each task's `## Inputs` names the design sections it builds, by
  path and section; read those, never a whole design. The one `docs/design/` edit a task makes
  is the removal of its own **Not yet** text from the status file its spec names (a stated
  departure from the implementer's text, whose "Not yours" otherwise holds); no other design,
  draft or `docs/help/` edit unless the spec names the file. Where the code teaches that the
  design is wrong, say so in the report; never change the design to fit.
- **The shapes and figures that cross tasks** are `temper/docs/plan/fabrig-scale/notes/contracts.md`.
  A figure there is binding; a figure a task needs that it does not give is the spec's, or a
  question.
- **Store formats change in place** (the user, 11-03, `inputs/request.md`): a schema's ceiling or
  shape changes with no new version and no migration of existing stores; a store whose layout
  fingerprint no longer matches is refused at open as jig's `protocol/store.md`, 5.3 says, and
  that is accepted. The generated code, the layout fingerprints and the goldens are regenerated
  by their writers, never by hand:
  - jig's families (store, client wire, worker wire) and the store's layout fingerprints:
    `SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p jig-store-codecs-world --test drift`
    (`jig/tests/codec-families.rs` rewrites every jig family; run it after your schema edit and
    commit only your own schemas' generated files and goldens plus
    `jig/crates/jig-protocol-store/src/generated/layouts.rs`; a diff in another family's
    generated file means your tree is stale: say so in the report);
  - temper's (store-v11, client wire, worker wire): `SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run
    -p temper-codecs-world --test suite -E 'test(drift::)'`;
  - smith's (transcript, channel, charter, local store): `SKEIN_CODEGEN_WRITE=1 heavy cargo
    nextest run -p smith-codecs-world -E 'test(/schema_matches_generated_code/)'`
    (`smith/tests/codecs/tests/suite/drift.rs`).
- **A budget or timeout of a test is the user's** (`skein/docs/foundation/testing-strategy.md`,
  8.1): no task raises a test's CPU budget, a profile's timeout or a tier's wall. The stock's
  spends, periods and charter times that a spec names are the user's figures (the draft) and are
  set as given; no other spend, period or time moves.
- **Figures are sizes the user decided:** each default holds the draft's table with 5x headroom.
  A task never lowers one of the draft's figures to make a check pass: where a check refuses a
  figure the spec gives, stop and ask with the check's words and the figure it needs. Where a
  spec says "the least that passes" for a coupled limit, take it and report it.
- **The resource gate is the user's:** no task measures it or acts on it but the one the plan
  names (`measure-box`). A task that sees a profile figure that preallocates memory or disk
  (a buffer allocated at its bound, a file written at its size) says so in its report, with the
  figure.
- **Test worlds:** each tree's own: jig's `jig/tests/<area>/` worlds and `jig/testing/`, smith's
  crates' `src/tests/`, temper's crates' `src/tests/`, `temper/crates/temper/tests/` (the
  `suite` binary) and `temper/tests/engine/...`. Fakes, never real services; no test reaches a
  network, a held login or a real repository. Each test is one thread and one loop, and a seed
  replays (`AGENTS.md`, 2).
- **Never run a live story:** no test of a `live*` or `proof_live` binary, with or without
  `--run-ignored`; `temper/crates/temper/tests/proof_live.rs` is edited only where a spec names
  its figures, and checked by `heavy cargo check -p temper --tests`.
- **Other plans beside this one** change some of the same files on their own trunks:
  `configuring-p3` (`temper-service`'s `stock.rs`, `limits.rs` and their tests,
  `jig-protocol-config`, jig's client wire v19 and its goldens, `temper-settings-form`),
  `public-reading-build` (`jig-protocol-config`, `temper-service/src/limits.rs`, people-v14),
  and `realistic-defaults`, an earlier edition of this draft. Keep each hunk to what the task
  needs: no reformatting, renaming or moving of code the task does not change. A conflict with
  what such a plan landed on main reaches this trunk by catch-up, not by you.
- No new crates, workspace members or dependencies beyond the spec's.
- Old-form counts only fall (`<tree>/scripts/old-forms.txt`); new text adds no old form (no
  "actual", "genuine", "originally").
- Every build and test through `heavy`, one at a time; the narrowest test first, and check the
  count it selected (nextest exits 4 when it selects nothing). Before pushing: `cargo fmt`, the
  old-form counts, `heavy cargo clippy -p PKG ... --all-targets -- -D warnings` naming every
  package you changed, then their focused tests and their direct dependents' (`AGENTS.md`, 6),
  cross-tree dependents by `heavy cargo check -p <dependent> --tests`.
- `temper ctl` reads stdin: a script or test that runs it gives it `</dev/null`.
- Before pushing a branch that touches Markdown, run
  `python3 -I -B tools/citations/citations.py --base origin/main` and fix what it lists in the
  files you own.
