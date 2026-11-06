# model-catalogue-1: what the stock's change reaches

Read by hand for `stock-seven`, 2026-11-06, none of it run. Each row is an edit the task may make, only for this. A
figure here is the reading's: take the test's where they differ, and say so.

New figures, set by Fable 5.1 and Astra 6: C 10,896, 2C 21,792, K 43,584, 5K 217,920, 24K 1,046,016, 80×24K 83,681,280,
81×24K 84,727,296. Alone: Opus 4,359 (K 17,436), Sonnet and Sol 2,180 (8,720), Terra 1,090 (4,360), demo 1,057 (4,228),
Luna 109 (436).

Keep `engine.llm`'s `providers` and `endpoints` as openai, anthropic, demo: many fixtures read `[0]` as OpenAI's. Only
`models` takes the table's order. `demo` declares no efforts and keeps 1/1/1 per 1,000. `--model` takes a model's
default effort since `ctl-offer`, so no walk needs `--effort`.

## `temper-init`

| Site | Edit |
|---|---|
| `template/{temper,bare}.json`, `template/{agent,bare-agent}.json` | section 13.4's table; `effort` and `shown` in the first two alone, since smith's agent denies unknown members; keep each model's exact spacing, as `temper-service`'s `tests/fixtures.rs` string-replaces `"output": 8192`, `"tool_payload": 10662` and `"transport": "tls"` |
| `template/temper.json`, the scripted delegate's `spend` and `tokens` | 21,140 → 217,920; 269 → 270: left alone it holds the coder (`jig-domain-authority/src/check.rs`, a budget at or below `minimum_run_spend` waits) |
| `src/tests/documents.rs`, `"spend": 21_140` | `217_920` |
| `src/tests/documents.rs`, `openai["models"]`; the `[sol, luna(sol), opus, demo]` built from the owned example; agent's `sol, luna(sol)`; `anthropic`, `anthropic_agent`; `("models", 2)` | an explicit table of the seven, with `demo_model()`; three Anthropic names and `models[0..3]`. The owned example stays as it is, so `the_default_is_the_owned_example_so_edited` is no longer a light edit |

`bare.json` isn't rewritten by the writer below: `the_bare_document_is_the_box_less_its_parts` holds it.

## `temper-service`, `src/tests/`

| Site | Edit |
|---|---|
| `first_model.rs`, the `models[0]` assertion and its message | `claude-opus-5-5` |
| `first_model.rs`, no account keeps `OPENAI` | `CLAUDE` |
| `stock.rs`, `single_model` | it takes `[0]` of each table, now Opus beside `openai`'s endpoint, so about 15 tests fail: take `engine.llm.models[3]` |
| `stock.rs`, "needs 3 alternatives" | 7 |
| `stock.rs`, the orders' table | generated row `[opus, sol, sonnet, fable, astra, terra, luna, demo]`, charter 3 `[opus, astra, sol, sonnet, fable, terra, luna, demo]`; a row for a document naming none |
| `stock.rs`, what moves C | swapping `models[0]` no longer does: init's row 10,896/43,584; a cheap row replaces the whole list, 7/256; add cheap first with Fable later |
| `stock.rs`, `2*1057, 4228, 24*4228` | `2*10_896, 43_584, 24*43_584` |
| `stock_brief.rs`, host-part rows | charters 1 and 2 render 110 bytes of names, +69: `(1,false,42_183,357_952)`, `(1,true,40_093,360_042)`, `(2,false,35_083,365_052)`; the test eprints `host-part row:`, so copy from there |
| `stock_brief.rs`, "Four model declarations", `MARGIN`, `room` | eight, 210, `357_952`. `brief_bytes` 356,270 holds |

`stock.rs`'s policy 5,774 (`size_of`), `run_bytes` (derived), `checks.rs` and `root_checks.rs` hold.

## `temper`

| Site | Edit |
|---|---|
| `tests/support/first_repository.rs`, `run_spend` | `truncate(1)` now fails to parse. Read `template/bare.json` with no truncation, or retain all but `demo`; `run_spend(5)` is 262,144. No single-model cut: Sol's cap of 8,720 leaves the second run short |
| `tests/support/forge_land.rs`, `cheapest_model_first` | moving a model first does nothing under named first choices: retain `gpt-6-luna` and `demo`, the OpenAI provider's models `[luna]`, no Anthropic provider, endpoint or model. The agent may keep extras (`startup/checks.rs` needs charters' models a subset). Rename it. **Live, so read, not run** |
| `tests/support/forge_land.rs`, `configure_live_model` | strip `efforts` and `effort` from both clones of Sol into Haiku, or they reach the real `claude`. **Live** |
| `tests/suite/forge_land.rs`, the expected prices | `2000/2000/10000/1000000`, or set Haiku's in the fixture |
| `tests/support/forge_land/rung.rs`, the `models[0]` rename | it would rename Opus: delete it, find Sol by name; drop the efforts override, assert `effort` is `high`; retain Sol and demo, so C 2,418, K 9,672 |
| `tests/support/forge_land/rung.rs`, the pinned budgets | 104K 1,005,888; 96K 928,512; 24K 232,128; 1,920K 18,570,240. Better, read K from `rules.maximum_run_spend`. **Live** |
| `src/tests/help.rs`'s `gpt-6-astra` | the name of `temper/docs/help/deployment.md`'s example, below |

## `temper-fake`, and temper's engine world

| Site | Edit |
|---|---|
| `temper-fake/src/backends.rs`, `selected` on `b"endpoint", b"anthropic"` | three matches now, "target must be unique", and the fake box won't build: set the first to fake-claude, remove the others; `remove` gains an array element's form, `EDITS` +2. Its order is then `[fake-claude, sol, astra, terra, luna, demo]` |
| `temper-fake/src/backends/tests.rs`, every charter's main is Sol | charter 3's is `gpt-6-astra` |
| `temper-fake/src/backends/tests.rs`, no `claude-opus-5-5` | add Sonnet and Fable |
| `temper/tests/engine/domain/tests/fixtures/stock.rs` | it reads `models[0]` beside `endpoints[0]`'s dialect: find `gpt-6.1-sol` by name |
| `temper/tests/engine/domain/tests/suite/fabrig_scale.rs`, `settings/b2.rs`, `K = 4228` | `8_720` with Sol's arithmetic, or read it from `stock.rules.maximum_run_spend` |
| `temper/tests/engine/domain/tests/suite/host_calls.rs`, a reference `spend` of 4,228 under a replaced model | `charter.spend = defaults.charters[0].spend`, or it faults on budget |

The rest of those worlds, `temper-playground`, `temper/tests/real-loop`, `temper/tools/stage-one` and
`jig/testing/jig-test/tests/support/live_sign_in.rs` hold their own documents, and need nothing.

## Docs, and the generated reference

- `temper/crates/temper-service/src/startup/members.rs` names a model's members: `efforts` ("init leaves it out"), and
  `effort`, `shown` and `picker`, new. Then `temper/docs/help/reference/configuration-engine.md`, by its tool.
- `temper/docs/help/deployment.md`, **Offering another model**, and `temper/docs/help/account.md`'s line: `gpt-6-astra` is now
  stock, so appending it is refused as `What::Unique`. Take `claude-haiku-5-5`, under the endpoint and provider
  `anthropic`, with `efforts` low to high, `"effort": "high"` and that "the engine's first effort is the default" replaced
  by "`effort` names the default; left out, it is the first". `src/tests/help.rs` follows.

## The live and process tiers

The queue's gate runs neither. Live, read by hand above, with nothing else moving: `temper/crates/temper/tests/live.rs`
and `proof_live.rs` (through `forge_land`, `rung`, `support/live.rs` on the owned document), `live_github*.rs` (their own
documents). Process, to run, in `temper`'s `--test process`:

```sh
heavy cargo nextest run --profile process-temper -p temper --test process -E 'test(=a_csv_change_passes_ci_and_a_blocking_agent_review_on_the_owned_forge_then_lands_and_survives_restart)'
heavy cargo nextest run --profile process-temper -p temper --test process -E 'test(=a_persons_plan_change_lands_on_the_owned_forge_and_is_found_after_a_restart)'
heavy cargo nextest run --profile process-temper -p temper --test process -E 'test(=playground_setup_adopts_widget_and_the_worker_reads_its_source_on_the_owned_forge)'
```

The last also reaches `run_spend`. The tier builds `fake-claude` for `TEMPER_TEST_CLAUDE` (`tools/tiers/packages.toml`).

## The largest turn

T, by reading, doesn't move: effort names are charged at the charter codec's fixed 32 bytes, names at a fixed 128, `room`
reads window and output, and `run_bytes` is fixed. So T stays 16,803,810, the host part 400,135, room 822,476 and offer
200,068, with window 1,048,576, output 8,192 and `reasoning_item` 65,536 on every new model. `tool_payload` 10,662 and
what follows it stay too. Check it, no write:

```sh
heavy cargo nextest run --locked -p temper --test suite -E 'test(startup::turn_bounds)'
```

If it fails, read why; its writer is `SKEIN_CODEGEN_WRITE=1` on the same command, covering `template/temper.json` and the
owned, cluster and GitHub fixtures, with `bare.json` by hand. Say the figure in your report either way.
