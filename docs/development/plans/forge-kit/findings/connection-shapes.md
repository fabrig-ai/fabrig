# Connection-shape assertion mapping

Frozen skein b481ad98db8c1460a0ae7dce6143729dc523c075; hourly07-followups scratch specimen, not a landed or package-validated correction. The19 assertion sites map1:1;83 explicit evaluations retain the same input rows. Readiness must check the mapping against its frozen source before treating it as proof.

| Original function | Assertion ordinal | Corrected function | Assertion ordinal |
|---|---|---|---|
| `an_unconfigured_forge_is_unavailable_at_zero_cost` | 1 | `an_unconfigured_forge_is_unavailable_at_zero_cost` | 1 |
| `an_unconfigured_forge_is_unavailable_at_zero_cost` | 2 | `an_unconfigured_forge_is_unavailable_at_zero_cost` | 2 |
| `a_stopped_forge_does_not_stop_another` | 1 | `a_stopped_forge_does_not_stop_another` | 1 |
| `a_stopped_forge_does_not_stop_another` | 2 | `a_stopped_forge_does_not_stop_another` | 2 |
| `a_stopped_forge_does_not_stop_another` | 3 | `a_stopped_forge_does_not_stop_another` | 3 |
| `a_stopped_forge_does_not_stop_another` | 4 | `a_stopped_forge_does_not_stop_another` | 4 |
| `a_stopped_forge_does_not_stop_another` | 5 | `a_stopped_forge_does_not_stop_another` | 5 |
| `an_idle_connection_is_reassigned_to_another_forge` | 1 | `an_idle_connection_is_reassigned_to_another_forge` | 1 |
| `an_idle_connection_is_reassigned_to_another_forge` | 2 | `an_idle_connection_is_reassigned_to_another_forge` | 2 |
| `an_idle_connection_is_reassigned_to_another_forge` | 3 | `an_idle_connection_is_reassigned_to_another_forge` | 3 |
| `an_idle_connection_is_reassigned_to_another_forge` | 4 | `an_idle_connection_is_reassigned_to_another_forge` | 4 |
| `start_backoff_doubles_to_its_maximum` | 1 | `start_backoff_doubles_to_its_maximum` | 1 |
| `start_backoff_doubles_to_its_maximum` | 2 | `start_backoff_doubles_to_its_maximum` | 2 |
| `start_backoff_doubles_to_its_maximum` | 3 | `start_backoff_doubles_to_its_maximum` | 3 |
| `start_backoff_doubles_to_its_maximum` | 4 | `terminal_start_states_ignore_every_start_answer` | 1 |
| `each_listing_uses_its_largest_answer_row` | 1 | `each_listing_uses_its_largest_answer_row` | 1 |
| `each_listing_uses_its_largest_answer_row` | 2 | `each_listing_uses_its_largest_answer_row` | 2 |
| `each_listing_uses_its_largest_answer_row` | 3 | `each_listing_uses_its_largest_answer_row` | 3 |
| `limits_check_admits_one_largest_row_of_every_listing` | 1 | `limits_check_admits_one_largest_row_of_every_listing` | 1 |

Evaluation inventory:2 unknown-forge +5 stopped-forge +4 eviction +(5×3 progression +2×3 terminal) +7×(2 oracle/bound +5 page rows) +2 admission =83. Shared harness invariant assertions are unchanged and excluded from this count.

Five backoff rows remain1,2,4,8,8 seconds; the terminal matrix starts at23s with Ready/Stopped and three Heard outcomes. No expected wire bytes, IDs, admission inputs or original assertion case may change. Named capacities replace only the sampled size/count expressions. The two unrelated new tests and inherited fixture/test prefixes remain unchanged.

Counts: six original new identities become seven; connection N→N+1,128→129 at this snapshot. Temper still has112 old component identities; its consumer/deletion ownership and−107 delta do not inherit skein’s new identities. Historical128 forecasts/proofs remain historical.

Checker evidence:5 positive/decoy and9 negative controls distinguish arithmetic/string/comment/renamed-receiver decoys; original exactly3 violations. Recovery and plan-shapes controls passed. Final package fixture edits were source-only after three attempts; clippy/default/fuzzy/browser remain unchecked. Retained full expressions, source reconstructions, patches, commands and logs: tmp/temper/forge-kit/scratch-hourly07-followups/{assertion-map.json,assertion-audit.txt,commands-and-results.md}.

Refresh any further reached assertion/count copy and list it in the commit body. Preserve controls/modes that have landed by implementation; do not require unlanded listing mode.

## Ready check, 2026-10-19T13:20Z

Independent frozen source: skein9fbda6b7076a20b5dcf387bbbc47453a3cea84a0; smith9d4430e956b558571687642291769afd884e520b; jig39c1e4db9dc923b0a2063468fa5533fa3c50113a; hearth61ab3d16477c4dc97b10d4b7dc2ac6e0cddf7ef0; temper79eff1cddcfe45e9584415c5338d8449cd73904e. Source-verified mapping19/83 and inherited prefixes pass; final fixture declarations compiled independently. One  attempt,20261019T131645-824.log,exit0,wait0s/run15s.

Format and checker/package all-target clippy with−Dwarnings pass. Connection5positive/decoy+9negative controls pass; original exactly3diagnostics; recovery6+/7-/2decoys and plan-shapes6+/6- pass. Listing mode absent at this source, so no future API required. Sequential default129PASS/0.102s,fuzzy0,browser0,empty profiles accepted with--no-tests=pass; page/backoff/terminal/eviction4/3/3/13ms. Test16lines, largest checker function39lines, files543/778/126/157; no budget/ceiling relaxation.

Artifacts: tmp/temper/forge-kit/scratch-ready-skein-03s1/{plan-name-audit.md,owners.json,launch-1.sh,attempt-1.sh,commands-results-timings.txt,attempt-1.full.log}; worker ended and own target removed. No full gate/process/recording/live/downstream build or future implementation proof. Preserve/recheck newly landed shared checker modes before implementation. The earlier preparatory package failure remains historical, not a claimed pass.
