# public-reading-build: standing rules

Every task of this plan names this note, and `notes/contracts.md` beside it, which holds the
shapes that cross tasks: build to it exactly.

- **The plan:** v0.1.0's public reading, as revised after its security review
  (`temper/docs/plan/public-reading-build/inputs/review.md`): jig's generic part
  (`jig/docs/design/domain/people.md`, section 13; `jig/docs/design/protocol/client.md`, 17.5
  and section 19; `jig/docs/design/client/domain/contract.md`, section 12) and temper's
  (`temper/docs/design/client/forge.md`, section 11; `client/browser.md`, section 8;
  `protocol/sign-in.md`, 2.4 and 2.5; `client/tools.md`, 5.3). jig never names temper
  (`AGENTS.md`, 1): no story, document or name of temper's goes in jig's code or tests.
- **The rules that are the point:** a public answer holds no text but the forge's link
  addresses, no configured name and no internal number; every task it names is public by
  people.md 13.7's one rule, filtered before anything is counted or ordered; a project that
  is private, being deleted or never made, and a task that is not public, are one answer,
  unknown, reached by the same path with the same work. A test that would pass with any of
  these broken is not done.
- **The tree:** your spec's, but for the lines it names in another tree, each edited only as
  it says. Read `AGENTS.md`, then the tree's `AGENTS.md`.
- **The design is binding:** the sections each spec cites, as they stand on your base. Never
  edit `docs/design/`, drafts, `docs/help/` (unless your spec owns a help page) or this plan's
  directory; what the code taught goes in the report. Each spec says which status entries it
  closes: none; the plan's last task updates the statuses.
- **The code's rules:** `skein/docs/foundation/programming-model.md`, section 1, and the
  workspace's lints (no float arithmetic, no `as`, no indexing, no unchecked arithmetic, no
  `unwrap`, no `todo!`, every `match` naming every variant). Bounded collections as the crate
  around you uses them.
- **Off by default:** every profile's public figures are zero; no test turns public reading
  on in a world that is not about it, so no other story changes.
- **Tests:** step tests in the crate's `src/tests/`, as the crate keeps them; worlds under
  their tree's `tests/`, under `tests/clippy.toml`: one thread, one loop, a seed replays, the
  seed named in every failure. A "fuzzy test" is a seeded sweep in a world's
  `tests/suite/fuzzy/` module, as `jig/tests/client-wire/tests/suite/fuzzy/client_wire.rs`
  is, run by the `fuzzy` profile.
- **Generated code:** schemas go through skein-codegen as their families do
  (`jig/tests/codec-families.rs`, temper's `temper/tests/codecs`); regenerate with the drift
  test's `SKEIN_CODEGEN_WRITE=1` and commit what it writes. Never hand-edit generated code.
- **One writer per file:** two tasks of this plan may work in one crate at once only on the
  files their headers list. A change to a file or a public item another task owns is a
  question, unless your spec pre-approves it.
- **No public item** beyond your spec's and the contracts'; private helpers are yours. A
  compiler-forced arm or field in your tree is always allowed, and listed in the report.
- **Old-form counts only fall:** run `python3 -I -B <tree>/scripts/old_forms.py` for each tree
  you touch.
- **Every build and test through `heavy`,** one at a time, the narrowest first; check each
  run's count (nextest exits 4 when it selects nothing). Before pushing: `cargo fmt`,
  `heavy cargo clippy -p PKG ... --all-targets -- -D warnings` naming every package you
  changed, their focused tests and their direct dependents' in every tree (`AGENTS.md`, 6),
  and `python3 -I -B tools/citations/citations.py --base origin/main`.
- **Other plans beside this one:** configuring-p2-build (on its trunk, not yet on main) edits
  `jig-client-protocol-wire`'s v19 schema, generated code and goldens, `contract/asks.rs`,
  the engine-clients' and link's `translate/asks.rs` and the store's people family; keep your
  hunks small and local, append variants last, and never reformat code your task does not
  change.
