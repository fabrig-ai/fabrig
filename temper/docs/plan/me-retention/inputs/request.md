# Request: `me` prints its retention as a readable duration

From fabrig/qa via agents/loop, 2026-11-02: a minor, off-path QA finding (coord/qa-ladder
findings.md 3, fab's `v0.1.0-ladder/finding-13`, triaged to the backlog), used as fab's second
probe of a fabrig integration plan from intake to main. The subject is the path, not the
change: keep the plan to one direct part of one task.

## The finding

`temper ctl DIR me` prints `retention: 86400000000000 ns`. Expected: a readable duration,
`1 day`, or the form ctl uses elsewhere for durations.

## The fix

- One area: `temper/crates/temper-client-tools/src/jig/render/document.rs` (~193-195,
  `out.number(me.retention.as_nanos()); out.put(b" ns\n")`): render the retention in the
  duration form ctl already uses elsewhere (find it; reuse it rather than writing a new one).
- Check whether a golden test expects ` ns` and whether help (`temper/docs/help`, with its help
  tests) names the form; update them to match.
- A regression test in temper-client-tools' unit tests (the `me` render or golden tests,
  `src/tests/printed.rs` or `src/tests/golden`): a day's retention prints in the readable form.
