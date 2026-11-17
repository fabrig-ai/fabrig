# wire: status

2026-11-17. Where the tree's code lags or departs from [wire.md](wire.md).

- **Not yet:** 2 and 2.2: no stream watches `me` from its opening; the opening names no version (`StreamOpen` has no `version`), the hello comes alone at once, there is no `StreamEvent::Opened` or watch `0`, and the wire is at version 27 [in flight: change 6330].
- **Not yet:** 2.3: the link's `OpenStream` names no version, and the link opens a watch of `me` rather than taking it from the hello and watch `0` [in flight: change 6332].
