# testing: status

2026-11-23. Where skein's code lags or departs from [README.md](README.md).

`layout.md`, section 1's test-only fork kit and
its documented process boundary are present, as is the row runner's
admission.
`README.md`, section 9 carries deferred cost work, including further folds,
allocation layout and the native fork floor.

`layout.md`, section 1's full-screen renderer and terminal crates, golden frames and
simulated-terminal world are present.

`service-tiers.md`, section 1's helper processes use `Starved::exit`, and real-loop cleanup
precedes the verdict. The deadline and real-loop tests cover the
one-process mode and children remove both runner variables when checking
failure. `README.md`, section 9 leaves caller adoption and suite CPU measurement separate.
A span counts its thread's allocations (`Span::made`, held by
`skein/tests/heap`'s `a_span_counts_the_allocations_its_thread_made`),
and skein-codec's layout step tests show a shared workspace reused with
none counted around each reused pass, never by an address.

- **Not yet:** 4: the probe's report of a contained tree.
  4: the probe for a reach, `skein_fake_machine::probe`, and the machine honouring a reach.
  [in flight: change 9030]
- **Not yet:** 4: the fake machine's metadata beyond mode `0o777`, times
  and attributes, its subvolume and filesystem answers, and tree programs.
- **Not yet:** 7: the focused cost-saving and suite CPU-margin proofs;
  selected instruction ratios establish neither a whole-suite saving nor its margin.
- **Not yet:** 7.1: `skein-codec-tests::suite`'s sweeps take no case from
  the seam [in flight: change 20862].
- **Not yet:** 7.1: `skein-lib-tests::suite`'s sweeps take no case from the
  seam [in flight: change 20863].
- **Not yet:** 7.1: skein-deflate's own sweeps take no case from the seam
  [in flight: change 20864].
- **Not yet:** 7.1: `skein-sim-tests::suite`'s sweeps take no case from the
  seam [in flight: change 20865].
- **Not yet:** 7.1: `skein-conformance-sim::suite`'s sweeps take no case
  from the seam, and its cancel, race and process tests each run a counted
  loop a configuration [in flight: change 20866].
- **Not yet:** 7.1: `skein-fake-checkout-tests::suite`'s sweep takes no case
  from the seam [in flight: change 20880].
- **Not yet:** 7.1: `skein-io-world::suite`'s sweeps take no case from the
  seam, and its appends and stores stop on coverage [in flight: change 20881].
- **Not yet:** 7.1: `skein-kv-tests::simulated`'s sweeps take no case from
  the seam [in flight: change 20882].
- **Not yet:** 7.1: `skein-echo-world::simulated`'s sweeps take no case from
  the seam [in flight: change 20883].
- **Not yet:** 7.1: `skein-fake-peers-tests::simulated`'s sweep takes no case
  from the seam [in flight: change 20884].
- **Not yet:** 7.1: `skein-forge-client-world::suite`'s sweeps take no case
  from the seam [in flight: change 20893].
- **Not yet:** 7.1: `skein-llm-connection-world::suite`'s sweeps take no
  case from the seam [in flight: change 20894].
- **Not yet:** 7.1: `skein-world-tests::simulated`'s sweeps take no case from
  the seam [in flight: change 20895].
- **Not yet:** 7.1: `skein-oauth-accounts-world::simulated`'s sweeps take no
  case from the seam [in flight: change 20896].

- **Departs:** 10: `skein/testing/skein-world/src/lib.rs` fences six
  items `cfg(unix)`, though these sections have the crate native, on
  Linux and macOS, so off unix it fails to build however it is fenced; no
  reason is recorded.
