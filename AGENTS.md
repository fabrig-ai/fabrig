# Agent guidance for fabrig

What every agent working here must know. Read it first, then the `AGENTS.md` of the tree you work in, which holds the facts about its code.

## 1. In one page

- **One repository, four trees, one Cargo workspace:** `skein/` (the io and protocol kit, and the code's rules), `smith/` (the agent kit), `jig/` (the kit for agentic applications), `temper/` (the application). One `Cargo.lock`, path dependencies.
- **Dependencies point one way:** skein names none of smith, jig or temper; smith names neither jig nor temper; jig never names temper. A missing generic mechanism goes into skein before its smith or jig consumer.
- **The code's rules are skein's:** `skein/docs/foundation/programming-model.md` and `testing-strategy.md`, section 1 first, a later section when the work touches it; `notes.md` beside them holds the reasons. Depart only with a reason written in the design or the commit.
- **The design is the contract:** each tree's `docs/design/`, cited by path from the repository root and section (`skein/docs/design/lib.md, 5`). Code must be rebuildable from it at a high level. Every design document has a "Later" section for what is out of scope now, and a status file beside it (`lib.status.md`) saying where the code lags or departs. A change that alters a design changes its document and status file in the same branch.
- **Old forms may only fall:** each tree's `scripts/old_forms.py` counts uses of old forms against `scripts/old-forms.txt`; a count may fall (`python3 -I -B scripts/old_forms.py --write` lowers it), never rise.
- **Stored formats and wire formats are free to change:** fabrig is pre-release with no deployment, so no backward compatibility and no version bump are needed.
- **Every build and test goes through `heavy`**, one command at a time; `default` (focused) and `fuzzy` are the nextest profiles.

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
- A login (Claude, Codex, ChatGPT) is never copied between machines.
- No probes against real services or repositories; never read another process's environment (`/proc/*/environ`, `ps e`).
- Search only your worktree and the paths your work names: never `~/.local`, `~/.config`, `/run/user`, `~/src/rust/tmp/heavy/`.

## 4. Git

- fabrig's clones, slots and scratch live under `~/src/fabrig/` on every machine (slots at `~/src/fabrig/slots/<n>`).
- No git command in anyone else's checkout, and none in a primary checkout (`~/src/fabrig/main`), not even `git fetch`.
- Push only branches named `free/<name>`. Never push main, never force-push, never push `ready/*`.
- Commits are small and say why.

## 5. Build machines

- Builders: hetzner6 (`free@2.29.120.8`; 8 cores, 15 GB) and hetzner9 (`free@2.29.101.185`; 8 vCPU, 15 GB), heavy admitting four builds at once on each. Their warm checkouts are the slots, where builds and tests run: `fab slot list`, `fab slot claim N RUN TASK` before using slot N, `fab slot release N RUN` after. Everything else (checkouts, worktrees, read-only work) is free. hel8, where you run, builds nothing.
- `heavy cargo ...` for every build and test; what builds nothing (`cargo fmt`, `cargo tree`, `cargo metadata`) runs directly. Never `heavy --gate`.
- One command at a time per checkout, and its inputs stay still until it reports. Never stop heavy's units or touch its state.
- Disk: heavy waits for disk by itself, up to 20 minutes. Its exit 75 is `Infra: disk`. Free disk only in your own `target/`; never `kache clean --cache`, `~/.cache`, `~/.cargo`, another's worktree.
- Every machine gives passwordless sudo: use it where the work truly needs root, and never to get round a rule: heavy's units and state, another's checkout, credentials and keys stay off limits as above.
- Builds set `TEMPER_BUNDLE=none` (the machines' cargo config does).
- Remote commands run in a login shell (`bash -lc`), or tools aren't on PATH.

## 6. Tests

- After each change, the narrowest test that checks it, in the package that holds it: `heavy cargo nextest run -p PKG --test BIN -E 'test(=NAME)'`, or `--lib`. A filter alone still builds every test binary of the package.
- Check the count: a run that selected nothing proves nothing (nextest exits 4).
- Read a failure's log (heavy prints its path) before anything reruns; no unchanged rerun, except once after exit 137.
- A failure your change can't reach runs once alone: passing, it's a flake.
- Fix a failure in the lowest tier that shows it: a skein fault seen downstream is reproduced in skein.
- Every tree's tier has a CPU budget and a wall hard limit; **starved** is a warning, never red on that account (skein/docs/foundation/testing-strategy.md, 8.1).
