# Agent guidance for fabrig

Provisional, 2026-10-28. What every agent working here must know, whatever its role. Read it first, then your role's file in `agents/`, then the `AGENTS.md` of the tree you work in, which holds the facts about its code.

## 1. In one page

- **One repository, four trees, one Cargo workspace:** `skein/` (the io and protocol kit, and the code's rules), `smith/` (the agent kit), `jig/` (the kit for agentic applications), `temper/` (the application). One `Cargo.lock`, path dependencies.
- **Dependencies point one way:** skein names none of smith, jig or temper; smith names neither jig nor temper; jig never names temper. A missing generic mechanism goes into skein before its smith or jig consumer. A gate step checks it.
- **The code's rules are skein's:** `skein/docs/foundation/programming-model.md` and `testing-strategy.md`, section 1 first, a later section when the work touches it; `notes.md` beside them holds the reasons. Depart only with a reason written in the design or the commit.
- **The design is the contract:** each tree's `docs/design/`, cited by path from the repository root and section (`skein/docs/design/lib.md, 5`). Every design document has a "Later" section for what is out of scope now, and a status file beside it (`lib.status.md`) saying where the code lags or departs; its designer keeps both.
- **main moves only through the merge queue.** You push a branch; you never push main, never force-push, and push only your own branches (a lead also publishes its implementers' reviewed tips as `ready/<area>/<task>`).
- **Every build and test goes through `heavy`**, one command at a time; the only test profiles are `default` (focused) and `fuzzy`.
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
- One task, one branch, one slot: branches are `<area>/<task>` (the coordinator's `coord/<task>`, the operator's `ops/<task>`), and a lead has as many in flight as it has implementers running; a branch is ready for the queue when pushed as `ready/<area>/<task>`. A rebased branch is pushed under a new name.
- Commits are small and say why; the merge queue keeps them as they are.
- Throwaway code lives untracked in your worktree and is gone before you push; a tool the work lacks is a question, never a script beside it.

## 5. Build machines

- Builders: hetzner3, 6, 7 (8 cores, 15 GB each; heavy admits three builds at once). Hetzner8 runs the gate; hetzner2 is reserved for the second gate, with no builder tasks. hel8 builds nothing.
- `heavy cargo ...` for every build and test; what builds nothing (`cargo fmt`, `cargo tree`, `cargo metadata`) runs directly. Never `heavy --gate`: the gate is the merge queue's.
- One command at a time, and its inputs stay still until it reports. Cancel with `heavy --cancel TOKEN` from the directory that submitted it; never stop heavy's units or touch its state.
- Disk: below 40 GB free heavy queues nothing. Free disk only in your own `target/`; never `kache clean --cache`, `~/.cache`, `~/.cargo`, another's worktree.
- Builds set `TEMPER_BUNDLE=none` (the machines' cargo config does).
- Remote commands run in a login shell (`bash -lc`), or tools aren't on PATH.

## 6. Tests

- After each change, the narrowest test that checks it, in the package that holds it: `heavy cargo nextest run -p PKG --test BIN -E 'test(=NAME)'`, or `--lib`. A filter alone still builds every test binary of the package.
- Check the count: a run that selected nothing proves nothing (nextest exits 4).
- Read a failure's log (heavy prints its path) before anything reruns; no unchanged rerun, except once after exit 137.
- A failure your change can't reach runs once alone: passing, it's a flake, reported with both logs and the machine; failing, it's a question.
- Fix a failure in the lowest tier that shows it: a skein fault seen downstream is reproduced in skein.
- The budgets, measured on an idle machine: a tree's focused suite within 30 seconds, its fuzzy suite (randomized: the `fuzzy` module of a package's `suite` binary) within 120 seconds. They are the user's; moving a test to escape one is raising it.

## 7. Asking

- A question is the first line of your report, `Question:`, with the options and your recommendation, then stop.
- The user's: budgets and timeouts, sign-ins and credentials, machines, and what is hard to reverse (a stored format, a public interface, a protocol). The coordinator asks the user; everyone else asks up their line.
- Before passing on what another agent said, check it against its report. Times come from `date -u`; what is unknown is `unknown`, never a guess.

## 8. Roles

`agents/coordinator.md` (one; designs and runs the pipeline), `agents/lead.md` (one per area at a time, ephemeral; design parts into tasks), `agents/implementer.md` (Codex; one task at a time), `agents/operator.md` (one; the merge queue, the machines, the tools and the ledger); `agents/operations.md` (the machines', slots', surveys' and queue's facts) and `agents/ledger.md` (each incident that cost throughput, and where its remedy lives) for every session that coordinates or leads. Agents talk through `post`, the coordinator at the hub: `post send [--as NAME] [--re ID] TO SUBJECT [FILE]` sends FILE (or stdin) to `NAME@MACHINE`; exit 0 or 3 is delivered, exit 2 is not; a body stays under 15 lines, and anything longer is a file whose path the message names. `post wait NAME` waits for mail, `post read NAME` reads it.
