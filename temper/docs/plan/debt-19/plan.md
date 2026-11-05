# debt-19: the nineteenth debt round

Seven independent items (`inputs/request.md`): skein D012, D014, D032; jig D008, D009; smith D011, D015. All seven are open at this plan's base; every **Measure** was re-read, and where
it has moved the part says so. No temper client view and no `temper.css`.

## Parts and tasks

| Source | Part | Task | Needs | Effort | Min | What it does |
|---|---|---|---|---|---:|---|
| smith D015 | `smith-paths` | `paths-bench` | | medium | 30 | the benchmarks' shared test code moves behind `fixture`; their `#[path]`s go |
| smith D015 | `smith-paths` | `paths-rest` | `paths-bench` | medium | 60 | 40 more go; six stay, each with its reason; the item goes |
| smith D011 | `bench-fault` | `bench-fault` | `paths-bench` | high | 150 | one `Fault`; no string errors outside the edges; the item goes |
| skein D012 | `llm-dialects` | `llm-mirrors` | | high | 30 | `Stop`, `Usage`, `Failure` once; three mappings go |
| skein D012 | `llm-dialects` | `llm-wire` | `llm-mirrors` | high | 50 | a private `wire`; Anthropic names nothing of `openai` |
| skein D012 | `llm-dialects` | `llm-output` | `llm-wire` | medium | 25 | one `wire::Output<P>`; the item goes |
| skein D014 | `setup-vocabulary` | `setup-plan` | | high | 40 | `plan.rs`; `plan::stop` once |
| skein D014 | `setup-vocabulary` | `setup-forgejo` | `setup-plan` | medium | 25 | `forgejo/` beside `github/`, a move |
| skein D014 | `setup-vocabulary` | `setup-replies` | `setup-forgejo` | high | 100 | a `Reply` per dialect behind a gate; `reachable` becomes two plans |
| skein D014 | `setup-vocabulary` | `setup-admin` | `setup-replies` | high | 60 | the admin step once; the item goes |
| skein D032 | `io-worlds` | `io-world-loop` | | high | 75 | `Referee::idle`; socket and either worlds on `skein_world` |
| skein D032 | `io-worlds` | `io-output-loop` | `io-world-loop` | medium | 25 | the output world likewise; the **Measure** reads 0 |
| skein D032 | `io-worlds` | `io-on-stage` | `io-output-loop` | high | 70 | all three on the stage; the check moves into it; the item goes |
| jig D009 | `limits-once` | `limits-core` | | high | 70 | the children's copies go; the core passes the values |
| jig D009 | `limits-once` | `limits-link` | | high | 40 | one `narrow` per class; the **Measure** reads 0 |
| jig D009 | `limits-once` | `limits-close` | `limits-core`, `limits-link` | medium | 10 | what's left goes to D007; the item goes |
| jig D008 | `link-translate` | `piece-within` | | medium | 20 | skein's `split_within`, `count`, `counted` |
| jig D008 | `link-translate` | `link-files` | | medium | 60 | 29 numbered files become files by record family |
| jig D008 | `link-translate` | `link-loops` | `piece-within`, `link-files` | medium | 50 | the link's 42 loops read one helper |
| jig D008 | `link-translate` | `piece-others` | `piece-within` | medium | 25 | the other crates' loops and wrappers |
| jig D008 | `link-translate` | `link-encoders` | `link-loops`, `piece-others` | high | 60 | 39 test-only encoders move under tests; the item goes |

## Critical path

`setup-plan`, `setup-forgejo`, `setup-replies`, `setup-admin`, 225 minutes. `paths-bench`, `bench-fault`, 180. `io-world-loop`, `io-output-loop`, `io-on-stage`, 170. `link-files`, `link-loops`, `link-encoders`, 170. `llm-mirrors`, `llm-wire`, `llm-output`, 105. None of the seven parts waits on another.

## Production code, item by item

The request asks each item to end with no more production code than it started, and to say so if it can't.

| Item | Net production (estimate) |
|---|---|
| skein D012 | about -45 |
| skein D014 | about -60 |
| **skein D032** | **about +14: it can't fall.** Below |
| jig D008 | about -800, with +24 in skein |
| jig D009 | about -15 |
| smith D011 | about -150 |
| smith D015 | about 0 |

## Overlaps with the drafts beside this one

- **`ui-goal-page/made`** adds one line to `documents_part3.rs`'s `task_fields`. `link-files` moves that file whole to `documents/task.rs` and lets nothing else join it, so git carries the line through the rename at 100% similarity. Its own first commit is a pure move. If a merge still conflicts, the cost is one line.
- **`ui-goal-page/made`** and **`ui-goals/delegate-counts`** touch `translate/queries.rs`, which no task here changes, and `jig-protocol-engine-clients/src/translate/{task,queries}.rs` and `jig-protocol-store/src/codec/tasks/record.rs`, which `piece-others` stays off.
- **`ui-goals`** adds a line to `jig-domain-tasks/src/domain.rs`, another hunk from `limits-core`'s.
- **`ui-settings/ways-kind`, `setup-intent`** touch `temper-engine-domain-forge` and `temper/tests/engine/**`. `limits-core` edits three test fixtures there, a literal's field each, compiler-forced.
- **`ui-settings/ways-ctl`** touches `temper-settings-form`, which D009 leaves alone, and says so in D007.
- No task touches a temper client crate. `jig-client-protocol-link`'s readers in temper compile unchanged, since every move there hides behind its module roots.

One link file has two writers at once by design: `limits-link` owns `src/limits.rs` and `tests/attempts.rs`, and `link-files` owns `translate/` and `tests/fixtures/`. Nothing crosses between them, so neither waits, which departs from one writer per crate for that reason alone.

## Decisions: D015

Re-measured: 57 attributes, as the item says.

- **Six stay, with their reason, and that's the item's second arm.** Five are the schema compiler's output (`generated/*.rs`), declared the same way in skein and jig, and one is `transcript-patches`' `patches/`, whose layout `skein/docs/design/codec.md`, 11 fixes. Hiding them behind `mod generated { .. }` would spell a convention differently in one tree for a count.
- **Two owners get a `fixture` feature:** `smith-bench` (`request`, `json`, `events`) and `smith-agent-shell` (`product_document`). Both are read by the crate's inline tests and by its own suite, so a world crate can't serve them. `smith-agent-process-world` takes the live probe, since it is that world's.
- **A module keeps its name,** so no test changes name: `grant_tests` moves to `grant_tests.rs` and `program_world` to `program_world.rs`, not the other way round.
- **`paths-bench` goes first and alone,** since `bench-fault` rewrites the same crate.

## Decisions: D011

Re-measured: the item's `Measure` reads 205 now, and it under-counts: `Result<[^>]*, String>` misses `Result<Vec<u8>, String>`. The wider search reads 246. The task checks the wider one.

- **The kinds are chosen in `notes/contracts.md`, 1,** so the task has none to choose.
- **The kind reaches no stored file.** `End::HarnessError { what }` and `End::Refused { setup }` keep their strings, and `Display` is the detail alone. The item says the format changes only if a kind reaches the summary; a stored format would be a stop-and-ask, so this round doesn't. A kind in the summary is left for whoever wants it.
- **`Setup` and `HarnessFault` go; `Refusal` stays:** its `file` and `key` are read by callers and tests, and it is what a document's reader returns, so it reaches a `Fault` by `From`. That's still one fault type in the harness.
- **No `From<String>`,** or the kinds would be decoration.

## Decisions: D012

Re-measured: 35, as the item says. The **Measure** under-counts: it misses `use crate::openai` and the bare `json::` lines that resolve through it, so `llm-wire` checks `\bopenai\b` too. Anthropic's test files keep their 9 lines of `openai::`, through re-exports, since the request says tests stay the same.

- **`Output`: the cut decided one generic data type, `wire::Output<P>`,** with `pub type Output = wire::Output<Part>` in each dialect. The two `Output`s are the same text, but each wraps its own `Part`, which genuinely differ. Three ways out, and each breaks something:
  - a generic, chosen: it departs from `programming-model.md`, 10.3 ("no generic items"), for a reason written in `skein/docs/design/llm.md`, 2: a data type with no methods hides no call, which is what the rule guards. `AGENTS.md`, 1 allows a departure with its reason written. Paths, tests and sizes don't change;
  - one superset `Part`, refused: it rewrites six Anthropic test assertions, which the request rules out, and gives Anthropic an `id` and a `phase` it never has;
  - two `Output`s, one neutral enum nested in each, refused: it changes every `Output::TextDelta` in 54 test lines and a public shape.

  **fabrig/debt can overrule this:** if the departure isn't wanted, `llm-output` is dropped and the item stays open on `Output` alone, reworded, since the other three words are paid by then.
- **`dialect::Output` stays, and its two mappers:** its `Part` is a translated block or an error and its failure carries a report, so it isn't a copy. Their identity arms for `TextDelta` and the like stay, since a function over both dialects would be a second generic item. **Paid when**'s "no identity mapping between copies" is met for `Stop`, `Usage` and `Failure`, its plain meaning.
- **`anthropic::decoded_bound` keeps its value.** It sums OpenAI's bound (including `List<Input>`, which Anthropic doesn't use) with its own. Dropping those terms would pay the coupling properly, but it changes what a public function returns, for a refactor. So its body moves to `dialect.rs`, which knows both, and `anthropic` re-exports it. Its one outside reader, `skein-fake-llm-protocol`, only asks whether it's `Some`: a later item could shrink it safely.
- **One test goes,** `translate.rs`'s test of the three mappings, since nothing's left to map. `llm-mirrors` names it.

## Decisions: D014

Re-measured: `GithubRows(` is 64, not 61, and only 27 are other-dialect rejections: 10 are GitHub's own or-patterns, 2 real matches, 3 definitions and 22 tests. So a split alone doesn't reach 0; the rename does the rest, and `setup-replies` checks the rule directly, since a rename alone would also read 0.

- **`plan::Reply` stays one flat enum at the host's boundary, and each dialect's plans see their own, behind a gate.** A `Reply` and an `Event` per dialect would make three pinned tests unrepresentable (`Unsigned` to a Forgejo plan is `KeyForm`, and so on): their assertions would go, a stop-and-ask. Nesting it (`Reply::Forgejo(..)`) breaks three sites in `temper-fake` for nothing. **Paid when** asks what plans see, and that's what changes. Written in `skein/docs/design/forge-setup.md`, 4.2.
- **`reachable` becomes two plans,** being the one that speaks both.
- **Public paths all stay, by re-export,** as `skein/docs/design/forge.md`, 4.4 does for a dialect: temper spells `reconcile::Why` alone 75 times. `Reachable::github` is the one `pub` that goes, and it has no outside reader.
- **`stop` is one function with thin wrappers,** not one method: each plan clears different fields, and sharing that would take a closure or a trait.
- **D037 isn't paid:** it has five other clauses. The rename empties its forge-setup `Github*` bullet, and `setup-replies` removes that bullet.

## Decisions: D032

Re-measured: 3. **The **Measure** reads 0 after `io-output-loop`, and **Paid when** isn't met until `io-on-stage`.** The item calls the stage "a decision, not a refactor". The cut took it, and pays the item, since debt-18 already left it open once.

- **The check moves into the stage, as an `assert!`.** A world can't see a stage's entries: no hook, no trait object, and `Queue` is concrete. The item's three ways: a witness (a trait and `impl Trait`, both out of step code, about +22 shipped lines with one reader), a test-only loop (not on the stage in any honest sense), or a panic path in shipped code. The third is what `skein/docs/foundation/programming-model.md`, 10.2 allows by name, "the invariant relied on", and the stage does rely on it: `room` is only sound if each entry submits no more than it declares. So every service gets the check too. `debug_assert!` is banned by clippy. **It adds about 14 production lines and one panic path, which fires only if io is wrong.** That's why this item can't end lower. **fabrig/debt can overrule this:** without `io-on-stage`, the item stays open on its stage clause, with its **Measure** at 0.
- **The ledger needs no shipped change.** `Ledger::asked` records a description by reference as a request is queued, and `Ledger::taken(count)` checks what the stage took. Recording at queueing alone would be unsound: a held-back `Close` sets `closing`, and later legitimate bytes trip it.
- **By reading, no schedule moves, and a digest decides.** The stage's three differences from the hand loops look unreachable at these limits (`io-on-stage` has the reasoning). If a digest differs, the task stops and asks. It isn't the cut's to retime anything. The digest is throwaway code, specified in `notes/contracts.md`, 2, so that it's no tool beside the work.

## Decisions: D009

Re-measured: 15. D007 is open, so bound classes in the schema aren't there to generate from; generating them here would mean changing skein's grammar and marking 296 fields in a schema two drafts are editing.

- **Capping stays. This answers the item's open question.** The backfill recommended refusing "like every other bound", but these knobs aren't single bounds: `rows` sizes 72 limits, and the schema bounds five of them below 32. Every shipped client sets `rows` at 16 or more, and refusing would refuse all of them. So: a knob past its *class's* ceiling is refused, as it is today, and a member the schema bounds lower keeps its ceiling. Written in `jig/docs/design/client/domain/client.md`, 14, and the status's departure goes with it.
- **`narrow` by class isn't a rename of `.min`.** One rule per class, every member through it; a field decides nothing for itself, and no `= w::CEILINGS.x` no-ops remain. A helper over the 15 alone would only have dodged the measure. **Membership is still listed, once per class,** which only generation removes: `limits-close` records that in D007, so **Paid when** holds for the rule, and D007 keeps the list.
- **The children take the values at construction, since they can't see up.** The opposite direction leaves the fleet's copy and contradicts `jig/docs/design/domain/engine.md`, 17.8. Three equality tests go, because the state can't be written any more: `limits-core` names them. This isn't a stored or wire format, nor a configuration name, and the compiler forces every reader, so the cut doesn't read it as a breaking public interface.
- **The six client `Limits` types stay:** one per concept. The review's duplicate is the expression `body_bytes + 512`, whose callers are in temper shells other drafts are changing, so it's left. `Levels` isn't a bound. `codec::View` is already derived once. `temper-settings-form`'s 36 `.min(CEILINGS` and the engine's 22 are outside the item's **Where**: `limits-close` records them in D007.

## Decisions: D008

Re-measured: 29 files, and 45 loops (the item says 44). Its "85 test-only encoders" is wrong: there are 102 such functions, 63 reached from production and 39 by tests alone (426 lines). `link-encoders` lists them.

- **The helper is non-generic, in `skein_codec::piece`:** `split_within`, `count`, `counted`. The item asks for "a codec piece reader". A whole-loop generic would need a closure or a trait, and decoders are inherent generated methods. So **"one helper" holds for split, charge and bound, and the `for` stays at each site with its row's own decoding.** A generator could take the rest, and that's D007's.
- **`to_bytes` is left out:** **Paid when** doesn't ask for it, and it means changing the generator and regenerating 26 files.
- **The encoders go under `src/tests/`,** as the item says, not behind a feature: jig's crates have no `fixture` features, and only the link's tests and conformance read them. The scripted engine has its own copies; conformance's 7 names read those, and that's truer to what conformance is for. The link's tests keep their copy, which `link-encoders` reports on, since removing it means a dev-dependency cycle this round has no need of.
- **`piece-others` is small, and is in** because the item's count names those loops too. A site with its own errors stays and says why.
- **The dead `!input.is_empty()` checks are left:** not this item's.

## No user questions

Nothing here is the user's by the lab's list. Three calls are the cut's, and fabrig/debt, the plan's owner, can overrule them: the generic `Output`, the assert in the stage, and capping by class.
