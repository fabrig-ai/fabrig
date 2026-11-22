# README: status

2026-11-20. Where temper's code lags or departs from [README.md](README.md).


- **Not yet:** 2: there is no `temper-client-web-page`: the shell is the bundle's `cdylib`.





- **Departs:** 2, *The crate graph*: `temper-client-domain-forge` also depends on
  `temper-protocol-records`, because the forge's side encodes and decodes a
  pasted bundle by it (`forge/forges.rs`), and on `skein-ui` for its phrasebooks
  (`words.rs`), and `temper-client-domain-pages` on `skein-json` to read a call's
  input (`pages/calls.rs`); the graph names none of them. Through them the
  deployment's client links the records crate (`../protocol/binaries.status.md`).

## Gaps and departures

- **Not yet:** `ui-ask-m1`'s `proposal-read` part was never designed,
  according to the retirement brief. Its landed run proposal read needs
  design coverage reconciliation against `jig/docs/design/domain/engine/executors.md`,
  3; this is a design gap, not an absent implementation.


T1's repair, goal 761, carries the launchpad's brief, Where you left off
and Needs you across the person's projects, a walked need's reason,
need ages, dated left-off rows and the blank row under the header
(`tui/pages/launchpad.md`). Its walk follows landing (`README.md`, 5);
Stop asking stays on the web (`../draft/surfaces.status.md`, 11).
