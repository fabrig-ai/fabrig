# limits-fuzzy: what `limits-worlds` left on the real loop

Copied from that plan's closing figures, whose directory leaves main when it's retired. All at one combined trunk, unoptimized, under raised patience. `R` is `fuzzy::repair::a_change_failing_ci_is_repaired_and_lands`, whose two profiles stopped partial at the fake forge's observation cap; `P` is `fuzzy::case_6_4::a_red_tip_pauses_kits_queue_for_one_repair_that_lands_before_a4_while_the_directors_interim_timer_waits_without_adopting_a4`; `G` is `github_land::a_chats_change_lands_on_connected_github`, focused. `U` is the one-process world's `public::every_inaccessible_identifier_is_one_unknown_byte_for_byte`, focused.

| Story | Ir, run 1 / run 2 | Native CPU s |
|---|---:|---:|
| R | 23,471,805,382 / 22,906,059,880, both partial | 7.35 |
| P | 21,430,390,341 / 21,249,528,979 | 5.43 |
| G | 11,561,761,382 / 11,595,415,230 | 2.64 |

The real-loop world's fuzzy suite is 18 tests at about 57 CPU s, so 2 CPU s out of the suite is about 3.5% of that world.

These shares are footprints. They overlap their callers, and none is a promised saving.

| Cause | Share, closing profiles | Why it stood |
|---|---|---|
| `skein-json`'s parser | R 9.36 / 8.83%; P 7.19 / 7.25%; G 6.68 / 6.66%; U 8.74% | Input validation stays. Not parsing a schema again needed a change of the producer's representation, or retained schema state keyed by charter, namespace and grants, which that plan judged past its local repair |
| Repeated whole-prompt measurement, `smith-protocol-llm::component::observed_prompt` and `skein_llm::client::measure` | R 7.35 / 6.20%; P 3.98 / 2.03%; G 3.70 / 4.26%; U 5.80%, at that plan's middle | Refusing before observing needs the admitted measurement handed across the prompt-observation and connection-reservation boundaries, a protocol change that plan left |
| String measurement, `skein_json`'s `escaped_len` | R 2.03 / 1.91%; P 1.28 / 1.29%; G 1.19 / 1.19%; U 2.69% | A trial of one validating, escaping pass saved 0.76% of U and was discarded. Trunk has since taken plain runs and dropped a duplicate measurement |
| Byte searches, `skein_lib::bytes` | G's terminal 1.81 / 2.54%; P's cues 1.03 / 1.04% | New-page checks need the search; the whole footprint can't go |
| `Iter::next` and `Enumerate::next`, self | R 2.38 + 1.71%; P 2.58 + 1.48%; G 2.37 + 1.77% | Not traced to their callers. `skein_lib::bytes::find_from` has one such loop, for a one-byte needle |
| `core`'s checks on copies, self: `is_aligned_to` + `copy_nonoverlapping::precondition_check` | R 2.91 + 2.21%; P 2.55 + 1.82%; G 2.53 + 1.74%. In `temper-conformance-world`'s two crash sweeps, with `new_unchecked::precondition_check` and `maybe_is_nonoverlapping`, about 14% | Not traced to their callers. They run per copy, so what could go is a copy, never the check |
| The policy-deadline scan, `temper_service::system::Service::next_policy_deadline` | P 9.5% raw, about 0.42% once scaled for callgrind's slowdown | A screen, not a saving: it mixes io readiness and timer wakes |
| Allocation's header layout, `skein_heap` | `temper-conformance-world`'s sweeps about 2.5% | Required by the counted allocator, and stays |
