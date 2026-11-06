+++
id = "stock-room"
part = "w6-room"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-service/src/stock.rs",
  "temper/crates/temper-service/src/tests/stock.rs",
  "temper/tests/engine/domain/tests/suite/forge_routes/status.rs",
  "temper/crates/temper/tests/fake_backends/walk/status.rs",
]
effort = "high"
minutes = 45
+++

# The stock keeps a chat's room outside the pools

## Goal

W6, decision 1. A fresh project's period is all reserved once its owner's pool opens, so `connector_repair_authority` gives no seed and Home's brief never runs, silently. Make the stock's period one chat's 24K larger than the owner's pool, as `notes/contracts.md`, 1, has it, and drop the funding cuts the two status worlds used to hide this.

This is the second of the request's options, the allowance outside the pool. The pool stays 80 × 24K, so nothing a person may spend changes. Cutting the pool would have made "eighty chats at once" false in the design, the help and two tests, for the same one line of code.

Acceptance:

- `notes/contracts.md`, 1's table holds in `stock::defaults`, for `temper init`'s models and for a cheap one.
- On the stock policy, with the owner's pool open, `repair_seed` gives a seed and a status run reaches its brief in the root's world, with no change to the defaults for it.
- The fake box's status walk passes with no `project fund` in it.
- With the period cut to the pool (a project's owners may set that), the run is refused as before: no brief and nothing asked of anybody.

## Inputs

- `temper/docs/plan/ui-walk-large/inputs/request.md`, W6.
- `temper/docs/plan/ui-walk-large/notes/common.md`; `notes/contracts.md`, 1.
- `temper/docs/design/protocol/configuration.md`, 13.4; `temper/docs/design/domain/forge.md`, 5.7, *What funds it*.
- `jig/docs/design/domain/authority.md`, section 7: a pool reserves from its period.

## Owns and may change

Owns the `files` above.

May also change, each only so:

- `temper/docs/help/budget.md`: its one sentence on the ceilings and a project's period says the owner's pool is 80 × 24K and the period one chat's more. A departure the task states: `docs/help/` is otherwise closed.
- `temper/docs/help/configure.md` and `temper/crates/temper/src/tests/help.rs`: the example's `period_spend` 8200000 beside rules' 8300000 sits below the new stock's 8219232. Make it 8300000 beside 8400000, in both files.
- `temper/docs/design/protocol/configuration.status.md`: remove the **Not yet** line on 13.4's room. `temper/docs/design/client/README.status.md`: remove the **Not yet** on 3.5, *Home on any project*, that starts "on the stock policy the status agent is left no funding". Only those lines.
- A test elsewhere in temper that holds a stock period's budget or what is left as a literal: that literal, and name it in your report.

Don't touch `temper-client-*`, `limits.rs` or `jig/`: other tasks own them.

Options, each pre-approved:

- **`startup_refuses_stock_budget_overflow...` or another overflow test moves** because 81 overflows sooner than 80: use `checked_mul`, so `checked` still answers none, and keep the test's meaning.
- **The status world's story needs a person's chat to open the pool:** start one, as the file's others do.
- **If a run with the room is refused all the same,** find why (`child_authority`; the delegate's `Budget::Named`). Fix it in `status_route.rs` if the fix is local, and say so; if it would change jig, report it under `## Left`, with the refusal, and keep the rest.

## Steps

1. `stock.rs`, in `checked`, and `project_policy`:

   ```rust
   // Eighty chats or goals funded at once from one pool.
   let pool_spend = chat_spend.checked_mul(80)?;
   // One chat's more in the period, which no pool reserves, for what the deployment
   // itself starts (protocol/configuration.md, 13.4).
   let period_spend = chat_spend.checked_mul(81)?;
   // rules: ceiling: ceiling(pool_spend), period_spend, ...
   // policies.insert(1, project_policy(period_spend, pool_spend, chat_spend))

   fn project_policy(period_spend: u64, pool_spend: u64, task_spend: u64) -> ConfiguredPolicy
   // ceiling: ceiling(pool_spend), period_spend,
   // roles 0 and 1 project_role(_, pool_spend, task_spend, ..), role 2 and 3 as they are.
   ```

2. `tests/stock.rs`, where it holds the old equality (`stock_...` at the `80 * 24 * k` loop, `80 * chat <= period`, and the `81 * chat_spend` bound): the ceilings and pools 80 × 24K, the period and the rules' period spend 81 × 24K, and "an eighty-first open chat exceeds the pool". Cut any row that repeats `stock.rs`'s list to the ones that can fail on the rule.

3. `forge_routes/status.rs`: remove the `funded` shrink of each role's `period_spend`. `funded` false now sets policy 1's `period_spend` to its role 0's, so the pool reserves the period whole. Add:

   ```rust
   #[test]
   fn a_stock_project_with_its_owners_pool_open_writes_its_brief()
   // ready, a chat opening the owner's pool (assert the period's funding shows
   // the pool reserved and 24K left), a landing, a next sitting, the run's words: facts(..).brief is some.
   ```

   Break the rule once: `period_spend` back to `pool_spend` in `stock.rs`; see this test fail on no brief.

4. `walk/status.rs`: remove the `project fund` act and its comment, and lower the answers' count it asserts.

5. Help and status lines, as above.

6. **A change of meaning runs its readers' tests:** what every stock project's period is. Besides the implementer's text, run `temper-service`, `temper-engine-domain-world`, `temper-client-world`, `temper-one-process-world`, `temper-system-world`, `temper-real-loop-world`, `temper --lib` (`tests/help.rs`), and `temper --test process -E 'test(=home_on_the_fake_box_keeps_a_landing_count_after_box_and_client_restarts)'`. Name them in your report.

7. fmt, `old_forms`, clippy, `python3 -I -B tools/citations/citations.py --base origin/main`; commit and push.

## Report

As `notes/common.md`, *Your report*, and the period, pool and room the stock gives on `temper init`'s box, read from a test's run.
