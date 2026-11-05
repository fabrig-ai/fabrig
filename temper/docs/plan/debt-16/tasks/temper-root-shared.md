+++
id = "temper-root-shared"
part = "temper-shared"
kind = "feature"
needs = []
files = ["temper/crates/temper-engine-domain/src/link.rs", "temper/crates/temper-engine-domain/src/admission.rs"]
effort = "medium"
minutes = 30
+++

# temper: the engine's root uses jig's job assignment, settled relay, sender label and `new_work`

## Goal

D004's part 3, in `temper-engine-domain`. Its root spells four things jig holds. Its outputs stay byte for byte.

Acceptance:

- `delivery` in `link.rs` calls `hub::job_assignment` and `hub::settled_answer`.
- `rg -n 'fn sender' temper/crates/temper-engine-domain/src/link.rs` finds nothing.
- `rg -n 'LinkUp' temper/crates/temper-engine-domain/src/admission.rs` finds nothing.
- `temper/crates/temper-engine-domain/src/decision.rs` is untouched.
- `heavy cargo nextest run -p temper-engine-domain -p temper-engine-domain-world` passes with no test changed.

## Inputs

- `temper/docs/plan/debt-16/notes/common.md`
- `jig/docs/development/debt/D004-ops-root-jig-wiring.md`
- `jig/docs/design/domain/root.md`, 3 and 5.

## Owns and may change

- **Owns:** `temper-engine-domain/src/link.rs`, `src/admission.rs`.
- **Stays, as temper's own:** `message`, `inbox_sender` and `inbox_words`. `jig_protocol_charter::message` renders other
  words (`QuestionAnswered`, `QuestionEnded`, `Escalation`, `Met`), and this crate links neither the charter nor smith.
  `inbox_sender` keeps its own text around the label.
- **No new dependency line:** `jig-domain-hub-link`, `jig-domain-host` and `jig-domain-tasks` are already there.
- **`Class` isn't adopted,** though the request names it. It lives in `jig-domain-link`, which this crate doesn't link, and
  adopting it would change the public `completes`, whose three callers in `temper-service` read a `bool`. `new_work` is the
  shared part that `root.md`, 3 names and that `root.status.md` asks for.
- **`decision.rs` stays:** `Core::decision_header` is `wrote || dirty`, and temper's needs `persist_header && dirty`.

## Steps

1. `hub::Request::AssignedJob`: keep the `job_workspaces.remove` and the assert, and they come first:

   ```rust
   let workspace = domain.job_workspaces.remove(&job.task);
   assert_eq!(workspace.is_some(), job.workspace.is_some(), "prepared job items");
   crate::Delivery::AssignedJob { channel, workspace, job: hub::job_assignment(job) }
   ```

2. `hub::Request::Relayed`: `hub::settled_answer(call.answer)`; on `Err(answer)` the call gets its answer back and the
   `crate::HostDelivery::Answer` goes out as it does now.

3. `inbox_sender` reads `word.from.sender()`; `fn sender` goes.

4. `admission.rs`, `worker_completes`:

   ```rust
   WorkerInput::Link { link: Link::Up { up, .. }, .. } => !up.new_work(),
   ```

   The sets match: only `Call` is new work on both sides. `use jig_domain_host::LinkUp;` goes.

5. `heavy cargo nextest run -p temper-engine-domain -p temper-engine-domain-world`.

6. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-engine-domain --all-targets --
   -D warnings`; the citations check. Commit and push.

## Report

The common note's items. No new rule, so nothing to break by hand.
