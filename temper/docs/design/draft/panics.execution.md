# panics: execution

2026-11-13. How [panics.md](panics.md) gets carried out. It designs nothing: each milestone writes its own design from the draft as it runs, and lands whole, bringing into line the design text its status file names for the tests it converts. Plans use the focused and fuzzy suites alone. Each milestone's figure is the count of `panics.md`, 7, read from main after it lands.

## 1. The milestones

| Milestone | Draft parts | Owner | Needs | Its figure after landing | State |
|---|---|---|---|---|---|
| P1, what a failed test leaves outside its process: `temper/scripts/proof-live.sh` returning the held login once the test process has ended, stopping and reaping it first on a signal, with the story's box where one is up or one started over the store; `Lent` and `run_walk`'s catch out; temper-box's ignored tests of the held record rewritten for the launcher, or gone; `temper/docs/design/testing/live-tier.md` following; `skein-throwaway-forgejo`'s Forgejo, forge conformance's runner and smith's live agent process started dying with their parent, their stores in the test's scratch | 5 | temper/dev | none | temper-box's ignored tests and count | done |
| P2, the rule and the count: `skein/docs/foundation/programming-model.md`, 10.4 and 10.1, and `skein/docs/foundation/testing-strategy.md`, 1 and 10, as `panics.md`, 3.2, words them; the count of `panics.md`, 7, with main's figures as its list, in the gate (`.config/gate/README.md`); `skein/docs/design/testing/README.md`, 5, 6 and 9, following the kits of `panics.md`, 4 | 3, 7 | temper/dev | none | the count against the draft's section 2 | built by goal 2655 (`skein/docs/foundation/programming-model.md`, 10.1 and 10.4; `skein/docs/foundation/testing-strategy.md`, 1, 8 and 10; `skein/docs/design/testing/README.md`, 5, 6 and 9; `tools/lints/design.md`, 7; `.config/gate/README.md`, 8) |
| P3, skein's checks return their findings: the world's and the simulator's referees, judges and failure reports, io's judge, `skein-rows` reporting its first failure, the fake peers', `skein-codegen`'s tables and patch check; their tests, and skein's ignored tests of that kind | 4 | temper/dev | P2 | skein's ignored tests and count | not started |
| P4, the other trees' checks: jig's referees and kits (`jig-core-kit`, `jig-client-kit`, `jig-tasks-kit`, `jig/tests/conformance`, the bundle of cases), smith's program referee, and temper's fake git, real-loop, walking, one-process referee and browser stories; their ignored tests of that kind | 4 | temper/dev | P3 | those trees' ignored tests and count | not started |
| P5, limits and assertions: tightness by what a build used; assertion tests out; a refusal, with its design, where the cause is outside the process, the store's contents among them, jig's link panics first; the ignored tests of those kinds in every tree | 3.1, 4 | temper/dev | P2 | no ignored test of the limit and assertion kinds | built by goal 2700 (`skein/docs/design/ui/README.md`, 13 and 14; `skein/docs/design/net/http/testing.md`; `skein/docs/design/net/tls.md`, 5; `skein/docs/design/runtime/io/README.md`, 5; `skein/docs/design/data/json.md`, 6; `jig/docs/design/domain/engine/restart.md`; `jig/docs/design/domain/engine/README.md`, 9; `jig/docs/design/domain/accounts.md`, 3.6; `jig/docs/design/protocol/store/README.md`, 10.9; `jig/docs/design/protocol/llm.md`, 8; `temper/docs/design/domain/root.md`, 8.1) |
| P6, the one-process runner: a starved pass exiting 0 after its line, and the runner starting a new process for the names left after any process that ends early; `skein-world`'s `Starved::pass`, `tools/tiers/`' runner and fixture, and `tools/tiers/design.md`, 11.5 and 11.6; skein-world's starved-pass and teardown tests across their child's end | 4 | temper/tiers | P2 | `resume_unwind`'s count | built by goal 2702 (`tools/tiers/design.md`, 10.2, 11.5, 11.6 and 11.10; `skein/docs/design/testing/README.md`, 5) |
| P7, the ban: `catch_unwind` and `resume_unwind` in `disallowed-methods` in every `clippy.toml` the gate's clippy reads, the managed variants, smith's copies and the root `tests/clippy.toml`; the count refusing any | 7 | temper/dev | P1, P3 to P6 | none of either left | not started |
| P8, skein's `#[should_panic]` tests, by the kinds of `panics.md`, 4 | 4, 7 | temper/dev | P7 | skein's count of them reaching none | not started |
| P9, jig's `#[should_panic]` tests | 4, 7 | temper/dev | P7 | jig's count of them reaching none | not started |
| P10, temper's, smith's, hearth's and `tools/`' `#[should_panic]` tests; the count refusing any | 4, 7 | temper/dev | P8, P9 | none left | not started |

## 2. Order

- **No priority mark:** panics aren't one of v0.1.0's critical pieces. Each plan starts once the user approves it, when places are free and marked work isn't about to arrive.
- **P1 first:** it's the one with a stake, a person's login and real processes left running. Until it lands, a held-login story builds with LLVM.
- **P2 beside P1:** the words and the count stop new catches before the conversions start.
- **P3 before P4:** jig's and temper's kits are built on skein's world.
- **P5 and P6 any time after P2,** side by side with P3 and P4.
- **P7 once the counts it bans are none.**
- **P8 to P10 last,** a tree at a time.
- **Each milestone lowers the count's list** in the change that lowers a count, and removes the ignores of the tests it converts.
