# panics: execution

2026-11-13. How [panics.md](panics.md) gets carried out. It designs nothing: each milestone writes its own design from the draft as it runs, and lands whole. Plans use the focused and fuzzy suites alone. Each milestone's figure is the count of the draft's section 7, read from main after it lands.

## 1. The milestones

| Milestone | Draft parts | Owner | Needs | Its figure after landing | State |
|---|---|---|---|---|---|
| P1, the held login returned from outside its test: `temper/scripts/proof-live.sh`'s exit trap taking `Lent`'s case with no box up; `Lent` and `run_walk`'s catch out; the ignored temper-box tests about the held record rewritten for the launcher, or gone; `temper/docs/design/testing/live-tier.md` following | 5 | temper/dev | none | temper-box's ignored tests and count | not started |
| P2, the rule and the count: `skein/docs/foundation/programming-model.md`, 10.4 and 10.1, as the draft's section 3.2 words them; the count of section 7, with main's figures as its list; `skein/docs/design/testing/README.md`, 5 and 6, following section 4's kits | 3, 7 | temper/dev | none | the count against the draft's section 2 | not started |
| P3, skein's checks return their findings: the world's and the simulator's referees, judges and failure reports, io's judge, `skein-rows`, the fake peers', the codegen tables; their tests, and skein's ignored tests of that kind | 4 | temper/dev | P2 | skein's ignored tests and count | not started |
| P4, the other trees' checks: jig's referees and kits (`jig-core-kit`, `jig-client-kit`, `jig-tasks-kit`, `jig/tests/conformance`), smith's program referee, and temper's fake git, real-loop, walking and browser stories; their ignored tests of that kind | 4 | temper/dev | P3 | those trees' ignored tests and count | not started |
| P5, limits and assertions: tightness by what a build used; assertion tests out; a refusal, with its design, where the cause is outside the process, the store's contents among them; the ignored tests of those kinds in every tree | 3.1, 4 | temper/dev | P2 | no test ignored for Cranelift but P6's; `catch_unwind`'s count | not started |
| P6, the one-process mode's starved pass without `resume_unwind`: `skein-world`'s `Starved::pass` and `tools/tiers/`' runner, with its fixture | 4 | temper/tiers | P2 | `resume_unwind`'s count | not started |
| P7, the ban: `catch_unwind` and `resume_unwind` in every `clippy.toml` variant's `disallowed-methods`, and out of the count | 7 | temper/dev | P1, P3 to P6 | none of either left | not started |
| P8, skein's `#[should_panic]` tests, by section 4's kinds | 4, 7 | temper/dev | P7 | skein's count of them reaching none | not started |
| P9, jig's `#[should_panic]` tests | 4, 7 | temper/dev | P7 | jig's count of them reaching none | not started |
| P10, temper's, smith's, hearth's and `tools/`' `#[should_panic]` tests; then the check refusing any | 4, 7 | temper/dev | P8, P9 | none left | not started |

## 2. Order

- **No priority mark:** panics aren't one of v0.1.0's critical pieces. Plans start when places are free and marked work isn't about to arrive.
- **P1 first:** it's the one with a stake, a person's login. Until it lands, a held-login story builds with LLVM.
- **P2 beside P1:** the words and the count stop new catches before the conversions start.
- **P3 before P4:** jig's and temper's kits are built on skein's world.
- **P5 and P6 any time after P2,** side by side with P3 and P4.
- **P7 once the counts it bans are none.**
- **P8 to P10 last,** a tree at a time.
- **Each milestone lowers the count's list** in the change that lowers a count, and removes the ignores of the tests it converts.
