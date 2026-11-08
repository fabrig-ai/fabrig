# dogfood-hacks: status

2026-11-08. Where the design lags or departs from [dogfood-hacks.md](dogfood-hacks.md).

- **Design lags:**
  - 2.1: `../domain/worker.md`, section 5 is as it was; `../domain/worker.status.md` and `../protocol/git.status.md` record the departure while the variable is set.
  - 2.2 to 2.7, 2.10 to 2.12: by intent, nothing in the design describes them: each is temporary and lives here until a real design replaces it.
  - 2.13: by intent, nothing in the design describes it, and it stays here until a real design replaces it; `../domain/worker.md`, section 12 names only the question it leaves. `../domain/worker.status.md`, `skein/docs/design/llm.status.md`, `skein/docs/design/llm-connection.status.md`, `skein/docs/design/fake-llm.status.md`, `smith/docs/design/protocol/agent.status.md` and `smith/docs/design/protocol/programs.status.md` record the hooks it adds. On Claude Code it reads what the program is given, not what the program sends.
  - 2.8: `../protocol/setup.md`, 6.2 and 6.8, and `../protocol/sign-in.md`, 9.4 give auto-registration and `ALLOW_ONLY_EXTERNAL_REGISTRATION` as the only settings and say nothing of `TEMPER_FORGE_NO_REGISTRATION`; `../domain/testing.md`, 2.6 and `../protocol/sign-in.md`, 9.8 say nothing of the leg it adds to the Forgejo sign-in story. By intent they stay so until a real design replaces it.
- **Design departs:**
  - 2.1, 2.4: they name a run's hooks as what the worker's own git runs across workstreams, and 2.1's real design waits on hooks being off. `core.hooksPath=/dev/null` already holds a hook off in every git the worker runs (`../protocol/git.md`, section 3), so `../domain/worker.status.md` names what that leaves, a filter driver or a URL rewrite in the `.git` a run wrote, because under those settings real git ran no `post-checkout` hook and did run a filter driver (the `worker-slots` cut, 2026-11-05).
  - 2.8: the request of 2026-11-05 has no `ALLOW_ONLY_EXTERNAL_REGISTRATION` so that `DISABLE_REGISTRATION` also closes the form Forgejo's link page would offer. At Forgejo 17.0.1 it doesn't: with these settings the page still serves a form for `/user/link_account_signup`, with no control in it. So 2.8 and the story hold that request's "no way to register" as no control in that form, both POSTs answering 403 and no account made; the story fails if a control appears. The plan's requester decided it, 2026-11-05: no custom template, since keeping one costs more than an empty wrapper is worth. Nothing records the user weighing it.
- **Open:**
  - 4: the worker's machine.
  - 4: which entries, if any, become real features before the switch.
  - 4: whether `ENABLE_INTERNAL_SIGNIN = false` leaves the link page's password form: not run, so 2.8 keeps internal sign-in on.
  - 4: what a deployment's operators may read of a run's prompts.
