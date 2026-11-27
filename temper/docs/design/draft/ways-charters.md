# A project's charters

Split from [ways.md](ways.md), its section 6, on 2026-11-27, and part of that draft: where an agent's words come from, and a project's charters, held whole in `.temper/charters/`. Its status is its own, and its execution is [ways.execution.md](ways.execution.md)'s.

## 1. Five sources

| Source | Says | Read from | Changed by |
|---|---|---|---|
| the harness | tools, the brief's frame, the checkout, delivery, the budget | smith's and temper's code (smith's `domain/run-start.md`) | a release |
| the charter | how this agent works: its instructions and its parts | `.temper/charters/` of the home repository, whole, where it has `ways.toml` (2); else the stock charter's files (`stock-charters.md`, section 2), or the deployment's charter where it sets one whole | the home repository (`ways-temper-dir.md`, 6); else a release, or the deployment's owners |
| the deployment's lines | what its machines are, and its own rules | a deployment overlay of the charter (jig's `domain/engine/overlays.md`) | the deployment's owners |
| the repository's guide | how this code is written, for every agent and person | `AGENTS.md` at the root of each workspace directory, as smith finds it; others as it says (smith's `domain/run-start.md`, 1.3) | the repository |
| the task | its words, its brief and its messages | the task (`../domain/agent/charter.md`, section 1) | its requester, and the run |

- **All or nothing, for words as for ways** (the user, 2026-11-27): a repository with `ways.toml` holds every charter its ways' agents run by, whole. Nothing of the stock's text is read for it, and a charter a way names that has no file fails the landing check (`ways-temper-dir.md`, 6), never filled in from the stock. A repository without `ways.toml` runs the stock ways with the stock charters, as today.
- **What stays outside, on purpose:** the harness, which is code; the deployment's lines, which say what its machines are and differ from one deployment to the next, while the repository is the same on all; figures and authority, which are settings (`ways.md`, 2.1); and the reviewer behind the floor (`ways.md`, 5.4).
- **Sources, not an order:** smith lays out the prompt in its own order, instructions and brief before workspace guides (smith's `domain/run-start.md`). The table says where each word comes from.
- **Each source says what only it can say.** A rule said in two of them is a fault, which the preview shows (4).
- **A run with no workspace** has no guide.

## 2. The project's charters

- **A file per charter, named for it** (`ways.md`, section 9): `.temper/charters/implementer.md`, `.temper/charters/reviewer.md`, `.temper/charters/lead.md`. A file holds the charter's whole text: its instructions, then its parts, each from a line `## part, on: REASONS` to the next, selected by its reasons as a stock part is (`stock-charters.md`, section 4).
- **Composed in place,** where a way's task names several files (`ways-recipes.md`, 2): their instructions joined in order, then their parts. A file shared by several charters, as the stock's common file is, is one file named by each.
- **Words alone.** A file can't set models, tools, wakes or budgets, which stay settings (`ways.md`, 2.1).
- **A front matter**, between `+++` lines, in TOML:
  - `settings = "implementer"`: the deployment's charter whose settings it runs by, its tools, hosts, waiting, resume and default budget. Left out, the deployment's charter of the file's own name; a file with neither fails the landing check.
  - `format`: the file's format (`ways-temper-dir.md`, 8).
- **Written from the stock at the start:** picking ways at a project's creation (`ways-temper-dir.md`, 7), or `temper ctl charter copy NAME` (`ways-temper-dir.md`, 2), writes the stock's files whole. After that they are the project's, and a release's changes to the stock's text reach them only through Ask temper's review (`ways.md`, section 11).

## 3. Carried as the project's charters

- **jig's charters gain a project scope:** a charter of the project's is its files' text at the project's repository revision (`ways-temper-dir.md`, 5), run by its `settings` charter's settings, with the deployment's overlay laid over it as over a stock charter.
- **A charter's identity gains the project:** the project, the charter's name, or for one composed in place its recipe and files, the repository revision, and the settings revision of its `settings` charter. Two projects' charters of one name are then two keys, and a record keeps them all.
- **Revisions as the work's are** (`ways-recipes.md`, 2): a piece of work reads its charters at the repository revision it started with, in every task it starts, until an amendment moves it. A running task's words don't change.
- **The worker gets assembled text,** as today, and needs nothing new to run a project's words. The engine and jig resolve the scope.
- **Bounded at the read:** a project's charter counts as any charter does (jig's `domain/engine/settings.md`, section 8). A file that would take a charter past a bound fails the landing check (`ways-temper-dir.md`, 6), naming the file and its bytes. A release, a settings change or a limit's change that puts a charter over a bound refuses new tasks of that charter, naming it, until it fits.

## 4. Seen and recorded

- **A preview:** `temper ctl settings show charter NAME --project PROJECT [--reason REASON]` prints a charter's assembled instructions, each source headed by where it came from and its size. A page draws the same in the next pass.
- **Each run records what it read:** the release, the settings revision, the project's repository revision, and the guide's commit, path and content hash, so that what a run was told can be read back from the repository while the run's records are kept.
