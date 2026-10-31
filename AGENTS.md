# Agent guidance for fabrig

Provisional, 2026-10-30. What every agent working here must know, whatever its role. Read it first, then the `AGENTS.md` of the tree you work in, which holds the facts about its code. Your role's instructions come from fab (section 8).

## 1. In one page

- **One repository, four trees, one Cargo workspace:** `skein/` (the io and protocol kit, and the code's rules), `smith/` (the agent kit), `jig/` (the kit for agentic applications), `temper/` (the application). One `Cargo.lock`, path dependencies.
- **Dependencies point one way:** skein names none of smith, jig or temper; smith names neither jig nor temper; jig never names temper. A missing generic mechanism goes into skein before its smith or jig consumer. A gate step checks it.
- **The code's rules are skein's:** `skein/docs/foundation/programming-model.md` and `testing-strategy.md`, section 1 first, a later section when the work touches it; `notes.md` beside them holds the reasons. Depart only with a reason written in the design or the commit.
- **The design is the contract:** each tree's `docs/design/`, cited by path from the repository root and section (`skein/docs/design/lib.md, 5`). Every design document has a "Later" section for what is out of scope now, and a status file beside it (`lib.status.md`) saying where the code lags or departs; its designer keeps both.
- **main moves only through fab.** You push your own branch; fab merges it into the run's trunk, checks the trunk and lands what is checked. You never push main or a trunk, never force-push, and push only your own branch.
- **Old-form counts only fall.** A task lowers `scripts/old-forms.txt` with `old_forms.py --write` for what it removes; fab recounts the file where merges meet. A raised count is red.
- **Every build and test goes through `heavy`**, one command at a time; `default` (focused) and `fuzzy` are the gate's profiles; the slow tiers use the root configuration's `<tier>-<tree>` profiles (`tools/tiers/run`), never in the gate.
- **On a doubt, stop and ask** (section 7); never improvise across a boundary.

## 2. Code

- A lint exception is a scoped `#[expect(..., reason = "...")]`, never `#[allow]`.
- Comments say what is true now, in the design's words: no history ("originally"), no emphasis words ("actual", "genuine").
- Code cites a design section at module or type level, not on every field.
- Test crates are ordinary Rust under their tree's `tests/clippy.toml`: one thread, one loop, and a seed replays.
- Each tree keeps its `clippy.toml` (smith's bans more types); its `AGENTS.md` lists its code facts (smith's module-doc rules, jig's "the core never links smith", temper's `docs/help/` compiled into its help tests, and so on).
- Some Markdown is read by tests (skein's UI fixtures and conformance designs, smith's design and fixture trees, jig's `protocol/store.md`, temper's `docs/help/`): a docs change there needs its tests.
- Public code names no `qa` path or address, and nothing private: emails, UUIDs, session names, tokens.

## 3. Security

- Never read a key, token, `auth.json`, `.credentials.json`, a credential record or a box's secrets; check such a file by `stat` and sha256 at most.
- A token reaches a request through a 0600 file (a curl config, a header file), never argv; keys sit 0600 in a 0700 directory, never in a worktree or a log.
- A login (Claude, Codex, ChatGPT) is never copied between machines; signing in is the user's.
- No probes against real services or repositories; never read another process's environment (`/proc/*/environ`, `ps e`).
- Search only your worktree and the paths your task names: never `~/.local`, `~/.config`, `/run/user`, `~/src/rust/tmp/heavy/`.
- Codex runs with umask 077, its logs 0600, its stdin `</dev/null`.

## 4. Git

- fabrig's clones, slots and scratch live under `~/src/fabrig/` on every machine (slots at `~/src/fabrig/slots/<n>`), nothing above them but this repository's own files.
- You work in your slot or worktree, on a branch; no git command in anyone else's checkout, and none in a primary checkout, not even `git fetch`.
- One task, one branch, one slot: fab names the branch (`<draft>/<task>`) and sets up the slot. A rebased branch is pushed under a new name.
- Commits are small and say why; merges keep them as they are.
- Throwaway code lives untracked in your worktree and is gone before you push; a tool the work lacks is a question, never a script beside it.

## 5. Build machines

- The builders are fab's (`fab/hosts.toml` in the agents repository); hel8 builds nothing.
- `heavy cargo ...` for every build and test; what builds nothing (`cargo fmt`, `cargo tree`, `cargo metadata`) runs directly. Never `heavy --gate`: the gate is the merge queue's.
- One command at a time, and its inputs stay still until it reports. Never stop heavy's units or touch its state.
- Disk: heavy waits for disk by itself, reclaiming idle slots, up to 20 minutes. Its exit 75 is `Infra: disk`: report it, never work around it. Free disk only in your own `target/`; never `kache clean --cache`, `~/.cache`, `~/.cargo`, another's worktree.
- **Every machine gives passwordless sudo:** use it where a task truly needs root, and never to get round a rule: heavy's units and state, another's checkout, credentials and keys stay off limits as above.
- Builds set `TEMPER_BUNDLE=none` (the machines' cargo config does).
- Remote commands run in a login shell (`bash -lc`), or tools aren't on PATH.

## 6. Tests

- After each change, the narrowest test that checks it, in the package that holds it: `heavy cargo nextest run -p PKG --test BIN -E 'test(=NAME)'`, or `--lib`. A filter alone still builds every test binary of the package.
- Check the count: a run that selected nothing proves nothing (nextest exits 4).
- Read a failure's log (heavy prints its path) before anything reruns; no unchanged rerun, except once after exit 137.
- A failure your change can't reach runs once alone: passing, it's a flake, reported with both logs and the machine; failing, it's a question.
- Fix a failure in the lowest tier that shows it: a skein fault seen downstream is reproduced in skein.
- Every tree's tier has the user's CPU budget and a wall hard limit; **starved** is a warning, never red on that account (skein/docs/foundation/testing-strategy.md, 8.1).
  Over budget is red: does the test belong here; can it be made faster; otherwise ask the user through the director with measurements. Moving a test to escape a budget is raising it.

## 7. Asking

- A question is the first line of your report, `Question:`, with the options and your recommendation, then stop.
- The user's: budgets and timeouts, sign-ins and credentials, machines, and what is hard to reverse (a stored format, a public interface, a protocol). fab carries every question to whoever answers it.
- Before passing on what another agent said, check it against its report. Times come from `date -u`; what is unknown is `unknown`, never a guess.

## 8. Roles

fab runs the pipeline (the agents repository's `docs/design/fab.md`): a cut, a Claude call, turns a draft into a plan and task specs; implementers are Codex runs, one task in one slot; answer calls settle questions; fab merges, checks and lands. Each gets its instructions from fab. The pre-fab pipeline's role files (lead, implementer, coordinator, operator, operations, ledger, backlog) are kept for reference only in the agents repository, under `old/fabrig/`: they give no instructions.
