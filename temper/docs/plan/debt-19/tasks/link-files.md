+++
id = "link-files"
part = "link-translate"
kind = "migration"
needs = []
files = ["jig/crates/jig-client-protocol-link/src/translate/documents.rs", "jig/crates/jig-client-protocol-link/src/translate/documents/**", "jig/crates/jig-client-protocol-link/src/translate/documents_part*.rs", "jig/crates/jig-client-protocol-link/src/translate/pages.rs", "jig/crates/jig-client-protocol-link/src/translate/pages/**", "jig/crates/jig-client-protocol-link/src/translate/pages_part*.rs", "jig/crates/jig-client-protocol-link/src/translate/facts.rs", "jig/crates/jig-client-protocol-link/src/translate/facts/**", "jig/crates/jig-client-protocol-link/src/translate/facts_part*.rs", "jig/crates/jig-client-protocol-link/src/translate/accounts.rs", "jig/crates/jig-client-protocol-link/src/tests/fixtures/**"]
effort = "medium"
minutes = 60
+++

# jig: the link's translation is grouped by what it holds, not split by number

## Goal

The first link step of `jig/docs/development/debt/D008-link-translate-split-by-number.md`. 29 numbered files become files named for their record families. Items move; no code changes, no name changes, and no path outside the crate does.

Acceptance:

- The item's **Measure** reads 0: `find jig -name '*_part[0-9]*.rs' | wc -l`.
- `rg -n 'path = "(documents|pages|facts|reads)_part' jig/crates/jig-client-protocol-link/src` finds nothing, nor does `rg -n 'mod part[0-9]' jig/crates/jig-client-protocol-link/src`.
- `heavy cargo nextest run -p jig-client-protocol-link` passes, with the same tests and no test file changed but the fixtures' moves. `heavy cargo check -p jig-client -p jig-client-protocol-http -p jig-protocol-engine-clients -p jig-client-scripted-engine -p jig-conformance-world -p temper-client-tools -p temper-client-domain --tests`.
- The first commit is pure `git mv`, plus the `#[path]` strings' or `mod` lines' change it forces, so `git log --follow` holds. Net production lines change by little (at most +40: `mod` and `use` lines).

## Inputs

- `temper/docs/plan/debt-19/notes/common.md`
- `jig/docs/development/debt/D008-link-translate-split-by-number.md`

## Owns and may change

- **Owns:** the files in `files`. No other file of the crate; `queries.rs` above all, which other plans are changing.
- **`documents_part3.rs` moves whole to `documents/task.rs`, and nothing else joins it.** A plan beside this one (`ui-goal-page/made`) adds a line to its `task_fields`, and at 100% similarity git carries that line through the rename. This is why `task_links.rs` stands apart from `task.rs`.
- **Items stay reachable by the same names:** `documents.rs`, `pages.rs` and `reads.rs` keep their glob re-exports, and `facts.rs` its explicit lists, so `translate::documents::task_fields` and the like hold. `jig-client/src/client.rs`, `temper-client-domain/src/forge/family.rs`, `temper-client-tools/src/recovery.rs` and `temper-client-shell-terminal/src/client.rs` read only the module roots.
- **The map,** children of `translate/documents/`, `translate/pages/`, `translate/facts/`:

  | From | To |
  |---|---|
  | `documents_part0` | `frame`: `frame_party`, `pool_ledger`, `project_role`, `needs`, `count`, `frame`. `chips`: `chip`. `task_links`: `task_status`, `task_link`, `subscription_kind`. `specification`: the rest |
  | `documents_part1` | `task_links`: `subscriptions`, `task_resource`. `specification`: `field_rule`, `verdict_choice`, `task_contract`. `authority`: `grant` through `authority_document`. `results`: `result_value` |
  | `documents_part2` | `offered`: `act_kind` through `offered`. `results`: `spend_line`, `ending`, `result*`, `chip_ending`, `task_result`, `effect`. `task_links`: `live_delegate` |
  | `documents_part3` | `task`: all |
  | `documents_part4` | `lines`: all |
  | `documents_part5` | `hosts`: `store_fill` through `host`. The existing `translate/accounts.rs`: `account_place`, `unread`, `state` |
  | `documents_part6` | `translate/accounts.rs`: `account`. `capacity`: `capacity` through `task_capacity_refusal`, `request_key`. `deployment`: `deployment`. `project`: `project_resource*` |
  | `documents_part7` | `project`: all |
  | `documents_part8` | `project`: `project_fields`. `runs`: the rest |
  | `pages_part0` | `activity`: all |
  | `pages_part1` | `history`: `history_change`, `address_change`, `retained_*`. `activity`: the rest |
  | `pages_part2` | `history`: all |
  | `pages_part3` | `notes`: all |
  | `pages_part4` | `ended`, `inbox`, by name. `spend`: `period_times` |
  | `pages_part5` | `spend`: `funding_record`, `spend_page`. `proposals`: the rest |
  | `pages_part6` | `proposals`: `funding_offer`, `proposal`, `escalation`, `question`. `person`: `shown_result`, `person_task` |
  | `pages_part7` | `decisions`: all |
  | `pages_part8` | `decisions`: the first item. `ended`: the rest |
  | `facts_part0` | `chips`: `phase`, `chip_*`, `closing_stage`. `holds`: `hold_*`, `failure_class`, `retry_notice`. `runs`: `run_*`, `active_run`, `call_identity`. `parties`: `person_address`, `proposer`, `recipient`. `parts`: `document_text`, `executor`, `proposal_key` |
  | `facts_part1` | `chips`, `holds`, by name. `parties`: `actor`, `party*`, `decision`, `person_*`. `wake`: `wake_*`, `results_wake`. `parts`: `part*`, `numbers` |
  | `facts_part2` | `parts`: `budget`, `funder`, `resource_*`, `dependencies`. `parties`: `requester`. `chips`: the rest |
  | `tests/fixtures/reads_part0` to `4` | flat `tests/fixtures/reads_wire.rs`, from `reads_part4`'s first item back |
  | `tests/fixtures/reads_part4` to `7` | flat `tests/fixtures/reads_values.rs`, the `read_value_*`. Flat, since `.config/fixture-readers.toml` globs `fixtures/*.rs` |

  The names are by reading. An item the map misses goes with its family, and a family that comes out too small to stand joins its neighbour. Say so in your report.
- **`executor` is defined in both `facts_part0` and `facts_part1`,** behind `facts.rs`' explicit lists. Keep them in two files if they differ; if they're the same, one goes. Report which.
- **`documents` and `facts` share names** (`chip_fields`, say), which is why they get a directory each: don't flatten them.
- **A move is a move:** no body changes. If a `super::` must be spelled again, spell it.

## Steps

1. Commit 1: every part to its dominant family's file, by `git mv`, one to one, with plain `mod x;` lines replacing `#[path]`. Compile.
2. Commit 2: the items regroup, per the map. Compile.
3. Commit 3: the fixtures, as the map says.
4. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`; `heavy cargo clippy -p jig-client-protocol-link --all-targets -- -D warnings`; the tests above. The citations check: debt reviews under `jig/docs/development/debt/reviews/` cite parts in dated text and may stay if the checker allows it. Commit and push.

No test pins a new rule, so nothing here needs breaking by hand.

## Report

The common note's items, what the map missed, what `executor` was, and the net lines.
