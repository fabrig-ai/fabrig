+++
id = "fake-pace"
part = "fake-pace"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-fake/src/llm.rs",
  "temper/crates/temper-fake/src/llm/**",
  "temper/crates/temper-fake/src/forge.rs",
  "temper/crates/temper-fake/src/forge/**",
  "temper/crates/temper-fake/src/wire.rs",
  "temper/crates/temper-fake/src/read.rs",
  "temper/crates/temper-fake/src/tests.rs",
  "temper/crates/temper-fake/src/tests/**",
]
effort = "high"
minutes = 60
+++

# The fakes' documents of their own gain `pace`

## Goal

A playground story's replies and CI take time a person can follow, set in its documents
(`temper/docs/design/playground.md`, 5.1). temper-fake's two documents of their own gain the
designed member `pace` (`temper/docs/design/protocol/setup.md`, 8.4, "Both documents may also hold
`pace`"): the LLM's `{"reply_ms"}`, skein's injected latency with its least and its most alike;
the forge's `{"ci_ms"}`, the `Checks` latency with its least and its most alike. Absent, 0: the
box's fakes and every runner keep none, and nothing they do changes.

Acceptance:

- `temper_fake::llm::Document::decode` reads an optional `pace: {"reply_ms": u32}`, at most 60000,
  refused past it or with any other member, naming it, as the document's other refusals do. The
  deployment's form (`Document::checked`, the box's derivation) has no pace: 0.
- The LLM domain that `Document::domain` and `configuration` build, and the domain limits the
  peer is given (`wire.rs`, ~94), have `latency_min == latency_max == reply_ms`; `llm::LIMITS`
  stays the const it is (its readers outside the crate keep it), and the paced limits are
  `LIMITS` with the two latencies set.
- `temper_fake::forge::Document::decode` reads an optional `pace: {"ci_ms": u32}`, at most 60000,
  refused past it, naming it. The forge `Document::seed` builds gives each repository's `Checks`
  `latency_min == latency_max == ci_ms`. `forge::checks(ci)` keeps its signature and its zero
  latency (the real loop calls it: `temper/tests/real-loop/src/green_light.rs`, `repair.rs`); a
  paced form beside it, e.g. `checks_paced(ci, ci_ms)`, is yours to name. `Document::fake` (the
  box's) has none.
- Tests (below) pass; every existing test of `temper-fake` and of its readers passes unchanged.

## Inputs

- `temper/docs/plan/playground-temper/notes/common.md`.
- `temper/docs/design/protocol/setup.md`, 8.4: the standalone documents' members (the tables
  before "Both documents may also hold `pace`") and the `pace` paragraph.
- `temper/docs/design/playground.md`, 5.1 ("Its pace").
- `skein/docs/design/fake-llm.md` (its injected latency) and `skein/docs/design/fake-forge.md`, 2.3
  (`Checks`' latency): read only.

## Owns and may change

- `temper-fake/src/llm.rs` (and `llm/`): the `Document`'s pace, its decode (`read.rs` if the
  member reader lives there), `domain`, `configuration` and `Configuration` (a field for the
  paced limits, if the peer needs them apart from the domain).
- `temper-fake/src/wire.rs`: the peer's domain limits taken from the configuration in place of
  `llm::LIMITS` (~94). Nothing else.
- `temper-fake/src/forge.rs` (and `forge/`): the `Document`'s pace, its decode, `seed`'s checks.
- `temper-fake/src/tests.rs` and `src/tests/`: new tests beside `llm_document_checks` (~29),
  `forge_document_checks` (~201) and `ci_cue` (~522):
  - `the_llm_documents_pace_is_its_replies_latency`: a document with `"pace": {"reply_ms": 1500}`
    decodes; its domain's limits read 1500 ms least and most; one without reads zero; 60001 and an
    unknown pace member are refused, naming the member.
  - `the_forge_documents_pace_is_its_ci_latency`: the same for `ci_ms`, read on the seeded
    forge's repository's `Checks`.
  - `a_paced_reply_waits_its_pace`: on the played domain (as an existing domain test of this crate
    drives one, or the peer's `next_policy_deadline`), a query asked at `t` is answered no
    earlier than `t + reply_ms`. If no test of this crate drives the domain in time, drive it
    through `skein_fake_llm_domain` directly, as its own tests do.
- If a struct literal of `Configuration` or `Document` outside these files breaks
  (`temper/crates/temper/src/fake.rs`, `fake_backends.rs`, `temper/tools/twin`), the field's
  value there is the unpaced one (zero); that is a compiler-forced edit.

skein's fake LLM arms its latency before each piece of a turn, not only before its first
(`skein/crates/skein-fake-llm-domain/src/domain.rs`, `arm`); a turn of text lines waits once. Do
not change skein: if your timing test shows a multi-piece turn waiting per piece, list it under
`## Left` as a departure from setup.md 8.4's "before each reply's first piece".

## Steps

1. The LLM's pace and its tests; `heavy cargo nextest run -p temper-fake --lib -E 'test(pace)'`.
2. The forge's pace and its test; the same filter.
3. `heavy cargo nextest run -p temper-fake`; `heavy cargo check -p temper -p temper-twin -p
   temper-real-loop-world --tests` (the crate's cross-crate readers; name any other the compiler
   shows).
4. `cargo fmt`; old forms for temper; `heavy cargo clippy -p temper-fake --all-targets -- -D
   warnings`. Commit; push.

## Report

As the implementer's text says, and: the names of the paced limits and checks, and whether a
multi-piece turn waits once or per piece.
