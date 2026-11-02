# README: status

2026-11-02. Where temper's code lags or departs from [README.md](README.md).

- **Not yet:** 3.4, 5.5: the composer's supplied default, jig's
  `resolved_default`, which neither jig's `Me` nor temper carries. 3.5:
  `Board { project }` with its reordering, and `Changes { project }`: no
  address in `temper-client-domain`'s `address.rs`. 3.5, 5.13, 10:
  `HowWorkRuns`, `Proposal { proposal }` and a proposal's card in a chat,
  with their tests. 3.5, 4.6, 8: Replace and Forget, on the Forges page
  and as the FORGE requests `Replace` and `Forget`: no such request in
  `temper-client-protocol-wire`'s schema or in `temper-service`'s routing.
  5.5: a plan's first use, its card in the conversation.
- **Departs:** 1, 3.5, 4.6, 8: forges are records kept in the store, made
  by `SetUp` and `Connect` on `Making` (`Kept` or `NewForge`), not each
  configured forge; `ForgeRow` also carries `web`, `repositories` and
  `owned`. 1, 3.3, 3.5, 4.6, 4.8, 5.1, 5.5, 5.6, 8, 11: the demo is the
  playground: `StartPlayground` in place of `StartDemo`, whose
  `StartFailure` has no `not_granted`; the read `playground`
  (`PlaygroundAnswer`, its `PlaygroundState` starting, running or
  resetting) in place of `Demo` and `DemoState`; `EndPlayground`, the
  banner's "set up temper", which answers `reset` and opens
  `Welcome { Forge }`. There is no model's or forge's half, the client
  removes no account, one banner line replaces the demo's banners and the
  setup strip, the start's failures are the group's, the account's, the
  adoption's and not connected, and 11's "Hiding the demo" and "Reset
  demo" are settled by the reset (`temper-client-domain`'s
  `forge/playground.rs`; `temper-client-view`'s `frame.rs`
  `playground_banner`). 3.5, 4.6, 8: a forge is removed, confirmed on
  Forges, by `RemoveForge`, answered `removed` or `remove_refused`; a set
  up found unfinished offers Go on and Start over (`forge/connect.rs`
  `Shown::Unfinished`). 3.3: of the addresses, the forge's side is told
  only of `Adopt`. Each because forges are store records and the
  playground is reset (`temper/docs/design/draft/playground-reset.md`, 3,
  4, 6).
