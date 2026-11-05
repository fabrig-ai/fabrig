# dogfood-hacks: status

2026-11-05. Where the design lags or departs from [dogfood-hacks.md](dogfood-hacks.md).

- **Design lags:**
  - 2.1: `../domain/worker.md`, section 5 keeps "a new workstream ... made again, empty" and says nothing of `TEMPER_WORKSPACE_REUSE`. The code is being built by the plan this draft was written beside.
  - 2.2 to 2.7: by intent, nothing in the design describes them: each is temporary and lives here until a real design replaces it.
  - 2.8: `../protocol/setup.md`, 6.2 and 6.8, and `../protocol/sign-in.md`, 9.4 give auto-registration and `ALLOW_ONLY_EXTERNAL_REGISTRATION` as the only settings and say nothing of `TEMPER_FORGE_NO_REGISTRATION`; `../domain/testing.md`, 2.6 says nothing of the leg it adds to the Forgejo sign-in story. By intent they stay so until a real design replaces it. The code is being built by the plan that added this entry.
- **Open:**
  - 4: the worker's machine.
  - 4: which entries, if any, become real features before the switch.
