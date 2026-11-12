# github: status

2026-11-11. Where the tree's code lags or departs from [github.md](github.md).

- **Not yet:** 1.1: an issue's comments ignore `since`, on the Forgejo
  layer (`skein-forge/src/forgejo/server/items.rs`) and the GitHub layer.
 
- **Not yet:**  the refused-copy repository and delete-scope gaps remain `forge.status.md`, 4.  The path-read refusal gap remains `forge.status.md`, 4.  The private-source fact remains `forge.status.md`, 4. 1.2: the GitHub layer refuses no push as `GH013`: a push reaches the domain, whose `Rejected` is its only refusal. So 11's required-signatures fact stays unchecked on the fake.
- **Departs:**  the shared-type departure remains `forge.status.md`, 4.  The captured-request queue departure remains `forge.status.md`, 4. 1.6: the user's request of 2026-11-04 has the fixture's App, accounts and repositories built once, in the fake; that holds in skein alone. Four builders of temper's stand beside it, each starting `Peer::github` from its own `Config` (`temper/crates/temper-fake/src/forge/github.rs`, `temper/tests/client-browser/src/github.rs`, `temper/tests/one-process/src/lower/github.rs`, `temper/tests/real-loop/src/github.rs`), because the item that request paid is skein's and they write no HTTP of their own. 3: `skein/docs/foundation/programming-model.md`, 5.4 asks that a state hold exactly its data, with no lifecycle in `Option`s beside it, but `skein-fake-forge-http/src/connection.rs`'s `Connection` keeps the captured head, the reply in hand and its offset beside a data-less `State`, as `github.md`, section 3 has it, because the core took the shape both servers had, so that no behaviour moved; no debt item names it.
- **Not yet:** 1.7: the GitHub layer's service gives no image of its state [in flight: change 1367].
