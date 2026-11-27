# Contracts: the composed workload takes part in campaigns

The shapes this goal's changes share and no other area reads (`temper/docs/design/draft/design-economy.md`, 4.3), for the composed workload's draws valid at any seed (`workloads.md`, 5), its sweeps' cases (8.5) and its catalogue (`composed-catalogue.md`). They bind as the design does. The goal's last change deletes this document and its status.

## 1. The draw without a world

```rust
/// The composed world's draw of faults and acts for a seed and the kinds a sweep
/// configures, made as `World::faulted_workload` makes it, without a world (workloads.md, 5).
pub struct FaultDraw { /* plans, acts, ineligible kinds */ }

impl FaultDraw {
    pub fn new(seed: u64, kinds: &[Kind]) -> FaultDraw;
    /// Each planned kind with the logical name of the goal it strikes, in plan order.
    pub fn planned(&self) -> Vec<(Kind, &str)>;
    /// Each configured kind the draw found no eligible goal for, in `Kind`'s order.
    pub fn ineligible(&self) -> Vec<Kind>;
    /// The parties' acts, each hold followed by its release.
    pub fn acts(&self) -> &[temper_workload::parties::Action];
}
```

- **At `temper_system_world::composed::FaultDraw`.** `World::faulted_workload(seed, kinds)` plans from it, so the step tests' draw is the world's. An observed kind, a bounds kind or `WRONG_TREE` (`composed-catalogue.md`, 1), is planned with no goal: `planned` gives it with the empty name, `WRONG_TREE` only where a goal is eligible for it and ineligible otherwise.
- **Its digest:** the step tests pin seed 6's draw for 8.2's kinds by a digest of `planned` and `acts`, so that a change that moves it fails there first.

## 2. The kinds a sweep configures

```rust
/// 8.2's kinds, in their order: the mixed sweep's at the gate.
pub const GATE_KINDS: &[Kind];
/// The catalogue's kinds beyond them (composed-catalogue.md, 2), grouped by
/// family in 8.2's order, each group under a comment naming its family.
pub const CAMPAIGN_KINDS: &[Kind];
```

- **At `temper_system_world::composed`.** The mixed sweep configures `GATE_KINDS` where `gate()` holds, and both lists otherwise. `World::faulted_workload` and `FaultDraw::new` take any kinds of the two, and refuse another.
- **A catalogue change** adds its kind to its family's group of `CAMPAIGN_KINDS`, and its draw, injector and target beside its family's, so that changes adding kinds of other families meet in no line of each other's.
