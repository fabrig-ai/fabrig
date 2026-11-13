# run-provider-outage: status

2026-11-13. Where the design lags or departs from [run-provider-outage.md](run-provider-outage.md).

Section 2 is designed. Item 1 in smith's `protocol/programs.md`, 5.6, for
the program backend, and skein's `llm/README.md`, 2.5, 4.7 and 4.8, for
the direct ones. Item 2 in jig's `domain/hosts/hub.md`, 3 (an outage) and
`domain/tasks/lifecycle.md`, 6, with a review's run in
`../domain/forge/change/gates.md`, 1: an outage there is retried, not a
failed review. Item 3 in jig's `domain/hosts/hub.md`, 3 (the
`ModelFailure` and its words), `protocol/channel/records.md`, 1,
`../domain/worker/README.md`, 4.3 and 10, and smith's `domain/run.md`, 9
and `protocol/channel.md`, 4. Section 5: no store changes, since a
failure's reason and a hold's words are kept already (jig's
`domain/tasks/lifecycle.md`, 6); jig's worker channel and smith's
channel move their versions with their layouts (`../protocol/wires.md`, 3).

As item 2 says, an outage is a failure unavailable or overloaded whose
evidence is unsent. A direct call that an edge or the provider answered
(a 5xx, or a 403 that is not the provider's own) has a response as its
evidence, so it stays the run's own failure, its hold naming it all the
same (jig's `domain/hosts/hub.md`, 3).

- **Design lags:** none.
- **Design departs:**
  - 2, item 1: `result_failure` gives smith's `llm::Failure`
    (`smith-domain-session`'s), not skein's `Failure`; its `Unavailable`
    and `Overloaded` are skein's by name. Because the program backend
    speaks smith's session vocabulary (the designer, 2026-11-13).
  - 2, item 3: a program's HTTP status rides in the failure's report
    (`Report::status`) beside its evidence, unsent, where smith's
    `protocol/programs.md`, 5.6 had it as the evidence. Because the
    draft's own words keep both, `status 403, unsent`, and item 2 keys
    the outage on the evidence being unsent (the designer, 2026-11-13).
- **Open:** none.
