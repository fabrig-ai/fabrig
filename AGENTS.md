# Agent guidance for fabrig

Provisional, 2026-10-30. What every agent working here must know, whatever its role. An agent that starts work on a tree also reads that tree's `AGENTS.md`, and no other tree's.

## 1. In one page

- **One repository, five trees, one Cargo workspace:** `skein/` (the io and protocol kit, and the code's rules), `smith/` (the agent kit), `jig/` (the kit for agentic applications), `temper/` (the application), `hearth/` (immutable machines, built on skein alone). One `Cargo.lock`, path dependencies.
- **A tree's layout:** step crates in `crates/*`, their worlds in `tests/*`, shared test harnesses in `testing/*` (`skein/docs/design/testing/README.md, 6`).
- **Dependencies point one way:** skein names none of smith, jig, temper or hearth; smith names neither jig nor temper; jig never names temper; hearth names skein alone, and nothing names hearth. A missing generic mechanism goes into skein before its smith, jig or hearth consumer. A gate step checks it.
- **fabrig names no tool, machine, session or repository of whoever runs its development;** `tests/boundary/tests/outward.rs` checks it.
- **The code's rules are skein's:** `skein/docs/foundation/programming-model.md` and `testing-strategy.md`, section 1 first, a later section when the work touches it; `notes.md` beside them holds the reasons. Depart only with a reason written in the design or the commit.
- **The design is the contract:** each tree's `docs/design/`, cited by path from the repository root and section (`skein/docs/design/runtime/lib.md, 5`), and written from the tree's drafts. Read the sections your work cites; a status file beside each document (`skein/docs/design/runtime/lib.status.md`) says where the code lags or departs. Writing or reviewing design follows section 6.
- **On a doubt, stop and ask**; never improvise across a boundary.

## 2. Code

- A lint exception is a scoped `#[expect(..., reason = "...")]`, never `#[allow]`.
- Comments say what is true now, in the design's words: no history ("originally"), no emphasis words ("actual", "genuine").
- Code cites a design section at module or type level, not on every field.
- Step crates follow the strict subset (`skein/docs/foundation/programming-model.md, 10`): no `std`, no `unsafe` but where a module says why, every `match` lists every variant (no `_`, no `matches!`), no indexing, no unchecked arithmetic, no `as`, no `unwrap`. The workspace's lints and each tree's `clippy.toml` hold it.
- A package's `[lints]` block and its `clippy.toml` are copies of a definition (`tools/lints/design.md`): a new crate takes a profile with `tools/lints/lints.py --set`, and the gate holds it there.
- Test crates are ordinary Rust under their tree's `tests/clippy.toml`: one thread, one loop, and a seed replays.
- A change to a stored family's schema brings its patch, `cargo run -p skein-codegen -- patch <crate>`, and the golden store's line (`temper/docs/design/protocol/baseline.md`, 7); `fabrig-boundary`'s `patches` test holds it.
- Some Markdown is read by tests (`.config/gate/read-by-tests.txt`): a docs change there needs its tests.
- Public code names no path or address of a machine outside the repository, and nothing private: emails, UUIDs, session names, tokens.

## 3. Security

- Never read a key, token, `auth.json`, `.credentials.json`, a credential record or a box's secrets; check such a file by `stat` and sha256 at most.
- A token reaches a request through a 0600 file (a curl config, a header file), never argv; keys sit 0600 in a 0700 directory, never in a worktree or a log.
- No probes against real services or repositories; never read another process's environment (`/proc/*/environ`, `ps e`).

## 4. Git

- Commits are small and say why; merges keep them as they are.
- Throwaway code lives untracked in your worktree and is gone before you push; a tool the work lacks is a question, never a script beside it.

## 5. Tests

- After each change, the narrowest test that checks it, in the package that holds it: `cargo nextest run -p PKG --test BIN -E 'test(=NAME)'`, or `--lib`. A filter alone still builds every test binary of the package.
- Development builds and tests set `TEMPER_INTERNAL_BUNDLE=none`: without it, a build of `temper` builds the browser's bundle for wasm32 in a nested cargo (`temper/docs/design/client/web/README.md`, 2.6).
- Check the count: a run that selected nothing proves nothing (nextest exits 4).
- Read a failure's log before anything reruns; no unchanged rerun, except once after exit 137.
- A failure your change can't reach runs once alone: passing, it's a flake, reported with both logs and the machine; failing, it's a question.
- Fix a failure in the lowest tier that shows it: a skein fault seen downstream is reproduced in skein.
- **While a plan is being built, only the focused and fuzzy suites run.** The browser, process and live tiers run after landing: a new story in one is written and compiled, not run, and a red they raise is repaired then (the user, 2026-11-06). That repair may run the test it repairs, by name, alone.
- Every tree's tier has the user's CPU budget and a wall hard limit; **starved** is a warning, never red on that account (skein/docs/foundation/testing-strategy.md, 8.1).
  Over budget is red: does the test belong here; can it be made faster; otherwise ask the user, with measurements. Moving a test to escape a budget is raising it.

## 6. Design documents

How drafts, design documents, their status files and execution files are written and kept is `DESIGN.md`, for whoever writes or reviews design.
