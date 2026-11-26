# wires: status

2026-11-26. Where temper's code lags or departs from [wires.md](wires.md).

- **Not yet:** 8: `Items` has no `rerun` [in flight: change 18913].

- **Not yet:** 5: the engine fits a start to the room at its own layout before converting it, and fits nothing again after: a start that would fit at the worker's layout once cut is refused as invalid, and one that grows past the room is refused without the cuts that would fit it. A converted frame past the room takes `Bound::Room` and is answered `Invalid(Start)`, not *untranslated*.
- **Not yet:** 7: no host-tool adapter can answer a call itself. An adapter returns a mapped tool and input or a problem, and the engine passes every successful mapping to the current decoder.
- **Not yet:** 10.1: the root stops taking queued openings when the core is stopping, so an opening accepted but still queued at the stop can be missing from the settled checkpoint.
- **Not yet:** 10.1: a derived kinds' table can admit more than 64 terms, and the workers component hands them up whole. The openings row holds at most 64, so its write is refused at encoding.
- **Not yet:** 10.2: `temper-engine upgrade --check` uses the copy's resolved settings, but `--workers` uses the limits resolved before opening the copy. `temper-box` uses the configuration document's preflight for both, without the copy's overrides or `limits-held`. The verdict can therefore use a different room or limits than the engine would at a start.
- **Not yet:** 10.2: an unreadable wires row is reported as `the copy keeps no wires`, as if the row were absent. If the rehearsal cannot read or decode an opening, it reports an empty openings list. Every configured worker is reported as having no opening kept, including those with readable rows.
- **Not yet:** 11: the wire tests compare `layout()` with the parsed text's fingerprint and, except for `smith-pieces`, check that a field type change moves it. They do not check that a rename leaves the fingerprint unchanged.
- **Not yet:** 11: the client freeze test mutates inline schema strings, and the host-tools layout and adapter tests use in-memory fixtures. The host-tools floor has a scratch test, but there is no end-to-end landing-gate check of these mutations over a scratch checkout and its base.

