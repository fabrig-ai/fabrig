# retired-plans-out: standing rules

Every task of this plan names this note. The request is
`temper/docs/plan/retired-plans-out/inputs/request.md`; its Bounds hold: documentation changes
only, and nothing under a live plan's directory changes.

## Read first

- `AGENTS.md`, sections 1, 2 and 7, and `DESIGN.md` whole (44 lines): the three layers, what a
  status file holds (dated, gaps only, never history; **Not yet** and **Departs** for a design
  document; **Design lags**, **Design departs** and **Open** for a draft), and that a document
  closes with its "Later" or "Open questions".
- `temper/AGENTS.md`, and the `AGENTS.md` of any other tree whose files your task changes.

## The 49 directories

The retired plans, in the request's order. "The 49" in every spec means this list.

```
jig/docs/plan/wasm32-jig
skein/docs/plan/playground-skein
skein/docs/plan/wasm32-skein
smith/docs/plan/live-smith-manual
smith/docs/plan/live-smith-reds
smith/docs/plan/live-smith-reds-2
smith/docs/plan/reference-m4
temper/docs/plan/citation-burndown
temper/docs/plan/collector-starved
temper/docs/plan/configuring-p1
temper/docs/plan/configuring-p2
temper/docs/plan/configuring-p2-build
temper/docs/plan/configuring-p3
temper/docs/plan/fab3-fixes
temper/docs/plan/first-run-projects
temper/docs/plan/forgejo-connect
temper/docs/plan/github-repair-race
temper/docs/plan/interrupted-calls
temper/docs/plan/live-temper
temper/docs/plan/live-temper-stories
temper/docs/plan/live-temper-stories-2
temper/docs/plan/look-welcome-chats
temper/docs/plan/me-retention
temper/docs/plan/open-default
temper/docs/plan/openai-manual
temper/docs/plan/phase-case
temper/docs/plan/playground-decisions
temper/docs/plan/playground-temper
temper/docs/plan/public-reading
temper/docs/plan/readiness-eintr
temper/docs/plan/reference-m1
temper/docs/plan/reference-m2
temper/docs/plan/reference-m3
temper/docs/plan/reference-m5
temper/docs/plan/reference-m6
temper/docs/plan/reference-m7a
temper/docs/plan/reference-m7b
temper/docs/plan/reference-m8
temper/docs/plan/reference-m9
temper/docs/plan/reference-m10
temper/docs/plan/repo-links
temper/docs/plan/status-gaps
temper/docs/plan/status-lines
temper/docs/plan/wasm32-list
temper/docs/plan/wasm32-more
temper/docs/plan/wasm32-rustls
temper/docs/plan/wasm32-temper
temper/docs/plan/wasm32-temper-2
temper/docs/plan/wasm32-temper-3
```

Every other `docs/plan/` directory is live and never changes in this plan: reference-walks,
v0.1.0-ladder, sign-in-wait, live-smith-reds-3, outward-rule, public-reading-build,
withdrawn-forms, this plan's own, and any plan started since. The only plan files this plan's
tasks write are the kept notes of `notes/contracts.md`, by the audit tasks, and the deletion of
the 49, by `plans-out`.

## What text added outside `docs/plan/` may say

The request: each fact is written "dated, in that document's words, with no reference to the
plan or to who ran it". So a line this plan adds or rewrites anywhere outside `docs/plan/`:

- names no plan (none of the 49, nor any other), no `docs/plan/` path, no task, spec, answer,
  cut, lead, designer, implementer, foreman (of the lab's runs), director, slot or run of the
  lab that carried the work out. "the user" may stand as who decided, where the source says the
  user decided (`DESIGN.md`, 2, a draft's **Design departs**); otherwise a decision is dated
  alone ("decided 2026-11-03").
- names nothing of the lab: none of the names `tests/boundary/tests/outward.rs`'s module doc
  lists (`fab`, `heavy cargo`, the agents repository, `fabrig/qa`, `agents/loop`, `hel8`, the
  home paths and the rest). The plans are full of them; a fact copied out of a plan is
  rewritten without them. The outward check fails a file whose count rises.
- gives a date as `YYYY-MM-DD` and a commit as a short hash of main's history
  (`git log --first-parent origin/main`).

## Where the design may change, and how

- **Allowed targets:** a tree's design documents and their status files
  (`<tree>/docs/design/**`), the drafts' status files (`draft/<name>.status.md`) and execution
  files (`draft/<name>.execution.md`), and `tools/tiers/design.md` with its status. Only the
  tasks whose specs say so write them.
- **Never changed:** a draft's own text (`draft/<name>.md`, `draft/archive/`), `draft/notes.md`,
  `docs/help/`, `AGENTS.md`, `DESIGN.md`, `skein/docs/foundation/`, any code (one comment line
  in `citations-out` is the exception its spec states).
- **Departure from the implementer's text,** which says `docs/design/` is not yours:
  `citations-out` and `keep-decisions` write the design, as their specs say, because the
  request is a docs task whose whole work is there.
- **A status file** that gains or changes an entry takes today's date on its date line
  (`date -u +%F`), as `DESIGN.md`, 2 has status files dated. A new entry is a bullet of its own
  under the right label, after that label's last bullet; an existing bullet changes only where
  your spec says.
- **Files other drafts are changing now** (their trunks are not yet on main): touch only the
  lines your work needs; an addition is a line or bullet of its own, never a rewrite of an
  existing line, so that the merges meet the fewest conflicts. They are, outside `docs/plan/`:
  - temper: `client/README.md`, `client/README.status.md`, `client/browser.status.md`,
    `domain/agent.md`, `domain/agent.status.md`, `domain/forge.md`, `domain/forge.status.md`,
    `domain/testing.md`, `protocol/configuration.md`, `protocol/configuration.status.md`,
    `protocol/setup.md`, `protocol/setup.status.md`, `protocol/README.status.md`,
    `playground.status.md`, `STATUS.md`, `README.md`, `draft/dashboards.md`,
    `draft/reference-case.execution.md` (section 2 only), all under `temper/docs/design/`;
  - jig: `domain/tasks.status.md`, `client/domain/objects.status.md`,
    `client/domain/pages.status.md`, `protocol/configuration.status.md`, `protocol/store.md`,
    `protocol/store.status.md`, under `jig/docs/design/`;
  - smith: `protocol/transcript.md`, `domain/session.status.md`, `STATUS.md`, under
    `smith/docs/design/`; skein: `docs/design/examples.status.md`;
  - `tools/tiers/design.status.md`, `tests/boundary/outward.txt`, `AGENTS.md`, `DESIGN.md`.

## Checks

A docs branch runs these in place of the implementer's text's fmt, clippy, old-forms and
package tests (a departure: the branch changes no code, but for `citations-out`'s comment line,
whose spec says what it runs). From the repository root, after committing:

- `python3 -I -B tools/citations/citations.py --all` prints `citations: 0 broken in HEAD` (it
  was 0 at this plan's cut, `ae878830b8`). It reads committed trees: commit first. A broken
  citation your branch did not add goes under `## Left`.
- A branch that changes any file outside `docs/plan/`:
  `heavy cargo nextest run -p fabrig-boundary --test outward` passes (10 tests at the cut; a run that
  selects none proves nothing). Leave `tests/boundary/outward.txt` as it is: a count that falls
  passes, and other drafts edit that file.
- A changed file that `.config/gate/read-by-tests.txt` lists runs its readers:
  `smith/docs/design/**`: `heavy cargo nextest run -p smith-bench`;
  `jig/docs/design/protocol/store.md`: `heavy cargo nextest run -p jig-protocol-store --lib`;
  `skein/docs/design/forge-github.md`: `heavy cargo nextest run -p github-conformance`.
  `temper/docs/help/**` never changes here.
- Every build and test through `heavy`, one command at a time (`AGENTS.md`, 5 and 6).

Commits are small and say why. Push your branch.
