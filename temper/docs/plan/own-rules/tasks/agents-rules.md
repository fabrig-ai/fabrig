+++
id = "agents-rules"
part = "agents"
kind = "docs"
needs = []
files = ["AGENTS.md", "DESIGN.md", "skein/AGENTS.md", "temper/AGENTS.md", "tests/boundary/outward.txt", "tests/boundary/tests/outward.rs", "tools/citations/README.md", "temper/docs/design/README.md", "temper/docs/design/domain/testing.md", "temper/docs/design/draft/notes.md", "temper/docs/development/debt/D008-pipeline-names-in-code.md", "jig/docs/development/debt/D014-task-record-union.md"]
effort = "medium"
minutes = 30
+++
# The root AGENTS.md and DESIGN.md hold fabrig's own rules only

## Goal

The root `AGENTS.md` mixes fabrig's own rules with the process of the lab that runs fabrig's
development (its pipeline, build tool, machines, slots, how to ask). Rewrite it to fabrig's
rules only, renumbered 1 to 6, with one new line for the outward check; reword `DESIGN.md`'s
two lab sentences and the trees' `AGENTS.md` lines that name the lab; fix every citation of
`AGENTS.md, N` the renumbering moves; lower the outward inventory. Wording only.

Acceptance:

- `AGENTS.md` reads exactly as step 1 gives it, but for kept lines your base changed (Options).
- `DESIGN.md`, `skein/AGENTS.md`, `temper/AGENTS.md` and the citing lines of step 4 read as
  steps 2 to 5 give them; `git diff` changes prose and one doc comment only.
- After your last commit, `python3 -I -B tools/citations/citations.py --all` reads 0 and
  `--base origin/main` reports nothing new.
- `tests/boundary/outward.txt` lists none of `AGENTS.md`, `DESIGN.md`, `skein/AGENTS.md`, and
  `heavy cargo nextest run -p fabrig-boundary --test outward` passes, 10 tests.
- `cargo fmt --check -p fabrig-boundary` and
  `heavy cargo clippy -p fabrig-boundary --all-targets -- -D warnings` pass.

## Inputs

- `temper/docs/plan/own-rules/notes/common.md`: the rules every task here keeps.
- `temper/docs/plan/own-rules/inputs/request.md`, items 1, 2 and Bounds: the request.
- `DESIGN.md`, 4: whoever renumbers fixes every citation.
- `tests/boundary/tests/outward.rs`, its module doc: what the outward check counts.
- `temper/docs/design/client/browser.md`, 2.6: what `TEMPER_BUNDLE` does (cited by the new
  `AGENTS.md`, 5).

## Owns and may change

- Owns `AGENTS.md` whole; in the other files of `files`, only the lines steps 2 to 6 name.
- **Departures from your implementer's text** (`notes/common.md`): you edit lines of
  `temper/docs/design/README.md`, `temper/docs/design/domain/testing.md` and
  `temper/docs/design/draft/notes.md`, the citations of step 4 only. No Rust code changes:
  `outward.rs` changes in one doc comment.
- Not yours: `tools/citations/README.md` but its line 3 (lines 4 and on are a parallel plan's,
  `outward-citations`); `temper/docs/design/domain/testing.md` but the line of step 4
  (around line 1368 is `landing-contract`'s); other entries of `tests/boundary/outward.txt`.
- Options, pre-approved:
  - **`AGENTS.md` at your base differs from `git show d8f6f26e6d:AGENTS.md`:** if only lines
    step 1 deletes differ (section 8's sentence lost its parenthesis in `outward-citations`),
    write step 1's text. If a line step 1 keeps differs, keep your base's words for that line
    in step 1's place and say so.
  - **A citation of `AGENTS.md` that step 4 does not list** shows in `citations.py` or a search:
    fix it by the same map (old 1 to 4 keep their numbers; 6 becomes 5; 9 becomes 6; a
    citation of 5, 7 or 8 is dropped and the sentence keeps its own words) and list it.
  - **An `outward.txt` entry of the three is already lower or gone at your base:** remove what
    is left of it. If the outward test finds a name on a line you wrote, reword your line.
  - **Rewrapping:** `notes/common.md`.

## Steps

1. **`AGENTS.md`**, whole, becomes this text (between the fences; lines unchanged from the
   base are copied, not retyped: sections 1 to 4 keep their surviving lines in order, old
   section 6 becomes 5, old 9 becomes 6, old 5, 7 and 8 go):

   ````markdown
   # Agent guidance for fabrig

   Provisional, 2026-10-30. What every agent working here must know, whatever its role. Read it first, then the `AGENTS.md` of the tree you work in, which holds the facts about its code.

   ## 1. In one page

   - **One repository, four trees, one Cargo workspace:** `skein/` (the io and protocol kit, and the code's rules), `smith/` (the agent kit), `jig/` (the kit for agentic applications), `temper/` (the application). One `Cargo.lock`, path dependencies.
   - **Dependencies point one way:** skein names none of smith, jig or temper; smith names neither jig nor temper; jig never names temper. A missing generic mechanism goes into skein before its smith or jig consumer. A gate step checks it.
   - **fabrig names no tool, machine, session or repository of whoever runs its development;** `tests/boundary/tests/outward.rs` checks it.
   - **The code's rules are skein's:** `skein/docs/foundation/programming-model.md` and `testing-strategy.md`, section 1 first, a later section when the work touches it; `notes.md` beside them holds the reasons. Depart only with a reason written in the design or the commit.
   - **The design is the contract:** each tree's `docs/design/`, cited by path from the repository root and section (`skein/docs/design/lib.md, 5`), and written from the tree's drafts. Read the sections your work cites; a status file beside each document (`lib.status.md`) says where the code lags or departs. Writing or reviewing design follows section 6.
   - **On a doubt, stop and ask**; never improvise across a boundary.

   ## 2. Code

   - A lint exception is a scoped `#[expect(..., reason = "...")]`, never `#[allow]`.
   - Comments say what is true now, in the design's words: no history ("originally"), no emphasis words ("actual", "genuine").
   - Code cites a design section at module or type level, not on every field.
   - Test crates are ordinary Rust under their tree's `tests/clippy.toml`: one thread, one loop, and a seed replays.
   - Each tree keeps its `clippy.toml` (smith's bans more types); its `AGENTS.md` lists its code facts (smith's module-doc rules, jig's "the core never links smith", temper's `docs/help/` compiled into its help tests, and so on).
   - Some Markdown is read by tests (skein's UI fixtures and conformance designs, smith's design and fixture trees, jig's `protocol/store.md`, temper's `docs/help/`): a docs change there needs its tests.
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
   - Development builds and tests set `TEMPER_BUNDLE=none`: without it, a build of `temper` builds the browser's bundle for wasm32 in a nested cargo (`temper/docs/design/client/browser.md`, 2.6).
   - Check the count: a run that selected nothing proves nothing (nextest exits 4).
   - Read a failure's log before anything reruns; no unchanged rerun, except once after exit 137.
   - A failure your change can't reach runs once alone: passing, it's a flake, reported with both logs and the machine; failing, it's a question.
   - Fix a failure in the lowest tier that shows it: a skein fault seen downstream is reproduced in skein.
   - Every tree's tier has the user's CPU budget and a wall hard limit; **starved** is a warning, never red on that account (skein/docs/foundation/testing-strategy.md, 8.1).
     Over budget is red: does the test belong here; can it be made faster; otherwise ask the user, with measurements. Moving a test to escape a budget is raising it.

   ## 6. Design documents

   How drafts, design documents, their status files and execution files are written and kept is `DESIGN.md`, for whoever writes or reviews design.
   ````

   `CLAUDE.md` stays `@AGENTS.md`, untouched.

2. **`DESIGN.md`:**
   - Lines 3-8 become:

     ```markdown
     What whoever writes or reviews design must know, beside `AGENTS.md`:
     how each tree's drafts, design documents, status files and execution
     files are written and kept. Whoever writes or reviews design reads
     this file; everyone else needs only `AGENTS.md`, section 1: the
     design is the contract, cited by path from the repository root and
     section.
     ```

   - Line 42's first sentence, "Citations are checked by code, not by hand: the queue rejects a
     branch with any broken path, section or status-label citation (`tools/citations/`).",
     becomes "Citations are checked by code, not by hand: a branch with a broken path, section
     or status-label citation does not land (`tools/citations/`)." The rest of the item stays.

3. **The trees' `AGENTS.md`:**
   - `skein/AGENTS.md`, 9: its code span `` `heavy cargo check -p skein-tls` `` becomes
     `` `cargo check -p skein-tls` ``; the rest of the line stays.
   - `temper/AGENTS.md`, 6: the line ends "so a help change needs those tests." (the clause
     "; leads write help pages, never implementers" goes: it is the lab's division of roles).
   - `smith/AGENTS.md` and `jig/AGENTS.md` name nothing of the lab: unchanged.

4. **Citations the renumbering moves or drops** (found with
   `git grep -n -E "AGENTS\.md[^a-zA-Z]{0,25}[0-9]|AGENTS\.md\`?, section"` at the base; the
   other hits cite sections 1 and 2, which keep their numbers, or sit in a dated review the
   checker leaves out):
   - `tools/citations/README.md`, 3: `(AGENTS.md, 9)` becomes `(AGENTS.md, 6)`.
   - `temper/docs/design/README.md`, 194: `` (the repository's `AGENTS.md`, section 9) ``
     becomes `section 6`.
   - `temper/docs/design/domain/testing.md`, 2160: `` (the repository's `AGENTS.md`,
     section 6) `` becomes `section 5`.
   - `temper/docs/design/draft/notes.md`, 3: `` (`AGENTS.md`, section 9) `` becomes
     `section 6`.
   - `temper/docs/design/draft/notes.md`, 454: "a login is never copied to make Codex a program
     (`AGENTS.md`, 3)." loses its citation (section 3 no longer holds the login rule): "a login
     is never copied to make Codex a program." The next sentence stays; rewrap the item.
   - `temper/docs/development/debt/D008-pipeline-names-in-code.md`, 49: "The pipeline that
     numbered those tasks is gone (AGENTS.md, section 8):" becomes "The pipeline that numbered
     those tasks is gone:".
   - `jig/docs/development/debt/D014-task-record-union.md`, 29: "the user's call
     (`AGENTS.md`, 7) and" becomes "the user's call and".

5. **`tests/boundary/tests/outward.rs`**, its module doc, lines 1-2: `(AGENTS.md, 2)` becomes
   `(AGENTS.md, 1)`, the new line of step 1 being the rule it checks; rewrap that sentence only
   if `cargo fmt` or the width asks.

6. **`tests/boundary/outward.txt`:** the files above then hold no name (at the base:
   `AGENTS.md` 24, `DESIGN.md` 1, `skein/AGENTS.md` 1, all on lines this task deletes or
   rewords). Remove those three entries; every other entry stays as it is
   (`tools/citations/README.md` 2, `temper/docs/design/domain/testing.md` 2,
   `temper/docs/design/draft/notes.md` 18 and `D008-pipeline-names-in-code.md` 1 keep names on
   lines this task does not touch).

7. **Checks,** once, after the edits (`notes/common.md`):
   - `cargo fmt --check -p fabrig-boundary`;
   - commit, then `python3 -I -B tools/citations/citations.py --all` (0) and
     `python3 -I -B tools/citations/citations.py --base origin/main` (nothing new);
   - `python3 -I -B scripts/old_forms.py` from the `skein`, `jig` and `temper` roots (no count
     moves: only Markdown changed there);
   - `heavy cargo clippy -p fabrig-boundary --all-targets -- -D warnings`;
   - `heavy cargo nextest run -p fabrig-boundary --test outward` (10 tests).

8. Commit (one commit per step is fine; say why in each) and push your branch.

## Report

As your implementer's text and `notes/common.md` say, and: the `AGENTS.md` lines your base
changed against `d8f6f26e6d`, if any, and what you kept; every citation you fixed beyond
step 4; the outward test's "Found N names in M files." line.
