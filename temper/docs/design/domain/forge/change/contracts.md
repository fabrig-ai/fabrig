# A held change: the goal's contracts

The shapes this goal's changes share while they are built (`../../../draft/design-economy.md`, 4.3), for the design of `repairs-releases.md`, 2 to 6. No lasting document cites this one, and the goal's last change deletes it with its status.

## 1. The hold record

Made by the record's change; read by the release's, the judgements' and the property's. In `temper-engine-domain-forge-change`'s `boundary.rs`:

- `pub enum Cause`: today's `Hold` enum renamed, `Flake` and `NoTest` gone, `CloseRefused` added after `Rejected`: `Stalled`, `Repairs`, `Runner`, `RedStands`, `Resolutions`, `Updates`, `BranchMoved`, `BranchMissing`, `PullClosed`, `Retargeted`, `Failed`, `Rejected`, `CloseRefused`, `Authority(AuthorityHold)`, `Questions`. It stays `Copy`.
- `pub enum Kind { Bound, Forge, Judgement, Failure }` and `impl Cause { pub const fn kind(self) -> Kind }`, by `repairs-releases.md`, 2's table: `Bound` for `Stalled`, `Repairs`, `Runner`, `Resolutions`, `Updates`, `Authority` and `Questions`; `Forge` for `BranchMoved`, `BranchMissing`, `PullClosed` and `Retargeted`; `Judgement` for `Rejected`, `CloseRefused` and `RedStands`; `Failure` for `Failed`.
- `pub struct Hold { pub cause: Cause, pub since: Wall, pub words: Box<[u8]> }`, the record; `State::Held { was: Box<State>, hold: Hold }`, its field `why` renamed `hold`, `was` never `Held`.
- The cause alone where only a cause is meant: `Decision::Hold(Cause)`, `Change.released: Option<Cause>`, `Change.ended_held: Option<Cause>`, `Facts.drift: Option<Cause>`.
- The policy makes the record with empty `words`; the decision that holds fills them before it commits, so that only a record read from the previous build is stored without words.

The judgements' change adds `pub judged: Option<Judged>` to `Hold`, `pub struct Judged { pub head: Commit, pub spec: u64 }`, some for a `RedStands` held by this build alone, and its store field (`repairs-releases.md`, 5).

## 2. The measure act

Made by jig's change; read by ctl's, the host tool's and the judgements' changes.

- `jig-client-domain`'s `Ask::Measure { project: u32, task: u64, revision: Revision }`, beside `Ask::Release`, with its wire record and `Ask`'s added arm in jig's client schema and both translations; answered as `Release` is.
- `jig-domain-core-vocabulary`'s `NamedAction::Measure { target: u64, revision: u64 }`, beside `NamedAction::Release`, which a run's host tool asks.
- `jig-domain-tasks`'s `Why::Measured`, beside `Why::Released`: the due reason of a task a measurement released, its hold's revision in `revisions.hold` as a release's is, kept in the task's record until its procedure's next start reads it.
- temper's `forge::TaskRelease { revision, at, measure: bool }`, its store record's added `measured: bool = false` (mapped `measure = measured` in the store's maps, since the codec generator reserves `measure` for its own method), made by the judgements' change: true where the release it took was `Why::Measured`.
