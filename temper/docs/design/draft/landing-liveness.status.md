# landing-liveness: status

2026-11-24. Where the design lags or departs from [landing-liveness.md](landing-liveness.md).

- **Design lags:**
  - 3: a change stores its landing's number and one handed-over ask, beside the landing's own members (`../domain/landing.md`, 4), and the start repairs stale links.
  - 4: forming reads jig's due flag and a settling memory, and judges "made ready by one event" from scheduling state (`../domain/landing-forming.md`, 2; `../domain/landing-turn.md`, 2). A full `landings` table is not named on the train (`../domain/landing-forming.md`, 3).
  - 5: no check of a live landing as a whole, at its steps or at start; a person's hold of a member is seen only when the landing next steps (`../domain/landing.md`, 4.3).
  - 6: only a cancelled landing has its own ending path; a failed one goes through generic settlement (`../domain/landing.md`, 4.2).
  - 7: local release failures carry no account, so their hold reads "could not release this task's resources".
  - 8: no design says how the previous build's links and single asks read.
  - 11: the landing policy's fuzzy checks that each wait names its wake, not the check of 5.1 (`../domain/landing-waits.md`, 3).
- **Design departs:** none.
- **Open:** none.
