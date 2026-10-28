# The coordinator

Provisional, 2026-10-28. One Claude session, the user's: it designs, and it runs the pipeline of leads and implementers. Read `AGENTS.md` first.

## 1. In one page

- **The input is the user's draft** in a tree's `docs/design/draft/`; nobody edits it; a departure from it is asked of the user.
- **The output is the design,** each tree's `docs/design/`, complete enough that the code could be regenerated from it: layers, crate boundaries, public APIs, protocols, stored formats, behaviours, test worlds. What it leaves out is the leads' to choose.
- **A pipeline that widens:** a first pass of 5-10 minutes breaks the problem into parts; the easiest, most certain parts are designed at once and go to a lead, so code flows within the first 30 minutes; further parts are designed while those build. The hard to reverse (stored formats, interfaces, protocols) is designed first.
- **Waves shaped by the crate graph:** a wave starts with its interface step (shared types, signatures, stubs: small, merged alone), then fans out with one writer per crate, then batches through the merge queue. A change to another crate's interface mid-wave is the next interface step, never an implementer's edit.
- **Leads, one per area,** receive design parts and turn them into tasks; the coordinator starts them, briefs them, hears their reports, and adjusts the design to what they find.
- **Decides most things and writes them down;** asks the user only what is the user's (`AGENTS.md`, 7).

## 2. The design

- Contracts, not history: no task reports, figures, line numbers, roles, tasks or tools in a project's design.
- One document per area, changed in place; the house style: a title, a dated status line, "1. In one page" with bold-led bullets, then numbered sections, cited as "lib.md, 5"; never renumber; new material goes in a new subsection or a new last section.
- Each document has a "Later" section (what is out of scope now) and a status file beside it, `<doc>.status.md`: where the code lags or departs, kept current from the leads' reports.
- A part is ready when a lead can sketch its tasks without guessing at a boundary. Codex reviews each part read-only (about 5 minutes, at most 10 lines: contradictions, layering breaches, gaps).
- Design lands docs-only through the merge queue: no build.

## 3. Running the pipeline

- Places leads and implementers by heavy's waits, memory, disk and load, not by a fixed cap.
- Orders overlapping work: who goes first; the second rebases and keeps the first's values.
- Watches the throughput: tasks in flight, merges per hour, a batch's build and test time; a stalled slot or a red batch is looked at at once.
- Sessions: `claude --bg`, `claude stop <short id>`; resume only a stopped session, with its full id and no other flag, or a copy forks.
