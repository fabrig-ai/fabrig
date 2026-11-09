+++
id = "reach-kit"
part = "kit"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-protocol-reaches/**",
  "Cargo.toml",
  "Cargo.lock",
  "temper/docs/design/domain/forge.status.md",
]
effort = "high"
minutes = 120
+++

# The kit: what reaches a person, as one pure judgement

## Goal

A new step crate, `temper-protocol-reaches`, holds what `temper/docs/design/domain/forge.md`, 5.10 designs: the ten kinds, the four classes, a project's table of items and path sets with its check and its stock tables, path patterns, the judgement and Stop asking's offers. Both the forge's `decided` and the client build on it; nothing calls it yet.

Acceptance:

1. The crate exists at `temper/crates/temper-protocol-reaches`, a workspace member with its `[workspace.dependencies]` line, `#![no_std]` with `alloc`, depending on `skein-lib` alone, its lints the `step` profile (`python3 -I -B tools/lints/lints.py --set step temper/crates/temper-protocol-reaches`), its description "What reaches a person: the classes, a project's table and the judgement".
2. Its public surface is `temper/docs/plan/ui-reaches/notes/contracts.md`, section 1, exactly: names, fields, derives and signatures. Inside them the code is yours.
3. Its behaviour is 5.10's, every bullet: `check`'s faults in the order the note lists them, the pattern's rules, the stock tables' items in 5.10's order, the judgement's subjects, `under`, the lift, the strictest class and its reason, `Reach`'s bits, and the offers and the tables they make.
4. **Table tests** in the crate's `src/tests.rs` (a module per area if it grows, as `temper-engine-domain-ways/src/tests/` does), one table test per area, each row naming its case:
   - `check`: the three stock tables pass; one row per `Refusal` variant, each the smallest table that shows it;
   - `matches` and `well_formed`: a literal path, `*` within a segment and never across `/`, `**` at the start, the middle and the end, `**` matching no segment, `include/vsr.h` against `include/vsr.h` and `include/vsr.hh`, `src/protocol.*` against `src/protocol.c` and `src/protocol/x.c`, `docs/protocol/**` against `docs/protocol` and `docs/protocol/a/b.md`; ill-formed patterns (empty segment, leading or trailing `/`, `a**b`) false and not well formed;
   - `judge`: a fixed kind is `Wait` and asks, Away on or off; a kind at `Decide` decides; at `Ask` asks, and decides with Away on (`away` true); at `AskAway` asks with Away on; a kind with no item is `AskAway`; a wording choice inside a set at `AskAway` asks with reason `Set(n)` and its `matched` bits; a set less strict than the kind leaves the reason `Kind`; two sets at the same class give the lower number; `sets` holds every touched set; `under` naming a worded item of the kind, then of a touched set, gives its class; `under` naming an item without words, another kind's or an untouched set's is not used (`under` 0 in the reach); `always` lifts `Decide` and `Ask` to `AskAway` with reason `Always`, and leaves a fixed kind's `Wait` with reason `Kind`;
   - `stock` and `away_may_decide`: each way's items in order; true for Agents review and Draft first, false for You approve;
   - `offers` and `narrowed`: a kind at `Ask` offers `Decide(Kind)`, whose table decides the same ask; with `under` the worded item moves, not the kind's own; a kind with no item gets one added at the end; a set reason offers `Words` and `Split`, `Words` adding the person's words at `Decide` (judged with `under` at the new item's place, decided), `Split` making the lowest free number named "NAME, in part" cut at a character to 40 bytes and keeping the set's items; a one-pattern set, or `matched` all its patterns, offers no `Split`; a full table offers none that would add; a fixed kind, `Always` and a decided reach offer nothing; `narrowed` with words empty or past 120 bytes is refused.
5. **The fuzz test,** `fuzzy::the_judgement_keeps_its_rules`, in the crate's `tests/suite.rs` as `temper-engine-domain-ways/tests/suite.rs` includes its `src/tests/fuzzy.rs`, 64 seeds of random well-formed tables (any items and sets within the bounds, checked by `check` first), random asks (any kind, 0 to 16 paths drawn from the sets' patterns' literal parts and from others, any `under`, `away` and `always`), holding for every ask:
   - a fixed kind is never decided and its class is `Wait`;
   - the class is at least the kind's and every touched set's, `under` applied;
   - decided only with `Decide`, or `Ask` and `away`;
   - with no `Ask` item in the table (`away_may_decide` false), Away on decides exactly what Away off decides;
   - You approve's stock decides nothing; `always` decides nothing;
   - the reason's subject holds the reach's class, and a `Set(n)` reason's set is among `sets`;
   - every table an offer makes passes `check`, and where the reason's subject was the only one at the reach's class, it decides the same ask (`Words` judged with `under` at the item it added).
6. **The bites** (the implementer's text): break the strictest-class rule (take the kind's class alone), see the set rows and the fuzz test fail; break the fixed kinds (let `Money` take its item), see the `check` row and the fuzz test fail; restore each.
7. Runs, counts in the report: `heavy cargo nextest run -p temper-protocol-reaches` (focused) and `heavy cargo nextest run -p temper-protocol-reaches --profile fuzzy`; `heavy cargo clippy -p temper-protocol-reaches --all-targets -- -D warnings`. The fuzz test stays under 2 CPU seconds; report its time.
8. Status: delete the **Not yet** entry naming `ui-reaches/reach-kit` in `temper/docs/design/domain/forge.status.md`.

## Inputs

- `temper/docs/plan/ui-reaches/notes/common.md` and `temper/docs/plan/ui-reaches/notes/contracts.md`, section 1.
- `temper/docs/design/domain/forge.md`, 5.10, all of it.
- `temper/docs/design/draft/what-reaches-you.md`, sections 2 to 5, for the words behind 5.10.
- For the crate's form: `temper/crates/temper-engine-domain-ways` (its `Cargo.toml`, `src/lib.rs`, `tests/suite.rs`, `src/tests/fuzzy.rs`); `tools/lints/design.md`; `skein/docs/foundation/programming-model.md`, 10 (the strict subset).

## Owns and may change

Owns the new crate whole and the status entry. May change the root `Cargo.toml` (its `members` line and its `[workspace.dependencies]` line, each in sorted place) and `Cargo.lock` as cargo writes it; nothing else.

Options, each decided:

- **A name the contract lacks** (a private helper, a test fixture): yours.
- **`Class` ordering:** derive nothing beyond the contract's derives; compare by `strictness`.
- **A random source:** `skein_lib::Rng`, as the ways crate's fuzz test uses it; a disallowed method in the test takes a scoped `#[expect]` as that file does.
- **If a contract signature can't be written in the strict subset** (a `const fn` that needs a `match` on a slice, say): drop the `const` and say so in the report; change nothing else of it.

## Steps

1. Read the Inputs; make the crate (manifest, lints profile, `lib.rs`), then the types and `check`, `stock`, `class_of`, `away_may_decide`, `set`.
2. `matches`, `well_formed`, `PathSet::matched`; their table.
3. `judge` and `Reach::decided`; its table. Then `offers` and `narrowed`; their table.
4. The fuzz test; the bites (acceptance 6).
5. The status entry; the runs of acceptance 7; `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; commit; `python3 -I -B tools/citations/citations.py --base origin/main`; push with `git push origin HEAD:refs/heads/<your branch>`.

## Report

The implementer's report: the surface as written (any `const` dropped), the tables' rows and the fuzz test's invariants, the bites and their failures, the runs' counts and the fuzz test's time.
