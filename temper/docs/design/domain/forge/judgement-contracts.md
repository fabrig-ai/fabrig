# A check's judgement: contracts

The shapes the changes building [judgement.md](judgement.md) share, binding while its goal runs (`../../draft/design-economy.md`, 4.3). The goal's last change deletes this document and its status once the code holds them.

## 1. The function

In `temper-engine-domain-forge-landing`, beside its run verdict (`src/verdict.rs`'s `Verdict` and `Fault`), public from the crate's root:

```rust
/// What a check showed of its verdict (judgement.md, 1).
pub enum Shown {
    /// A passed verdict: a passed run, a settled close.
    Passed,
    /// The change's own failures: a promised record naming them, or, where
    /// no record is promised, a failure the run shows with no fact of the
    /// runner's.
    Failures,
    /// Nothing the change caused: a promised record missing, not read or
    /// naming no failure; a run whose every failure is the runner's; a
    /// check's own error.
    Nothing,
}

/// A check's class (judgement.md, 2).
pub enum Judgement {
    Passed,
    Refused,
    /// The runner's, naming the first of its facts.
    Runner(Fault),
    Broke,
}

/// `runner` is the first fact of the runner's among all the run's jobs,
/// whatever else they showed; a job or run cancelled by no one of
/// temper's is `Fault::Cancelled`, the run interrupted.
pub fn judge(shown: Shown, runner: Option<Fault>) -> Judgement;
```

- `judge` holds the order of judgement.md, 2: `Failures` is `Refused` whatever `runner` is; `Nothing` is `Runner(fault)` with a fact and `Broke` without one; `Passed` is `Passed`.
- Each site maps its own evidence to `Shown` and its run's facts to `runner`, and keeps its own allowance and hold (judgement.md, 3). `Fault` keeps its variants; whether `Unreported` stays among them, now that a missing report is `Nothing`, is the change building the gate's.

## 2. The flakes setting

In `temper-engine-domain-forge-vocabulary`'s `settings/landing.rs`:

```rust
/// Whether a project's checks can flake (judgement.md, 4).
pub enum Flakes {
    /// `"none"`.
    Never,
    /// `{"rerun": N}`, N at most the deployment's `checkout.rerun_processes`.
    Rerun(u32),
}

pub struct ProjectLanding {
    // ...its members as before...
    /// Unset is `rerun` at the deployment's figure.
    pub flakes: Option<Flakes>,
}
```

- The bound a run's or job's items carry (`reds.md`, 5.2): none for `None`, 0 for `Some(Flakes::Never)`, N for `Some(Flakes::Rerun(N))`.
