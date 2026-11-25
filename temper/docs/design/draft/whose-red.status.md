# whose-red: status

2026-11-25. Where the design lags or departs from [whose-red.md](whose-red.md).

- **Design lags:**
  - 3: no single judgement of a check's outcome. A change's CI, a landing gate, a known flake and a repair's verdict each have their own path (`../domain/forge/change/repairs.md`, 1; `../domain/landing-proof.md`, 2.4; `../domain/landing-regate.md`, 2), and pre-PR's outer exit and record read a broken check as a refusal (`tools/pre-pr/design.md`, 6). No project setting says whether a project's checks can flake.
  - 4: a close command has exit 0 and failure only (`../protocol/tooling.md`, 4.2): a refusal is the job's own failure, tried three more times before the close holds.
  - 5: a no-change landing repair holds `Failed`, and a flake or no-test verdict holds before its regate (`../domain/forge/change/repairs.md`, 1).
  - 6: fourteen hold arms, each with its own release (`../domain/forge/change/repairs-releases.md`, 1).
  - 7: no record of temper's own writes. Drift is judged against the expected head, with a pair of heads beside a branch hold (`../domain/forge/change/drift.md`, 1; `../domain/forge/branches.md`, 3).
  - 8: no design for how the previous build's holds and branches read.
- **Design departs:** none.
- **Open:**
  - 3.3: with `checks.flakes none`, the landing gate's known-flake rerun is off too. The user's word (2026-11-25) named pre-PR's reruns, and the gate runs the same tests.
