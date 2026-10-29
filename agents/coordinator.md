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
- A part is ready when a lead can sketch its tasks without guessing at a boundary. Codex reviews each part read-only (about 5 minutes, at most 10 lines: contradictions, layering breaches, gaps). For what is hard to reverse (stored formats, public interfaces, protocols) the review comes before the part goes to a lead; for the rest it runs beside the lead's first tasks, and its findings amend the part.
- **A consistency pass** every few parts: a read-only Codex run over the documents changed since the last pass, for contradictions between documents and drift between a document and its status file.
- Design lands docs-only through the merge queue: no build.

## 3. Running the pipeline

- Places leads and implementers by heavy's waits, memory, disk and load, not by a fixed cap.
- Orders overlapping work: who goes first; the second rebases and keeps the first's values.
- Watches the throughput: tasks in flight, merges per hour, a batch's build and test time; a stalled slot or a red batch is looked at at once.
- Sessions: `claude --bg`, `claude stop <short id>`; resume only a stopped session, with its full id and no other flag, or a copy forks. A session starts from a directory Claude trusts (`~/src/rust/...`), its scratch elsewhere.
- **No branch crosses trees.** An API another tree uses changes in three steps, each its own branch: skein (or the upstream tree) adds the new form beside the old, each downstream crate moves, the old form goes. A sum read downstream goes reader first. So every crate stays one writer's, and no part waits on a branch that spans four trees.
- **Slots:** shares live in `tools/slots/hosts`; the rest is a free pool that a lead claims and releases itself with installed `claim` from PATH on hel8. Another lead's share requires its owner's agreed lend (`--lent-by SHARE_OWNER`). The coordinator moves shares when a tree runs dry by changing `hosts` through the queue.
- **The queue restarts only through `tools/queue/restart`,** run in a login shell on the gate machine (`bash -lc`): it checks PATH, waits for the batch, and starts `mq` unbuffered.
- **One fix, one keeper:** when two agents have made the same fix, the coordinator names the branch that stays before either is withdrawn; a crossed withdrawal leaves neither in the queue.
- **A task binds what it can check:** for tool configuration (a filter, a profile, a script's flags) the acceptance check binds (the selection unchanged, a count, an exit), not the syntax, which the tool's version decides.
- **The remedies ledger:** every stall, spurious red or extra round trip that a tool, a rule or a test approach could have prevented is written down with its cost, and its remedy goes where the next agent reads it (a role file, a tool, a test rule) within the hour, so it does not happen twice.
