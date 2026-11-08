+++
id = "ways-surface"
part = "paths"
kind = "interface"
needs = []
files = ["temper/crates/temper-engine-domain-ways/**", "Cargo.toml", "temper/crates/temper-engine-domain/Cargo.toml", "temper/crates/temper-engine-domain/src/lib.rs", "temper/crates/temper-engine-domain/src/translate.rs", "temper/crates/temper-engine-domain/src/boundary.rs", "temper/crates/temper-engine-domain/src/decision.rs", "temper/crates/temper-engine-domain/src/route/see.rs", "temper/crates/temper-engine-domain/src/route/kinds.rs"]
effort = "high"
minutes = 90
+++

## Goal

Create the crate every other engine task of this plan builds on,
`temper-engine-domain-ways`, with its public types and its small certain
functions, and widen the root's boundary for the `ways` document, the
`View` read and `see ways`, so that the derivation, the stock's data, the
root's reads, the family's encoding and `see`'s text can be written in
parallel. `derive` is a stub here; task `ways-derive` writes it.

Acceptance:

- The crate exists as `notes/contracts.md`, section 1 has it, every name
  and signature as written, with the `step` lint profile, in the
  workspace, and `cargo doc`-clean comments citing
  `temper/docs/design/domain/forge.md`, 8.13 and
  `temper/docs/design/protocol/configuration.md`, 13.10 at module level.
- Bodies: `Stop::same`, `Path::same`, `items`, `laid`, `laid_ways`,
  `current`, `can`, `first_sentence` and `subject` whole, each with unit
  tests; `derive` returns the inputs' branch and repository (`OWNER/NAME`)
  with no stops, and says in its doc comment that `ways-derive` writes it.
- The root's boundary of contracts, section 4: `Config::ways_stock`,
  `ForgeRead::View`, `ForgeReadAnswer::View`, `Delivery::Ways`,
  `SeeAnswer::Ways`, `temper_engine_domain::ways`; every compiler-forced
  arm answering as below; the workspace builds and the focused tests of
  every package you touched pass.

## Inputs

- `temper/docs/plan/ui-ask-m3a/notes/common.md` and `notes/contracts.md`,
  sections 1 and 4 (binding).
- `temper/docs/design/domain/forge.md`, 8.13; `temper/docs/design/protocol/configuration.md`,
  13.10 and 13.2 (*A project's rule may name a branch*, the forge's held
  rule 1); `temper/docs/design/client/README.md`, 4.6 (*A view, the read
  `View`*).
- The forge's types the crate takes: `temper-engine-domain-forge`'s
  `settings::{Held, LandingRule, ProjectWays, Value, project_stock}`,
  `Follows`, `covers_branch`; `temper-engine-domain-forge-change::Way`.
- How a lint profile is set: `tools/lints/design.md`; `python3 -I -B
  tools/lints/lints.py --set step temper/crates/temper-engine-domain-ways`.

## Owns and may change

- **Owns:** `temper/crates/temper-engine-domain-ways/**` (new), until
  `ways-derive` takes `derive`.
- **Wiring, one line each:** the root `Cargo.toml`'s `members` and
  `[workspace.dependencies]` entries; `temper/crates/temper-engine-domain/Cargo.toml`'s
  dependency; `temper/crates/temper-engine-domain/src/lib.rs`'s
  `pub use temper_engine_domain_ways as ways;`.
- **The root's boundary,** each a variant or field and nothing else:
  `translate.rs` (`Config::ways_stock`), `boundary.rs` (`ForgeRead::View`,
  `ForgeReadAnswer::View`), `decision.rs` (`Delivery::Ways`, and the
  bound's match for `SeeAnswer::Ways`, which is bounded as `Settings` is),
  `route/see.rs` (`SeeAnswer::Ways`).
- **Compiler-forced arms, answering as the code is until the task named:**
  - `route/kinds.rs`: `ForgeRead::View` admitted for a party with a role
    in `project` (`domain.core.people.role(party, project).is_some()`), as
    kinds 2, 6 and 7 are in a project's scope; answered
    `ForgeReadAnswer::View(Box::new(ways::View::NotDrawn(b"not drawn yet".into())))`
    (`ways-view` replaces it).
  - `temper-service`: `family.rs`'s `forge_read` arm for
    `ForgeReadAnswer::View` returns `None`, and `route.rs`'s arm for
    `Delivery::Ways` sends nothing (`ways-family` replaces both).
  - `temper-engine-smith/src/see.rs`: `SeeAnswer::Ways` answered as
    `NotYet` is, `see ADDRESS: not there yet.`, and its `following` arm
    with no addresses (`ways-read` replaces both).
  - Every construction of `translate::Config` in any package (worlds,
    fixtures, the service's startup, the playground) gains
    `ways_stock: Box::default()` (`stock-ways` fills the service's).
- **Options, pre-approved:** if a type of the forge's the contract names
  is not public, add its `pub use` to `temper-engine-domain-forge/src/lib.rs`
  and say so. If `Held` lacks `Hash` or `Eq`, derive them there. If a
  derive of the contract's types fails for a forge type's missing derive,
  add that derive, nothing else.

## Steps

1. Make the crate: `Cargo.toml` as `temper-engine-domain-forge-tiers`'s,
   with the dependencies of contracts, section 1; set its lint profile with
   `tools/lints/lints.py --set step`; `src/lib.rs` with
   `#![cfg_attr(not(test), no_std)]`, `extern crate alloc;`, modules
   `path` (Actor to `derive`), `ways` (`StockWay` to `current`, the
   documents, `View`, `subject`) and `words` (`can`, `first_sentence`),
   re-exported at the root.
2. Write the bodies:
   - `items`: for numbers 1 to `RULES`, `Set(Value::Rule(..))` where the
     way sets it, else `Remove`, each `kind: 2, scope: project`; then, where
     `ways` is some, kind 7 number 1 `Set(Value::Ways(laid))`, the
     project's own (or `project_stock(7)`'s) with `drafts`, `goals` and
     `way` replaced.
   - `laid`: deployment rules (scope 0) stay in their order; the
     project's numbers take the set rule, or on `Remove` the `stock` rule
     of that number re-scoped to `project`, or nothing; the result ordered
     as `Domain::landing_rules` orders (deployment first, then the project's
     by number).
   - `laid_ways`, `current`, `subject` (no `as`: `u64::from`), `can`'s
     words of contracts, section 1, `first_sentence` as its comment says.
3. Tests in the crate (`src/tests.rs`, `#[cfg(test)]`): one table each for
   `items` (the three ways of 13.10 over a project with and without its own
   kind 7), `laid` (set, remove back to the stock's, remove with no stock,
   deployment rules kept), `current` (first equal; none), `first_sentence`
   (a full stop, none, a long one cut at a char's end), `can` (none, each,
   all). Build the three ways in the test as 13.10's table has them; they
   are test data here, the stock's are `stock-ways`'s.
4. The root's boundary and the forced arms, as above.
5. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo
   clippy -p temper-engine-domain-ways -p temper-engine-domain -p
   temper-engine-smith -p temper-service --all-targets -- -D warnings`;
   `heavy cargo nextest run -p temper-engine-domain-ways`; `heavy cargo
   check --tests` for each other package whose `Config` construction you
   touched; the focused tests of `temper-engine-domain`'s `--lib` and
   `temper-service`'s `--lib`. Commit and push.

## Report

As the implementer's text asks; also list each file outside your `files`
where a forced arm or field went, and any forge type you made public.
