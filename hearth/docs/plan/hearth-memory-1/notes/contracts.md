# hearth-memory-1: contracts

The run's binding calls, by the design in `hearth/docs/design/domain/render.md`, 8.4 and `hearth/docs/design/domain/testing.md`, 8.4.

## 1. Every formula's fate

The rule: a formula stays when it sizes an edge (`skein/docs/foundation/programming-model.md`, 6.3), or when a test that isn't a peak test
reads it, and then `valid` reads it too. Any other formula goes. No production code reads a `worst_case` today, so nothing but `valid` reads
what stays.

### Goes, in `sums`

| Formula | Its tests |
|---|---|
| `hearth_service_builder::worst_case` | `the_shipped_limits_have_a_worst_case_within_1_gib`, `a_larger_limit_gives_a_larger_worst_case` |
| `hearth_domain_builder::worst_case` | none |
| `hearth_protocol_builder::limits::worst_case` | none |
| `hearth_domain_store::limits::worst_case` | none |
| `hearth_domain_build::limits::worst_case` | `the_plan_fits_its_worst_case` |

### Goes, in `render`

`hearth_domain_render::limits::worst_case`, `revision::decoded_worst_case`, `compose::worst_case`, `entries::worst_case`,
`entries::held_worst_case`, `errors::Collector::worst_case`.

### Stays, renamed, in its family's task

| Task | Today | Becomes |
|---|---|---|
| `git` | `hearth_protocol_git::limits::worst_case(decoder, limits)` | `limits::decoder_bound(decoder, limits)` |
| `accounts` | `hearth_protocol_accounts::worst_case(limits)` | `decoder_bound(limits)` |
| `mmdebstrap` | `hearth_protocol_mmdebstrap::output::worst_case`, `packages::worst_case`, `tail::Tail::worst_case` | `output::decoder_bound`, `packages::decoder_bound`, `tail::Tail::decoder_bound` |
| `config` | `hearth_protocol_config::json::worst_case`, `json::held`, `schema::worst_case`, `declaration::worst_case` | `json::decoder_bound`, `json::decoded_bound`, `schema::decoder_bound`, `declaration::decoder_bound` |
| `template` | `hearth_protocol_template::limits::worst_case(decoder, limits)`, `expression::worst_case` | `limits::decoder_bound(decoder, limits)`, `expression::decoder_bound` |
| `tree` | `hearth_protocol_tree::limits::worst_case(kind, limits)`, `limits::held(kind, limits)`, `decode::worst_case` | `limits::decoder_bound(kind, limits)`, `limits::decoded_bound(kind, limits)`, `decode::decoder_bound` |
| `tree` | `hearth_domain_tree::limits::worst_case(limits)` | `limits::walk_bound(limits)` |
| `render` | `hearth_domain_render::evaluate::worst_case`, `evaluate::template::worst_case` | `evaluate::evaluation_bound`, `evaluate::template::run_bound` |
| `valid` | `hearth_client_protocol_records::printer::worst_case` | `printer::valid(limits) -> bool`, `limits.chunk != 0` |

## 2. `valid`

Built last, in `valid`, on what section 1 leaves:

```rust
/// Whether these limits are representable: every edge bound they size is
/// finite (skein/docs/foundation/programming-model.md, 6.3).
#[must_use]
pub fn valid(limits: &Limits) -> bool
```

It sits in `hearth_protocol_builder::limits`, `hearth_domain_render::limits`, `hearth_domain_builder` and `hearth_service_builder`, and each
calls the ones below it. `hearth_service_builder::Service::new` asserts it. The assertion beside it, `limits.io.is_usable() && limits.queue >= 3`,
stays as it is: it is a D001 form.

## 3. `parent::checked`

In `hearth/tests/render/src/parent.rs`:

```rust
pub fn checked(repository: &Repository, seed: u64, limits: limits::Limits, ceiling: u64) -> Run
pub fn checked_diff(from: &Repository, to: &Repository, seed: u64, limits: limits::Limits, ceiling: u64) -> Run
```

`drive` takes `skein_world::Memory::Within(ceiling)` or `Memory::Unchecked`.

## 4. The large render

- **The scenario:** a revision of 4,096 entries, 512 of them documents, declarations and templates, the rest static files, with 64 targets,
  through the builder world at `hearth::limits()`, the shipped limits.
- **Its name:** `fixtures::large::scenario()`, in `hearth/tests/builder/src/fixtures/large.rs`.
- **Its test:** `a_large_render_holds_no_more_than_its_ceiling`, in `hearth/tests/builder/tests/suite/memory.rs`.
- **The design names these figures.** A task that must change them says so in its report, and `close` carries them into the design.

## 5. Tests a formula already has

These stay, with the formula's new name in the test's name and body. Each is the formula's test of its value, so `notes/common.md`, "A formula
left" asks no other. Every other formula that stays gets one.

| Formula | Today's test | Becomes |
|---|---|---|
| `hearth_protocol_accounts::decoder_bound` | `the_worst_case_covers_the_list_and_the_names` | `the_decoder_bound_covers_the_list_and_the_names` |
| `hearth_domain_tree::limits::walk_bound` | `the_worst_case_is_a_sum_that_fits_or_nothing` | `the_walk_bound_is_a_sum_that_fits_or_nothing` |
| `hearth_protocol_tree::limits::decoder_bound`, `decoded_bound` | `worst_case_and_held_are_none_past_a_u64_for_a_resolution_only` | `the_bounds_are_none_past_a_u64_for_a_resolution_only` |
| the same | `worst_case_and_held_are_none_past_the_encoders_depth` | `the_bounds_are_none_past_the_encoders_depth` |
| `hearth_domain_render::evaluate::template::run_bound` | `a_run_whose_worst_case_cannot_fit_returns_none` | `a_run_whose_bound_cannot_fit_returns_none` |
