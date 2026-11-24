# An agent whose host is gone: contracts

2026-11-24. The shapes the changes of the goal building
[`host.md`](host.md), section 4's *An agent whose host is gone ends on
its own*, share, and no other area reads (temper's
`draft/design-economy.md`, 4.3). They bind as `host.md` does. The
goal's last change deletes this document once the code holds them.

## 1. The figure

In `smith-agent-profile`, beside `teardown_bound` in
`src/profile/bounds.rs`, re-exported at the crate's root as
`host_limits` is:

```rust
/// The most an agent takes to exit once its host is gone: the link's
/// cancel grace plus its exit grace (protocol/limits.md, section 7).
/// `None` when the sum overflows.
pub fn orphan_exit(host: &smith_host_domain::Limits) -> Option<skein_lib::Duration>;
```

The agent derives `host` once, at startup, with `host_limits` over its
own profile, limits and endpoints, and refuses an overflow of either as
`derived`, naming `orphan_exit` for this one. Its effective limits list
it as `orphan_exit`, and the link's no-progress deadline as
`no_progress`, in the layer that lists `teardown`.

## 2. The loss

In `smith-agent-service`, one entrance in `src/service/` for a host
gone, whatever showed it:

```rust
/// The host is gone: start cancel, grace, drop (domain/host.md, section 4).
/// A later loss keeps the first failure.
pub(super) fn host_lost(service: &mut Stages, failure: Failure);
```

The failures that reach it are `Failure::ChannelEnded` and
`Failure::WriteFailed`, and the silence's own, added beside them:

```rust
/// The host said and took nothing for the link's no-progress deadline
/// while the agent waited on it.
HostSilent,
```

Its standard error line is `smith: the run could not answer: the host
was silent past its deadline`.

## 3. The process world

In `smith-agent-process-world` (`smith/tests/agent-process/src/world.rs`):

```rust
/// The link's figures as the agent derives them from this world's configuration.
pub fn link(&self) -> smith_host_domain::Limits;
/// When the agent's process exited, on the world's clock.
pub fn exited_at(&self) -> Option<skein_lib::Time>;
```

A story bounds an end with `smith_agent_profile::orphan_exit(&world.link())`
and `world.link().no_progress`, never a number (limits.md, section 11).
