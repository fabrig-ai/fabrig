# box-link-first: status

2026-11-06. Where the design lags or departs from [box-link-first.md](box-link-first.md).

- **Design lags:**
  - 2, 5: `../client/greenfield/app.js`, the first run's terminal in the mock that is the client's target (`../client/greenfield/README.md`): it shows `First run`, `Logs:`, `Starting serve`, the block in its old words and `Ctrl-C stops temper.`, and says the box then writes a line when something starts, ends or fails. The plan's cut left the mock alone, the client's own work having priority over its files.
- **Design departs:**
  - 2: `../protocol/setup.md`, 7.2: a fresh link's blank line is on stderr, the link on stdout, where this draft's words can read as both on stdout: its own "the link alone on stdout, the other lines on stderr" and `temper help commands`' "its stdout" want a script reading stdout to read links alone. A terminal shows the same. Decided by the plan's cut.
  - 2, 3, 4: `../protocol/setup.md`, 7.1 and 7.2: a clients' listener at port 0 is not held, as 7.1 has it, so the box has no port to print: serve draws its own first secret and the block is written at its first link. The default document's port is fixed (8.2), so a person's box never meets this. Decided by the plan's cut, to keep 7.1's rule and leave such a document working.
- **Open:**
  - 6: a progress page; the time of day on error lines (`../protocol/setup.md`, 16).
