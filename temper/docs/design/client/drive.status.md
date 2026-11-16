# drive: status

2026-11-16. Where temper's code lags or departs from [drive.md](drive.md).

Unlisted sections have no known gap.

- **Not yet:** 2.1 and 3.8: deployment sessions and deployment patch lines are not built; the session starts a box alone.
- **Not yet:** 2.6: recipe walks do not select ctl sessions for chains without web or tui lines; the walker injects web and tui reset lines.
- **Not yet:** 3.4: `{model.key}` is not parsed or resolved from the box log.
- **Not yet:** 2.2, 2.3's *Its links*, 2.5 and 6's *No link logged*: `session/box.log` holds the box's stdout too, its sign-in links among them [in flight: change 5252].
- **Not yet:** 2.3's step 2 and 6's *ctl from a file*: ctl signs in with the box's first link on its stdin (`sign-in -`); no `session/link` is written.
