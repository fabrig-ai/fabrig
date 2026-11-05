# adopt-access: the plan

From `inputs/request.md`.

## Why it happens

- **Step 3 is a bug.** `host::Plan::Adopt` is staged only by `temper-service/src/setup/lifecycle.rs`, `advance`, for a fake forge's start-time repositories. `temper-engine-domain-forge/src/adoption.rs`, `start`, goes straight to its reads, so a person's adoption grants nothing on any mode, owned too. `temper/docs/design/domain/forge.md`, 4.2, and `protocol/setup.md`, 5.4, say it should, and no status file said it didn't. The admin's token is kept (`Item::AdminToken`, `lifecycle.rs`, `keep_admin`), and a start's reconcile does get the adopted ids (`reconcile_local`).
- **Step 4 on a managed forge is the design.** `protocol/setup.md`, 5.5: "no listener: it polls", followed by `setup/runtime.rs`, `forge` and `local`, and pinned by `string_managed_serves_no_listener_even_when_owned_forges_have_webhooks`. Other sections read otherwise. As the request says, that is a question and not a widening: `managed-listener` in `plan.toml`.
- **The silence.** An adoption whose bot can only read is admitted, and `temper ctl` prints only "temper may: read."
- **The plan can't start from an id.** The wire's `Adopt` carries owner and name, and the bot can't see a private repository, so `Locate` on its token answers `NotFound`. Hence `adopt-plan`: `Adopt` given a path, with no wire change.

## Parts and tasks

| # | Task | Needs | Effort, minutes | What it lands |
|---|---|---|---|---|
| 1 | `adopt-plan` | | high, 45 | skein's `Adopt` given a repository's path |
| 2 | `adopt-grant` | 1 | high, 170 | the failing story first; the domain asks, the routing runs `Adopt`, a stop refuses; `temper ctl`'s words; a managed start's stop on the row |

The request's real-Forgejo test is the elsewhere story, extended rather than a new Forgejo start, for the process tier's budget.

## Critical path

`adopt-plan`, then `adopt-grant`: about 215 implementer-minutes.

## Question

`managed-listener`: no task waits for it. Until it's answered the story asserts no hook on a managed forge, and the hook on an owned one is checked on the fake forge in `temper-service`. The recommended answer is one small follow-up, which `plan.toml` names.

## Overlaps

- **`ui-away` (cut in flight)** changes `temper-engine-domain-forge/src/{adoption,boundary,domain}.rs`, `temper-engine-domain/src/{boundary,admission}.rs` and `temper-service/src/family.rs`. `adopt-grant` adds variants beside `Reachable` and a stage in `adoption.rs`. Unavoidable: that's where an adoption starts. The edits are small and additive.
- **`clearer-words` (cut in flight)** names no files yet; `adopt-grant` adds two words and rewords none.
- **`ui-chat-draft`** changes `client/README.status.md`; `adopt-grant` adds and shrinks its own entry.
- **The UI's files** stay out: the form's read-only line stays **Not yet** in `client/README.status.md`.
