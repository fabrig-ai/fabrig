# scale-restart: figures

| Test | Before (Ir) | After (Ir) | Cause | What changed |
|---|---:|---:|---|---|
| `temper-engine-domain-world::suite::fabrig_scale::a_goal_tree_takes_384_tasks` | 17,457,265,311 | 9,362,561,515 | Every saved task rebuilt the tracked projection's whole plan, deeply cloning all retained task words and phases | Update an existing projection task in place; move retained tasks when appending. All 384 tasks, the capacity refusal and the restart oracle stay |
| `temper-engine-domain-world::suite::fabrig_scale::eighty_chats_are_funded_at_once` | 2,922,851,330 | 2,923,111,661 | Eighty durable admissions, cancellation and the restart/read oracle; no tracked projection in this story | Stays: the required eighty funded chats and eighty-first refusal are unchanged; no single avoidable cause reaches 0.2 CPU s |
| `temper-box::lib::tests::restart::an_aborted_serve_restarts_in_every_ending_order` | 2,465,836,366 | 2,465,838,814 | 720 complete simulator lifecycles: startup, six endpoint deliveries, replacement and teardown | Stays: all 720 orders and every assertion; sharing the startup snapshot requires broader support than a small clone |
| `temper-box::lib::tests::restart::early_reply_pipe_failure_and_busy_logs_allow_restarting` | 3,834,834,688 | 3,834,837,463 | The same 720 lifecycles, plus the early failed reply and a held log write | Stays: all 720 orders; additional turns drive the required pipe and log activity |
| `jig-domain-core::lib::tests::effects::saving_a_projection_preserves_neighbours_when_updating_or_appending` | unknown | 4,013,388 | New focused regression; no before test exists | Checks update, append, unchanged saves, emitted durable rows and retention of neighbours' word allocations |

## Measurement and growth

Each count is one exact test through nextest and Callgrind, in the same
unoptimized build on the same builder. The scale profiles use `--profile
fuzzy --ignore-default-filter` on both sides to leave room for instrumentation;
the box and focused regression profiles use the default profile. No setting
or timeout changed. Counts include the test process, not nextest's listing
processes. All profiles passed.

The command, with `--test suite` instead of `--lib` for the scale stories:

```sh
NEXTEST_DOUBLE_SPAWN=0 \
CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER="valgrind --tool=callgrind --callgrind-out-file=$PWD/target/limits-heavy/NAME.%p.cg" \
heavy cargo nextest run -p PACKAGE --lib -E 'test(=FULL::NAME)'
```

The 384-task story loses 46.37% of its instructions, about 1.25 CPU s when
scaled against the request's 2.7 CPU s. `projections::save` alone falls from
8,299,019,885 to 211,326,253 Ir, a 97.45% reduction. The test's `live`, `pool`
and `assignment` helpers were not the cause.

A temporary 192-task comparison used four coordinators with 47, 47, 47 and
46 leaves, and a matching root delegation allowance. It was removed; the
committed scale story is unchanged. Whole-story costs, including setup and
the restart/read oracle:

| Tasks | Before (Ir) | After (Ir) | Before Ir/task | After Ir/task |
|---|---:|---:|---:|---:|
| 192 | 6,183,765,618 | 4,136,144,039 | 32,207,113 | 21,542,417 |
| 384 | 17,457,265,311 | 9,362,561,515 | 45,461,628 | 24,381,671 |

Doubling the tree multiplied projection saving by 3.91 before the fix
(2,122,351,953 to 8,299,019,885 Ir). Afterwards those costs are 78,842,002
and 211,326,253 Ir. The whole story is still superlinear: its whole-tree
feeds grow with the retained tree. The expensive deep copies at every task
save are gone; finding an existing entry and moving the array on append
still walk retained entries.

A restart order costs 3,424,773 Ir before and 3,424,776 after in the basic
story; with both faults, 5,326,159 before and 5,326,163 after. These are totals
divided by 720, including setup and teardown. There is no retained state
between orders, so their cost is linear in the 720 fixed-size lifecycles.
The small changes in unchanged stories are below 0.01%.

The new regression was checked by deliberately replacing the append's move
with a clone. It failed with `task 5 copied neighbour 1`; restoring the move
passed. The bound tests and restart tests were neither renamed nor merged.

## Found and not fixed

- Whole-tree projection feeds: `finish_decision` costs 1,903,543,827 Ir in
  the final 384-task profile, including copying the complete plan required
  by `jig/docs/design/domain/connectors.md`, 8. The extra `ProjectionFeed`
  clone is 755,438,766 Ir and a boxed feed's drop is 425,775,339 Ir. Moving
  the owned feed to the final connector could avoid that copy and drop,
  about 0.18 CPU s against the original profile and request figure: below
  the stop rule. Eliminating whole-tree handoffs would change the feed
  contract. The remaining entry scan and append moves are within the
  211,326,253 Ir save total, about 0.033 CPU s by the same calculation.
- The unchanged restart/read oracle costs 2,463,099,017 Ir in the final
  tree story and 761,158,338 Ir in the eighty-chat story. The tree's
  production store point reads cost 1,465,387,008 Ir, largely task joins
  and decoding; those are inside the oracle total, not additional to it.
  All retained rows and every task/result answer are still compared.
  Avoiding these reads would remove coverage; optimizing joins/codecs
  would need a separate store profile and a change in `jig-protocol-store`.
- Restart setup repeats `probing_serve` 720 times: 797,266,182 Ir in the
  basic before profile and 797,602,028 Ir with both faults. In the basic
  profile shutdown through `run_to_end` adds 700,679,316 Ir. The harness,
  box service, `skein-io::Io`, simulator and fake machine do not provide a
  clonable snapshot. Sharing this prefix would require snapshot support
  across those owners, including the I/O entities, rather than one small
  derive. Construction through `with_stubborn` alone is only 47,079,679
  Ir (1.91%); its turn loop drives startup's real work. The probe's trace
  walk is small. The 720 six-ending deliveries, fault activity and final
  quiescence/descriptor checks stay in each of the four permutation tests.
- The eighty-chat profile has 934,790,604 Ir in store requests and
  1,984,606,025 Ir in its admission helper, with store/engine routing
  nested within it. Its close oracle is included above. These totals
  describe required work, not one removable scan; even the whole store
  request total is about 0.13 CPU s against this row's original 0.4 CPU s.

## Tests that went

None.
