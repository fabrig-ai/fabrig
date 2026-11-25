# Agent guidance for fabrig

Provisional, 2026-10-30. What every agent working here must know, whatever its role. An agent that starts work on a tree also reads that tree's `AGENTS.md`, and no other tree's.

## 1. In one page

- **One repository, five trees, one Cargo workspace:** `skein/` (the io and protocol kit, and the code's rules), `smith/` (the agent kit), `jig/` (the kit for agentic applications), `temper/` (the application), `hearth/` (immutable machines, built on skein alone). One `Cargo.lock`, path dependencies.
- **A tree's layout:** step crates in `crates/*`, their worlds in `tests/*`, shared test harnesses in `testing/*` (`skein/docs/design/testing/layout.md, 1`).
- **Dependencies point one way:** skein names none of smith, jig, temper or hearth; smith names neither jig nor temper; jig never names temper; hearth names skein alone, and nothing names hearth. A missing generic mechanism goes into skein before its smith, jig or hearth consumer. A gate step checks it.
- **fabrig names no tool, machine, session or repository of whoever runs its development;** `tests/boundary/tests/outward.rs` checks it.
- **The code's rules are skein's:** `skein/docs/foundation/programming-model.md` and `testing-strategy.md`, section 1 first, a later section when the work touches it; `notes.md` beside them holds the reasons. Depart only with a reason written in the design or the commit.
- **The design is the contract:** each tree's `docs/design/`, cited by path from the repository root and section (`skein/docs/design/runtime/lib-containers.md, 1`), and written from the tree's drafts. Read the sections your work cites; a status file beside each document (`skein/docs/design/runtime/lib-containers.status.md`) says where the code lags or departs. Writing or reviewing design follows `AGENTS.md`, section 6.
- **On a doubt, stop and ask**; never improvise across a boundary.

## 2. Code

- A lint exception is a scoped `#[expect(..., reason = "...")]`, never `#[allow]`.
- Comments say what is true now, in the design's words: no history ("originally"), no emphasis words ("actual", "genuine").
- Code cites a design section at module or type level, not on every field.
- Step crates follow the strict subset (`skein/docs/foundation/programming-model.md, 10`): no `std`, no `unsafe` but where a module says why, every `match` lists every variant (no `_`, no `matches!`), no indexing, no unchecked arithmetic, no `as`, no `unwrap`. Only 10.2's exceptions (lib, the codecs' helpers and generated implementations) are generic code. The workspace's lints and each tree's `clippy.toml` hold it.
- A package's `[lints]` block and its `clippy.toml` are copies of a definition (`tools/lints/design.md`): a new crate takes a profile with `tools/lints/lints.py --set`, and the gate holds it there.
- Test crates are ordinary Rust under their tree's `tests/clippy.toml`: one thread, one loop, and a seed replays.
- A change to any family's schema, stored or sent, moves no version and brings no patch: every family's version is frozen, a layout is named by its fingerprint, and a value at another layout is converted by a plan derived from the two texts (`skein/docs/design/data/codec/README.md`, 5). An added field takes a default (`= value`), a renamed part its former name (`was`), a dropped field its tombstone (`skein/docs/design/data/codec/README.md`, 3), and a change no plan derives is its owner's: a store's written job (`jig/docs/design/protocol/store/layouts.md`, 6) or a wire's written converter (`jig/docs/design/protocol/channel/layouts.md`, 6). A crate keeps its families' current texts alone, never an older one. fabrig keeps no code for a layout older than what dev.fabrig.ai runs (`temper/docs/design/draft/archive/store-floor.md`; `temper/docs/design/draft/layouts-both-ways.md`, 7.4): a job, a converter or a tombstone goes once dev.fabrig.ai has passed the layout that needed it, and a format outside the registry (the component's own rows, the secrets' store) is fixed at its version, a change to one designing, in its own landing, how dev.fabrig.ai's store gets there.
- A change to a deployment's configuration (`temper.toml`, a worker's configuration, a small file a build reads at its start) keeps the previous release's form readable: a member that moves or is renamed is read from its old place too, with one warning naming the new one, until dev.fabrig.ai runs a build that reads the new form; a change no plan derives brings a written converter, tested against a fixture of the previous release's form, never a deployment's own configuration (`temper/docs/design/draft/config.md`, 8).
- A generated file under a crate's `src/generated/` is never edited or merged by hand: its schema, and for a translation its maps file too, are its specification. The merge job makes it again from those inputs where the schema texts, maps and generator merged clean (`temper/docs/design/protocol/tooling.md`, 4.1); where they conflict too, merge the schema and maps texts, regenerate the file once from the inputs in `tools/merge/codecs.txt`, using `skein-codegen` or the family's drift test in write mode. Verify the result with the family's drift test with `SKEIN_CODEGEN_WRITE` unset.
- Some Markdown is read by tests (`.config/gate/read-by-tests.txt`): a docs change there needs its tests.
- Public code names no path or address of a machine outside the repository, and nothing private: emails, UUIDs, session names, tokens.

## 3. Security

- Never read a key, token, `auth.json`, `.credentials.json`, a credential record or a box's secrets; check such a file by `stat` and sha256 at most.
- A token reaches a request through a 0600 file (a curl config, a header file), never argv; keys sit 0600 in a 0700 directory, never in a worktree or a log.
- No probes against real services or repositories; never read another process's environment (`/proc/*/environ`, `ps e`).

## 4. Git

- Commits are small and say why; merges keep them as they are.
- Throwaway code lives untracked in your worktree and is gone before you push; a tool the work lacks is a question, never a script beside it.
- **A journey's recording and pages have one writer.** Under `temper/docs/design/client/web/journeys/` and `temper/docs/design/client/tui/journeys/`, each `recording`, each page and the files printed beside them are written on main by the journeys' writer alone, after each landing (`temper/docs/design/client/journey-writer.md`). A change commits none: it edits a journey's `README.md`, `walk` or `scripts.json` as it edits a test, compares pages by a print into scratch (`cargo run -p temper-client-walk -- print --out DIR`), then restores them. The delivery check refuses a delivery that changes one beside any other file.

## 5. Tests

- After each change, run the narrowest test that checks it in the package that holds it with `tools/tiers/test -p PKG --test BIN -E 'test(=NAME)'`, or use `--lib`. It takes nextest's arguments, counts each test where available and shows the layer's allowance once main's counts are fetched. A filter alone still builds every test binary of the package.
- Development builds and tests set `TEMPER_INTERNAL_BUNDLE=none`: without it, a build of `temper` builds the browser's bundle for wasm32 in a nested cargo (`temper/docs/design/client/web/bundle.md`, 1).
- Check the count: a run that selected nothing proves nothing (nextest exits 4).
- Read a failure's log before anything reruns; no unchanged rerun, except once after exit 137.
- A failure your change can't reach runs once alone: passing, it's a flake, reported with both logs and the machine; failing, it's a question.
- Fix a failure in the lowest tier that shows it: a skein fault seen downstream is reproduced in skein.
- **While a plan is being built, only the focused and fuzzy suites run.** The browser, process and live tiers run after landing: a new story in one is written and compiled, not run, and a red they raise is repaired then (the user, 2026-11-06). That repair may run the test it repairs, by name, alone.
- Every tree's tier has the user's CPU budget and a wall hard limit; **starved** is a warning, never red on that account (skein/docs/foundation/testing-strategy.md, 8.1).
  Over budget is red: the gate refuses a layer over its budget or grown past its allowance, and the goal's close refuses the growth too (`tools/tiers/budgets.md`, 8); a test past its reference only warns (the user, 11-19). To repair it, read `skein/docs/foundation/testing-strategy.md` in full first, then ask: does the test belong here; can it be made faster, which means the same checks for less; otherwise ask the user, with measurements. Moving a test to escape a budget is raising it, and so is checking less in any form: a lower sample, fewer seeds, cases or assertions, a test removed or ignored. Every figure in `tools/tiers/budgets.toml` is the user's: a change never adds or raises one. The one line a change may write there itself is a check reason (`tools/tiers/budgets.md`, 13), when it does fewer checks for a reason that answers no budget: the tested code is gone, another test makes the same checks, or a move to its tier.

## 6. Design documents

How drafts, design documents, their status files and execution files are written and kept is `DESIGN.md`, for whoever writes or reviews design.
