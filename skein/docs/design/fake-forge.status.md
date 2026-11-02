# fake-forge: status

2026-11-02. Where skein's code lags or departs from [fake-forge.md](fake-forge.md).

- **Not yet:** 2.3: a run's report: a workflow's failing and flaky test ids, `Cue::Tests`, the artifact each completed run uploads, the selection input, and `Read::Artifact` and `Read::Commits` (`skein-fake-forge-domain/src/api.rs`: `Workflow`, `Cue` and `Read` hold none of them). 10.2: the GitHub layer refuses no push as `GH013`: a push reaches the domain, whose `Rejected` is its only refusal. So 11's required-signatures fact stays unchecked on the fake.
