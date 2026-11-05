# dogfood-hacks: status

2026-11-05. Where the design lags or departs from [dogfood-hacks.md](dogfood-hacks.md).

- **Design lags:**
  - 2.1: `../domain/worker.md`, section 5 is as it was; `../domain/worker.status.md` and `../protocol/git.status.md` record the departure while the variable is set.
  - 2.2 to 2.7 and 2.9: by intent, nothing in the design describes them: each is temporary and lives here until a real design replaces it.
  - 2.8: `../protocol/setup.md`, 6.2 and 6.8, and `../protocol/sign-in.md`, 9.4 give auto-registration and `ALLOW_ONLY_EXTERNAL_REGISTRATION` as the only settings and say nothing of `TEMPER_FORGE_NO_REGISTRATION`; `../domain/testing.md`, 2.6 says nothing of the leg it adds to the Forgejo sign-in story. By intent they stay so until a real design replaces it. The code is being built by the plan that added this entry.
- **Open:**
  - 4: the worker's machine.
  - 4: which entries, if any, become real features before the switch.
