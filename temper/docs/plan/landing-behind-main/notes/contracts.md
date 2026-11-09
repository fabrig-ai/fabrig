# landing-behind-main: the binding calls

The shapes and words each task builds to. No two tasks of this plan share
code: each owns its half, and these are the run's decisions where a reader
or an answer call needs them.

## A. A head behind its base

- **Containment** (`temper/docs/design/domain/forge.md`, 5.3): the connector's
  `ancestry_status` reads a `Read::Commits` result as

  | Result | Status |
  |---|---|
  | `Ok(Commits { commits: [], more: false })` | `Passed` (contains) |
  | `Ok(Commits { commits: nonempty, more: _ })` | `Failed` (lacks) |
  | `Err(TooLarge)` | `Failed` (lacks) |
  | `Ok(Commits { commits: [], more: true })` | `Unknown` |
  | any other answer or error | `Unknown` |

  Both readers take it: a change's step (`StepStage::Compare`) and a
  catch-up's (`StepStage::CatchUp`).
- **Forgejo's commits plan** (`skein/docs/design/forge.md`, 5.2, the
  `Read::Commits` row): Forgejo lists a comparison's commits newest first.
  The plan keeps the oldest commits the answer's rows and bytes hold, giving
  up the newest it holds as older ones come, and counts every row it is
  given. The page asked, its commits from `skip` to
  `min(total, skip + page_size)` counted from the oldest, is answered when
  the kept commits hold it whole, oldest first, with
  `more = total > skip + page_size`; otherwise `TooLarge`. `total_commits`
  unequal to the rows given stays `InvalidAnswer`; a decoder's `Bound` or
  `TooLong` stays `TooLarge`.

## B. A resolution that starts

- **The charter bound** (`temper/docs/design/domain/agent.md`, 4.3), in
  `temper-service/src/assignment.rs`:

  ```rust
  /// A worker's charter allowance where the engine doesn't know it: smith's
  /// `run_bytes` at its floor, which the stock agent's `tool_payload` doesn't
  /// raise (smith/docs/design/protocol/limits.md, 3.10;
  /// temper/docs/design/domain/agent.md, 4.3).
  pub const WORKER_CHARTER_BYTES: u64 = 65_536;

  /// The bound a charter is fitted to: the hub's, and no more than its
  /// worker takes (temper/docs/design/domain/agent.md, 4.3).
  #[must_use]
  pub fn charter_bound(hub: u64, agent: Option<u64>) -> u64
  ```

  `charter_bound(hub, agent) = hub.min(agent.unwrap_or(WORKER_CHARTER_BYTES))`.
  `settings::Engine` gains `agent_charter_bytes: Option<u64>`, the local
  agent's `host_limits.charter_bytes`, none where the process has no local
  agent, set beside `host_reply_bytes`. `Encoding.charter_bytes` is
  `charter_bound(settings.jig.limits.hub.charter_bytes,
  settings.agent_charter_bytes)`. The link's own sizes keep
  `hub.charter_bytes`.
- **A named conflicting file** (`temper/docs/design/domain/forge.md`, 8.4,
  *A resolution*): in a resolution's pull section (`PullMode::Conflict`),
  a file is carried whole as now where all its bytes (`File: `, its path,
  `\nBefore:\n`, the before side, `\nAfter:\n`, the after side and `\n`) fit
  the room its section has left; otherwise it is named, these two lines
  exactly:

  ```text
  File: PATH
  [not carried, past the brief's room: B bytes before, A bytes after; read it in your checkout]
  ```

  `B` and `A` in decimal, a side the file lacks as `absent before` or
  `absent after` in its place. Other modes carry files as now.
