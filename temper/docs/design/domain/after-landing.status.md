# after-landing: status

2026-11-11. Where temper's code lags or departs from [after-landing.md](after-landing.md).

Section 10's public queue merges read the log's commits and tasks alone;
there is no known gap in that projection. Older public history is deferred
in section 14. The log's other gaps and the retained change rows remain below.

- **Not yet:** 7, *A goal that has ended*: a red of no goal's is not filed as a finding of its project, once per landing, so no triager hears it [in flight: change 751].
- **Not yet:** 7, *A goal that has ended*: a red of a landing that brought a follow change filed with its follow's owner's area [in flight: change 2006].
- **Not yet:** 3, by plan `forge-polls-quiet`: a tier polls its run in
  flight every `poll`, not at the client's `poll_at`.
- **Not yet:** 3: the pending verdict status a run posts on its tip
  before it is held (section 9's). 5.2, 11: the suspects' rule and a
  red's pause, kind 4's alone (`landing.md`, 13.1, and its status):
  every red range is probed (`temper-engine-domain/src/translate.rs`),
  and no red pauses its repository. 6: a cleared verdict is news to no
  goal, only a red one (section 7); and a verdict after the bound, or
  after a verifying change is cancelled, reaches no one: the
  `Verification` row goes with the landing's last change, and no
  landing log or red of no goal's keeps the landing's verdict. 8: no trailers in landing news; no `landing list
  --newer-than`. 9: no `temper/verdict/<tier>` status or comment;
  purpose tag 8's operations 6 and 7 are unused. 10: no `trailers` or `trailer_bytes` limit;
  a push from outside temper, trailers and verdicts stay out of the log
  until sections 6, 8 and 9 need them.
  12: the stories of a documentation landing cleared at once by its
  receipt and of the late red kept for findings; by turn, only a change
  waiting for main's tier and its cleared result naming its run, the
  rest running under queue alone
  (`tests/engine/domain/tests/suite/forge_routes/`).
- **Departs:** 10: a tier task holds the project's whole ceiling, not
  kinds 6, 7, 9, 16 and 17 alone, as a landing does (`landing.md`,
  section 11), because jig gives a procedure the deployment starts its
  project's ceiling (`jig-domain-core/src/effects/connector.rs`,
  `procedure_seed`).
- **Departs:** 10: a change's close does not take its row, as this section
  and the request of 2026-11-05 say it does: no step erases a `ChangeRow`,
  because the answer of 2026-11-05 kept it for a closed change's panel. The log
  holds its titles without it: `a_landed_change_keeps_its_title_after_its_close`
  reads them after a restoration of the log and its repository alone.
- **Departs:** 5.2: probes visit changed-key landings linearly, oldest
  first, rather than bisecting the range. The first candidate whose commit
  fails a test owns it; the last candidate owns what remains without a
  probe. Unchanged-key landings are skipped. No reason for choosing this
  traversal over bisection is recorded.
