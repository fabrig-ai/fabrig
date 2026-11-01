# Request: `--until phase:<Phase>` in the spelling `task show` prints

From fabrig/qa via the director, 2026-11-01: a minor, off-path QA finding
(coord/qa-ladder findings.md 42, walk ql-b1-5), used as fab's first probe of
a fabrig integration plan from intake to main. The subject is the path, not
the change: keep the plan to one direct part of one task.

## The finding

`temper ctl DIR task show TASK --until phase:Running` prints the generic
usage line. Only the lower-case spellings parse (`phase:running`,
`phase:due`), but `task show` prints the phase capitalised (`Running`,
`Due`), and no help names the accepted spelling.

Reproduction, on any box: `task show 1 --until phase:Running` gives the
usage line, while `--until phase:running` is met.

## The fix

- One area: `temper/crates/temper-client-tools/src/jig/subject.rs`,
  `phase()` (~line 75): accept either case (an ASCII case-insensitive
  match of the words it lists, so `Running`, `running` and `RUNNING` all
  parse; `BackingOff`/`backing-off` as `task show` prints it).
- Say it in `temper ctl help task show` (the help text under
  `temper/docs/help`, with its help tests).
- A regression test in temper-client-tools' unit tests,
  `temper/crates/temper-client-tools/src/tests/subject.rs` (lib), beside
  the existing phase parse tests: the printed spelling parses to the same
  phase as the lower-case one.
