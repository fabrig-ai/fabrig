# Retrospective: accounts

2026-10-20.

## 2. Outcome

`done.md`'s "Done when, run 2026-10-20T00:55:14Z" holds each command's output. After the last landing, the supervisor's Sonnet sub-agents ran the commands in worktrees of each repository's `origin/main`.

| Item | Result |
|---|---|
| 1. Every step landed or dropped | **Met.** "56 of 56 steps landed; 0 running, 0 blocked, 0 ready, 0 to check, 0 waiting." |
| 2. Each session's own "Done when" | **Met after ruling 151.** Three commands failed as stale while their properties held: skein 01's `access_token` grep found skein 11.1's `testing/skein-fake-peers/tests/oauth.rs`; skein 07's count read `machine.rs`, but its check moved to `machine/fetch.rs:197` under ruling 118; smith 03's `Refusal::(Endpoint\|Dialect)` pattern matched `connection::Refusal::Endpoint`, an unrelated enum, at `crates/smith-protocol-llm/src/component.rs:553`. The rest passed, skein 01's and 02's `scripts/check.sh` included. |
| 3. The licence | **Met** in skein and smith. jig got the same licence at `c2282f63` (01:27 on 10-20), outside the plan. |
| 4. The test-time aims | **skein missed; smith met on a re-measure** (ruling 152). skein's medians of package sums were 20.83 s focused against 16, and 21.06 s fuzzy against 19. The runs took 21.07, 20.83 and 15.53 s, and 21.06, 21.09 and 16.31 s, at a one-minute load of 7.4 to 8.8 on 8 cores, just after an owners gate; the third met both aims. In smith, the command's logs, written into the worktree, made `smith-bench`'s probe test refuse a dirty tree, so each focused run stopped at 18 to 20 of 2,453 tests; the loop still exited 0, and fuzzy took 93.64 s against 79. The re-measure, with logs outside the worktree at load 12.67, gave 59.81 s of 60 and 78.17 s of 79. The first run of each pair was about 50% slower than the other two. |
| 5. No budget raised | **Met.** 15 s and 60 s in both. |
| 6. Nothing else checked | Nothing to run. |

The release came 32 minutes after the last landing. "Done when" and its two rulings took 29 of them.

What the plan leaves for later is in `later.md`'s 15 rows, 5 of them added during the run: rulings 116, 145, 146 to 150 (one row), 150 and 152. "For the user" E (ruling 152) has no answer yet.

## 5. What did not

7. **jig-repin.s1 spent 99 minutes on a flake.**
   - **What happened.**
     - jig's browser story `under_a_prefix_the_inbox_s_links_carry_the_prefix` failed at person step 9, waiting for "Cancelled", in three of four gates.
     - Rulings 135 and 139 gave budgets of 45 and 60 minutes, with 30 focused runs. All of them passed except one at the base pins, which also fail it "if less often seen".
     - It landed under the README's known-flake rule at 20:31.
     - Its `later.md` row, owed since 19:31, came with ruling 145 at 21:14.
   - **Cause:** a race in jig's client, older than the run.
   - **First visible:** 18:52. **Acted on:** 19:03 and 19:31.
   - No proposal. The race is section 9, jig item 1. The known-flake rule ends with this README.
8. **The 15 s budget failed 7 of the 14 failed land gates.**
   - **What happened.**
     - smith's gates timed out four times (rows 160, 433, 494 and 511), each with emacs at about 100% CPU when the gate started; hearth's three times.
     - smith's passing gates took 11.8 to 14.5 s, a median of 13.2, against 15. Five workspace runs in the build lane also met the timeout.
     - At the base, smith's suite took 13.1 s, 87% of its budget. The planner's note for review warned of it at 10 s.
   - **Cause.** Contention, not growth: smith's time does not grow with its test count. Its slowest test, `native_validated_dense_prompts_fit_the_actual_joint_receiving_ceiling` (`config.rs:1399`), takes 3.6 s alone and 5.3 to 6.5 s loaded, and the margin is 0.5 to 3 s.
   - **First visible:** 13:53 (row 160). **Acted on:** each gate passed on its next try. The 17:35 check named the budget for review, and no record of it reached `later.md`.
   - Proposal 5. smith's suite is section 9, smith item 1.
9. **skein's aims were met only on a quiet machine.**
   - **What happened.**
     - Over the 43 passing skein land gates, the aims' package sums were 15.8 to 28.7 s focused and 17.5 to 28.6 s fuzzy. Both aims were met only at row 896, at 22:42 after the reboot, and at an owners gate at load 2.48.
     - "Done when" ran at load 7.4 to 8.8, one second after an owners smith gate ended, while the graph was being re-indexed.
     - smith's re-measure met its aims by 0.19 s and 0.83 s, because the median of three dropped a cold first run 50% slower.
   - **Cause.** The aims are a sum of single tests' times, which load stretches. The six slowest skein tests date from 10-04 to 10-09, not from this run.
   - **First visible:** 12:23, at the first gate. 9 of the 11 checks said "load, not growth".
   - **Acted on:** ruling 152, with "For the user" E.
   - Proposal 6.
## 6. Decisions taken during the run

52 rulings, 101 to 152, each a smith commit "Ruling N of accounts", and 26 steers. By kind:

| Kind | Rulings | Reached the design? |
|---|---|---|
| Shapes against `main`, from ready checks and re-checks (25) | 101 to 105, 108, 109, 114, 115, 117, 118, 120 to 123, 126 to 128, 131, 132, 136 to 138, 143, 144 | Plan only. 128 (fake issuer literals use `Method::Post`; world token POST times at least 60) is "not in the design yet" (`done.md`). 144 added skein-10.s1, which follows oauth.md 6.4 |
| Blocks (15) | 106, 107, 111, 112, 113, 116, 124, 130, 133, 135, 139, 140, 141, 147, 149 | Only 112, as skein `0ccaf59`, oauth.md 1, 5 and 6.7 (below) |
| Test-shape rules (8) | 110, 119, 125, 134, 142 to the README or a session file; 146, 148, 150 to `later.md` | Plan rules. testing-strategy.md section 10 says most of them already. 148's "fixture constants once per crate" and "setup drains earlier events" are "not in the design yet" |
| Order and steps (2) | 129: `jig-repin.s1` now, and the start order toward the repins; 144, skein-10.s1 | Plan only |
| "Done when" (2) | 151, three stale commands; 152, the aims, with "For the user" E and a `later.md` row | Plan only |

The decisions whose substance belongs in a design:

| Ruling | Commit | What | Reached the design? |
|---|---|---|---|
| 112 | `8ab73679`, 14:03 | An empty added-parameter or sign-in-again name is refused as `Limit`, as an empty client id is; an empty value is admitted; `client_worst_case` prices the entries | **Yes:** skein `0ccaf59` (14:01), with smith `d275fd98` (05.3's Size 240 to 275), by designer `fe8ad627` |
| 130 | `b638629f`, 17:58 | jig's and temper's repins: `Refused` reaches no downstream event and adds no hub accessor; `Unauthorized` maps to `Rejected`, `Limited` to `Exhausted`, an absent reset to zero | **No:** "not in the design yet". An unknown reset reads as "available now" (section 9, jig item 4) |
| 141 | `617c6e72`, 20:02 | smith's spare-memory floor is 460,000,000 bytes, and a later margin below it is a question, never a lower floor | **No:** the floor is in no design document |
| 128, 148 | see above | Test-library conventions | **No** |

tools' proposal 3, adopted at 12:30 (`656ef69`), says a question decided without a designer reaches the design before the next plan. Rulings 130 and 141 are such questions, and `accounts-sources`, the next plan, builds on both.

**"For the user."**
- A to D were decided in the decision before the plan landed. They are the licence's copyright line, smith's fault names, the repins' defaults downstream, and no live check.
- C was relied on in ruling 130.
- E was added by ruling 152 at 01:23, and has no answer yet.

**The design questions** 6 to 16 were answered by the designer before the plan landed, each as recommended: smith `137a1cb4`, skein `d4ef1680`, `378fd8cf` and `49725eca`.

## 9. Debts left in the code

For each repository's `docs/development/tech-debt.md`. Only smith and jig have one, and neither lists any of these. They are ordered by what they cost `accounts-sources`, the next plan.

**smith**

1. **The default suite has no room under its 15 s budget.**
   - **Where:** `.config/nextest.toml`, `global-timeout = "15s"`. The slowest tests:
     - `crates/smith-agent-shell/src/config.rs:1399` `native_validated_dense_prompts_fit_the_actual_joint_receiving_ceiling`, 3.6 s alone and 5.3 to 6.5 s loaded;
     - smith-bench's `probe_command` test, 2.1 to 3.2 s;
     - smith-bench's `offline recorded_catalogues_and_results`, 2.0 to 3.1 s;
     - smith-tools-world's `a_domain_with_every_kit_full_stays_within_its_worst_case`, 1.7 to 2.7 s.
   - **What it cost:** four land gates and five build-lane runs.
   - **Since:** these tests date from 10-05 to 10-11. The run added 65 tests and about 0.3 s.
   - **Needs:** the cause found before the next plan's first gate (proposal 5).
2. **The memory margin is 4.02 MB above the floor.**
   - **Where:** `crates/smith-agent-shell/src/config.rs:1606`, `>= 460_000_000`, lowered from 490,000,000 by ruling 141.
   - **How much:** 1,750,572,809 bytes complete, against 2,214,592,512 declared.
   - **Cause:** `Kind::First` and `Kind::Window` gained `account` and `backend` (`crates/smith-domain-session/src/record.rs:119-137`), and `Turn` grows with its `Kind`. That is about 27 MB, 22 MB of it from `61342a7a`.
   - **Risk:** the next plan's program backend and its accounts sit on the same sum. The floor is in no design.
   - **Needs:** a decision before the first increment that grows `Turn`: box `Kind`'s payload, which ruling 141 declined, or re-price the declared memory.
3. **smith-bench's probe test fails in any dirty checkout.** This is live-text's smith item 1, still open.
   - **Where:** `benchmarks/tests/probe_command.rs:105` builds an arm from the checkout, and `benchmarks/src/arm.rs:97` refuses it.
   - **What it cost:** "Done when" item 4, and nine of the authors' runs, 1,704 s. The message names no file.
   - **Since:** `98163c75`.
   - **Needs:** the test builds from a clone of its own. The next plan changes the arms (benchmarks.md 6, 7 and 10).
4. **Long files grew.**
   - `crates/smith-agent-service/src/service.rs`: 2,920 to 3,087 lines; `9bd292ba` alone added 74.
   - `tests/agent-process/src/world.rs`: 1,433 to 1,648.
   - `tests/channel/tests/stories.rs`: 1,255 to 1,404.
   - `crates/smith-events/src/write.rs`: 2,190 to 2,283; `read.rs`, 1,865 to 1,941; `responses.rs`, 1,855 to 1,919.
   - Near 800, on the next plan's path:
     - `crates/smith-local-service/src/credentials.rs`, 759;
     - `crates/smith-local-protocol/src/terminal.rs`, 774;
     - `crates/smith-local-shell/src/settings/tests.rs`, 756;
     - `crates/smith-protocol-llm/src/component.rs`, 768;
     - `crates/smith-host-protocol/src/component.rs`, 775.
   - Needs code-organization.md 7's consolidation, before an increment adds more than 40 lines to `service.rs`, `config.rs` (1,712) or `credentials.rs`.
5. **The withdrawn forms, marked for `accounts-sources`** (`later.md` rows 1 to 8):
   - `crates/smith-agent-shell/src/config.rs:182`, `_account`, ignored;
   - `crates/smith-protocol-channel/src/credentials.rs:20-22`;
   - `crates/smith-agent-service/src/service.rs:926,938`;
   - `crates/smith-local-service/src/service/accounts.rs:56`;
   - `crates/smith-local-shell/src/credentials/product.rs:161`, a fixed 60 s `refresh_lead`;
   - the `unreachable!` driver events in `crates/smith-local-service/src/credentials.rs:513,738-750` and `credentials/private.rs:83,95,115`, which `smith logout` makes reachable.
6. **Landed tests that break section 10** (`later.md`):
   - `crates/smith-agent-service/src/tests/accounts.rs:15-22,39`: bare `Limits { 256, 1024, 8 }`, `account: 7` and `reset_ms` in long rows;
   - `tests/local-process/src/process.rs:128`, the issuer's floor retyped;
   - `tests/protocol-llm/tests/adapter_boundary.rs:204,239,262`, a function in a row.

**skein**

1. **The fake issuer's response floor is typed in six places, in three repositories.**
   - **Where:** `testing/skein-fake-peers/src/oauth.rs:633-635` computes it inline. It is retyped as `256 + 6 * 1024 + 267` at:
     - `tests/oauth-accounts/src/world.rs:66,1029,1169` and `sign_in/protocol.rs:64`;
     - smith's `tests/local-process/src/process.rs:128`;
     - jig's `tests/protocol-accounts/src/fixtures.rs:8-9` and `world.rs:130-131`.
   - **Risk:** the next plan moves the issuer to a sign-in account's addresses. A new field changes six files, or the peer refuses with `Error::Limits` and no pointer to the cause.
   - **Needs:** one exported function in skein-fake-peers, then the repins.
2. **The oauth-accounts world's sign-out stories bypass the issuer's revocation route.**
   - **Where:** `tests/oauth-accounts/src/world.rs:1124-1136,1300` point the token endpoint at `/revoke`, and `sign_out.rs:29,33` takes a story it ignores.
   - **Since:** skein 09's report said this lasts "until the hosted issuer's separate route lands in skein 11". `21ad555` landed it, and the world was never switched.
   - Two near-copies of the stream pump, `world.rs:1117` and `sign_in/protocol.rs:13`, each sit behind a `too_many_lines` expect in tests, against the Don't list.
3. **The keep-time flake** (`later.md`, ruling 116).
   - **Where:** `tests/world/tests/real.rs:129`, whose 5 ms deadline is taken before the world's setup. It failed once in 56 runs.
   - **Same pattern, unrun:** `:148` and `:160`.
4. **Long files grew.**
   - `crates/skein-oauth-accounts/src/component.rs`: 1,135 to 1,291 lines. `has_work` (`:1063`) and `next_deadline` (`:1117`) are 130 lines of scheduling.
   - `tests/oauth-accounts/src/world.rs`: 1,224 to 1,407.
   - `tests/llm/tests/fake.rs`: 1,210 to 1,419.
   - `crates/skein-oauth-accounts/src/private_tests.rs`: 382 to 777.
5. **Landed tests that break section 10.**
   - In `later.md`:
     - `crates/skein-oauth-accounts/src/openid_tests.rs:18,38,42`, bare ports;
     - `:78-85`, `signed_in_names_the_records_subject` asserts `subject.is_none()`, the opposite of its name;
     - `shared_tests.rs:372,376`.
   - In no record yet:
     - `tests/llm/tests/credentials.rs:94,114`, three outcomes under one name, and a branch on a row's `expected`;
     - `crates/skein-oauth/src/machine_tests/issued.rs`, `b"client-1"` seven times, and `revoke.rs`'s 6-tuple rows;
     - `machine_tests/openid.rs:290`, bare 2047, 2048, 8192 and 8193;
     - `tests/llm/tests/fake_credentials.rs:148-165`, a mixed table.
6. **Production repeats.** `crates/skein-oauth-accounts/src/component/accounts.rs:346` and `:387` repeat one origin test, and `:387` compares `Option == Option`, so two unparsable origins would agree (an inference). `exchange.rs:186-213` branches on `Method::Post` four times.
7. **The aims on x1** (`later.md`, ruling 152).
   - The six slowest tests predate the run: `tests/llm/tests/fuzzy_client.rs:199`, 8.26 s, and `tests/tls/tests/fuzzy_world.rs:118`, 5.75 s, among them.
   - The fuzzy gate took 32 to 52 s of its 60 in the run.
8. **The second part's markers:** `crates/skein-llm/src/types.rs:15,362,388` and `client.rs:1665`, and `FileLower::Remove` (decision 18).

**jig**

1. **A client race fails a gate now and then** (`later.md`, ruling 145).
   - **Where:** `tests/client-browser/tests/browser_tier.rs:37`, `under_a_prefix_the_inbox_s_links_carry_the_prefix`, person step 9.
   - **Where to look first:** the order of the durable `Cancelled` answer and the task watch's `Gone`. The unproved suspects are `crates/jig-client-view/src/ui.rs:798` and `objects.rs:720`.
   - **Cost:** 99 minutes of jig-repin.s1.
   - The known-flake rule that let it land ends with this plan's README.
2. **`const DIALECT: u32 = 1` is in four files** (`later.md`): `crates/jig-protocol-transcript/src/tests.rs:4`, `tests/client/src/joint/fixture.rs:5`, `tests/client/src/joint/stories/job_stories.rs:5` and `tests/ops/src/conformance/scripts.rs:34`. smith exports no such number to name it from.
3. **Accounts that nothing compares.** jig still carries `Endpoint.account` (`crates/jig-protocol-config/src/llm.rs:22`) and `Model.account` (`crates/jig-domain-core/src/run.rs:303`). The agent's `ModelBound` has no account. So a wrong account fails at run time, not at startup (an inference). See temper, item 1.
4. **Ruling 130's mapping has no design.**
   - **Where:** `testing/jig-test-worker/src/domain.rs:387-399`.
   - **The effect:** `crates/jig-domain-accounts/src/domain.rs:207-217` benches an account with no known reset until `now + 0`.
   - **Risk:** key, environment and program accounts make `Refused` and an unknown reset common.

**hearth**

1. **The default suite cannot pass on x1.**
   - **Where:** `tests/tree/tests/memory.rs:321`, `walking_a_layer_at_its_limits_holds_no_more_than_the_worst_case`, 12.6 to 14.1 s.
   - **How much:** 2,150 tests take 22.6 to 25.4 s on x1, against 6.5 to 8.8 s on hetzner.
   - **Since:** `3df74d3`, 23:03. `c4d0231` trimmed one test.
   - Every hearth repin from x1 meets it. Needs hearth's own cause found, never a bigger budget.

**temper**

1. **The account match was dropped without a question** (`later.md`, ruling 150).
   - **Where:** `crates/temper-service/src/startup/checks.rs:447-454`. The text and tests still say "under the same name and account": `startup/problems.rs:85` and `tests/checks.rs:80,86`.
   - **The root** is smith's: `EndpointOptions` has no account, and `config.rs:182` ignores it.
2. **Ruling 130's mapping** is also in `crates/temper-worker-domain/src/agents.rs:127-135`.
3. **Landed tests that break section 10** (`later.md`):
   - `tests/worker/domain/src/protocol.rs:21`, `SMALLEST: [u8; 120]`;
   - `b"acc"` six times;
   - `"dialect": 1` in `docs/examples/owned.json:26` and `owned/agent.json:22`.
4. **Each upstream limit's growth is hand arithmetic** in `docs/examples/owned.json`, at `:192` and `:825`.
