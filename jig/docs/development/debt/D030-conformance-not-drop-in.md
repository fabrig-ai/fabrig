# D030: The conformance kit still needs mixed application adapters

- **Kind:** tests
- **Where:** `jig/testing/jig-conformance/src/harness.rs`, `jig/testing/jig-core-kit/src/conformance.rs`, `jig/tests/conformance/src/lib.rs`, `jig/tests/ops/src/conformance/mod.rs`, `jig/tests/ops/src/joint/engine.rs`, `temper/tests/conformance/src/lib.rs`
- **Seen:** 2026-10-31 at `c30cf6a03`, in the jig code-quality review (T6 "the conformance world is not drop-in", A11 F1, F2; order of work, item 18)
- **Checked:** 2026-11-06 at `82d63043ff`
- **Value:** medium (an application adopts the kit without adapters of its own)
- **Size:** M (five mixed boundary functions across four adopters)
- **Status:** open
- **Paid when:** an application adopts `jig-conformance` by naming its root, its store and its fakes; the kit answers the core's own peers itself.
- **Measure:** `awk '/pub trait Application/,/^}/' jig/testing/jig-conformance/src/harness.rs | grep -cE '^\s+fn '` (19 at this writing)

## What

`Application` has 19 functions and 7 associated types. `jig_conformance::core`
builds the shared store view and snapshot, answers account requests and renders
settled answers. The referee owns its effect vocabulary, and the scenario policy
supplies each kind's recovery class. These shared parts still need an adapter
that mixes the core's boundary with the application's own.

The four `Application` impls measure as follows, counting physical lines from
`impl Application for ... {` through its closing brace, including blanks and
comments. The five function columns count each signature through its closing
brace; attributes are outside those counts.

| Adopter | Impl lines | `deliver` | `poll` | `inbound` | `cold` | `observed` |
|---|---:|---:|---:|---:|---:|---:|
| `Testing` | 478 | 155 | 45 | 12 | 6 | 18 |
| `Ops` | 764 | 375 | 105 | 19 | 11 | 54 |
| `OpsApplication` | 328 | 166 | 32 | 12 | 6 | 39 |
| temper's `Temper` | 522 | 72 | 102 | 30 | 27 | 62 |

Each adopter still writes these five functions, directly or by wrapping `Ops`:

- `deliver` routes core loads, account requests, worker calls and party replies
  alongside connector and application deliveries. It translates the answers into
  root events and records `Drained` in the adopter's outside evidence.
- `poll` advances hosts, parties and systems, returns their events, and supplies
  the first period and account or credential inputs alongside application scripts
  and retries. Adopters differ in whether an existing durable period suppresses
  the first-period event.
- `inbound` records core requests and worker completion evidence alongside
  application requests, keys and protocol acknowledgements, before the root
  receives them.
- `cold` observes the last durable store and reconnects retained peers, while
  clearing the adopter's correlation state and preparing its scripts and retries
  for the rebuilt root.
- `observed` combines durable core snapshots, host and system evidence with
  application observations and milestones, preserving their arrival order for
  the referee.

The **Paid when** does not hold: naming a root, store and fakes still leaves
these mixed translations for the application to write.

## Why it matters

`jig/docs/design/README.md`, 6.6 ships the conformance world for applications.
Repeated core arms in application adapters can drift. The departure from the
shared-peer promise is recorded in `jig/docs/design/domain/testing.status.md`, 5.

## What stands in the way

- `Harness<A>` needs a trait to run concrete application boundaries generically
  in the world (`jig/docs/design/domain/testing.md`, 1).
- `type Domain = root::Domain` rules out implementing a kit trait on another
  crate's root through the orphan rule; step code itself holds no traits.
- Decoding reads `jig-protocol-charter`, which links smith. Every application
  links this kit, including applications without agents, so shared decoding
  cannot bring that dependency into it.
- The first period and `Drained` differ between adopters in events the referee
  sees. Moving them into one shared answer would change observed behaviour.

A trait must remain, so the Measure has no target of 0. Sub-traits or an
argument enum that folds several functions into one would reduce the count
without removing the mixed adapters. The remaining work needs a boundary that
respects these constraints and lets the kit answer the core's peers itself.
