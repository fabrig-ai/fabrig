# configuration: status

2026-11-11. Where jig's code lags or departs from [configuration.md](configuration.md).

- **Not yet:** 3: a provider's `direct` endpoint, its checks, and a list read on it [in flight: change 927].
- **Not yet:** 4: the `programs` row's credential source on a provider's `direct` endpoint (`../domain/accounts.md`, 3.4) [in flight: change 928].

- **Not yet:** 3 and 3.2: a template's `ended_by` `standing` and a model's `cache_life_ms` (`../domain/tasks/standing.md`, section 7).
- **Not yet:** 3: parts under a resuming charter (`../domain/tasks/standing.md`, 5.4).
- **Not yet:** 3: a profile's `brief_core_budgets.now` (`../domain/tasks/standing.md`, section 6).

`configuration.md`, sections 3 and 3.1 and `limits.md` have no known gap in ordered charter parts and
their instruction files, path-specific refusals, the
`tasks.charter_parts` limit of 8 in every profile, or a template's
`later`. Parts count with the stored policy against `core.run_bytes`;
later choices are refused under a resuming charter.

Section 3 has no known gap in passed model defaults, refusing document
model tables under those defaults, provider endpoints or charter `rest`.
Section 4's agent configuration declares endpoint limits, without models.
Section 3's provider-list keep and drop patterns and their bounds have no
known implementation gap.

Section 3's list configuration is optional: no keep means no read, and
keep requires an endpoint. Invalid patterns are refused at their path.

`limits.md` has no known gap in standard's `authority.batch` of 32,
`hub.charter_bytes` of 262,144 and `hub.outcome_bytes` of 131,072.
Small keeps its charter and outcome figures. The larger batch sizes
aggregate bounds without reserving their capacity ahead.

`limits.md` has no known gap in either profile's returning redirect
bound of 16,384 or listener head of 24,593. Small's full unescaped
answer needs 13,407 bytes; `driver_fits` and the whole configuration
check refuse an undersized redirect bound or listener head, while the
outgoing URL stays within `ADDRESS`, 2,048. Focused rows cover each
driver-member refusal and both supplied profiles. The long-answer
failure reaches the accounts domain as `SignInAgain`; carrying its
figures to the application's page is deferred in `configuration.md`, section 8.

- **Departs:** 5: the reference service's first periods come from its engine's routing, which hands the core each `Period` (`jig/testing/jig-test-service/src/engine.rs`), not from a child of its root, because jig's test root composes no period child.

Section 3's optional clients' `approval` boolean is present, defaults to
false when absent, and reaches the clients' component's settings.
- **Not yet:** 3: retention's horizons in the deployment member: none is parsed. 3, 4, 6: the engine's own slots for agents in the engine, and TLS for the workers' listener and the worker's `engine` address: `jig/crates/jig-protocol-config/src/hosts.rs` and `address.rs` read plaintext and in-process addresses alone. 5: `ops`'s calendar child (`../examples.md`, section 4).  the people-envelope diagnostic gap remains `limits.status.md` (Not yet, the whole document).
