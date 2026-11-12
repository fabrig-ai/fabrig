# dogfood-hacks: status

2026-11-11. Where the design lags or departs from [dogfood-hacks.md](dogfood-hacks.md).

- **Design lags:**
  - 2.1: `../domain/worker/checkouts.md` is as it was; `../domain/worker/checkouts.status.md` (Departs, the whole document) and `../protocol/git.status.md`, 2 and 4, record the departure while the variable is set.
  - 2.2 to 2.7, 2.9 to 2.12: by intent, nothing in the design describes them: each is temporary and lives here until a real design replaces it.
  - 2.13: by intent, nothing in the design describes it, and it stays here until a real design replaces it; `../domain/worker/README.md`, section 10 names only the question it leaves. `../domain/worker/README.status.md`, `skein/docs/design/llm/README.status.md`, `skein/docs/design/llm/connection.status.md`, `skein/docs/design/testing/fakes/llm.status.md`, `smith/docs/design/protocol/agent.status.md` and `smith/docs/design/protocol/programs.status.md` record the hooks it adds. On Claude Code it reads what the program is given, not what the program sends.
  - 2.8: `../protocol/setup/owned.md`, 2 and 8, and `../protocol/openid-provider.md`, 4 give auto-registration and `ALLOW_ONLY_EXTERNAL_REGISTRATION` as the only settings and say nothing of `TEMPER_FORGE_NO_REGISTRATION`; `../testing/process-tier.md` and `../protocol/openid-provider.md`, 8 say nothing of the leg it adds to the Forgejo sign-in story. By intent they stay so until a real design replaces it.
- **Design departs:**
  - 2.1, 2.4: they name a run's hooks as what the worker's own git runs across workstreams, and 2.1's real design waits on hooks being off. `core.hooksPath=/dev/null` already holds a hook off in every git the worker runs (`../protocol/git.md`, section 3), so the residual clone-trust gap remains `../domain/worker/checkouts.status.md` (Departs, the whole document).
  - 2.8: the request of 2026-11-05 has no `ALLOW_ONLY_EXTERNAL_REGISTRATION` so that `DISABLE_REGISTRATION` also closes the form Forgejo's link page would offer. At Forgejo 17.0.1 it doesn't: with these settings the page still serves a form for `/user/link_account_signup`, with no control in it. So 2.8 and the story hold that request's "no way to register" as no control in that form, both POSTs answering 403 and no account made; the story fails if a control appears. The plan's requester decided it, 2026-11-05: no custom template, since keeping one costs more than an empty wrapper is worth. Nothing records the user weighing it.
- **Open:**
  - 4: the worker's machine.
  - 4: which entries, if any, become real features before the switch.
  - 4: whether `ENABLE_INTERNAL_SIGNIN = false` leaves the link page's password form: not run, so 2.8 keeps internal sign-in on.
  - 4: what a deployment's operators may read of a run's prompts.
