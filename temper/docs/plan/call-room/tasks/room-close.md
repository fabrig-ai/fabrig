+++
id = "room-close"
part = "close"
kind = "docs"
needs = ["large-argument", "smith-room", "jig-room", "reference-room", "report-through"]
files = ["temper/docs/design/protocol/configuration.status.md", "temper/docs/design/domain/agent.status.md", "temper/docs/design/domain/forge.status.md", "jig/docs/design/domain/hosts.status.md", "jig/docs/design/protocol/configuration.status.md", "smith/docs/design/protocol/limits.status.md"]
effort = "medium"
minutes = 30
+++

# The status files say what this plan left

## Goal

The cut wrote this plan's design and marked it **Not yet** in each status file. Everything it built leaves them; what the tasks measured replaces what the cut could only estimate.

**Departures from the implementer's text:** this task edits `docs/design/`, as listed below. It runs no build beyond what the docs need: no test of code, none of it changed.

Acceptance:

- The **Not yet** entries the cut wrote are gone from `temper/docs/design/protocol/configuration.status.md` (13.8, 13.9), `temper/docs/design/domain/agent.status.md` (4.4), `temper/docs/design/domain/forge.status.md` (8.1), `jig/docs/design/domain/hosts.status.md` (8), `jig/docs/design/protocol/configuration.status.md` (6) and `smith/docs/design/protocol/limits.status.md` (3.10, 2.4), each only for what its task's report and the trunk show as built. What is still not built stays, saying so. The entry on charter 5's words stays: this plan doesn't change them.
- `python3 -I -B tools/citations/citations.py --base origin/main` lists nothing new.

## Inputs

- `temper/docs/plan/call-room/notes/common.md`; `temper/docs/plan/call-room/notes/contracts.md`.
- `DESIGN.md`, sections 2 and 4.
- The trunk's `git log` for this plan's tasks; `rg` for the figures below.

## Owns and may change

Owns the status files in the header. May also change, each only as said:

- `temper/docs/design/protocol/configuration.md`, 13.8 and 13.9: if `room.json` holds 63,740 and 48,000, say those are the reference's figures, and that the larger was refused and by what; if it holds 131,072 and 98,500, drop the sentence about the fallback. The figures in the table of 13.9 follow. If `reference-room` moved a limit beyond 13.8's table, add its row.
- `temper/docs/design/domain/agent.md`, 4.3: `it leaves at least 361 bytes` takes what `every_stock_charter_admits_its_whole_maximal_brief` holds now, if that moved. `temper/docs/design/protocol/configuration.md`, 13.4 and section 11's `355,965` and `configuration.status.md`'s, if `root.brief.brief_bytes` moved.
- `smith/docs/design/protocol/limits.md`, 3.10's last paragraph, if `smith-room` measured otherwise.
- `jig/docs/design/protocol/configuration.md`, 6, if `jig-room` kept `authority.batch` out of `standard`, and `temper/docs/design/protocol/configuration.md`, 13.8's *A batch* with it, saying what refused it.

Options, each pre-approved:

- **`smith/docs/design/**` is read by tests** (`.config/gate/read-by-tests.txt`): keep its section numbers, and if you edited `limits.md`, run `heavy cargo nextest run -p smith-bench`.
- **A figure you can't find a measurement for** stays as the design has it, and goes under `## Left` in the report.

## Steps

Read each task's merged commits by `git log --stat` and the hunks in question; the figures by `rg -n` in the tests that pin them. Edit each status line. `DESIGN.md`, 2: a status records only the gaps as they stand now, never history. Run the citation check. Commit, push.

## Report

Each entry removed or kept, with why in a line. The figures you changed, old and new.
