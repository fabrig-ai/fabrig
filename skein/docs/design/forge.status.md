# forge: status

2026-11-04. Where skein's code lags or departs from [forge.md](forge.md).

- **Not yet:** 4.3: `skein-forge/src/forgejo/pin.rs` pins 17.0.0 and 13.2.0, with Linux's digests alone and a `digest(binary, arch)`; the conformance's facts are recorded at them (9).
- **Not yet:** 9: the seven `actions-artifact` facts and their ten transcripts hold at Forgejo 17.0.0 and are unchecked at the pin: they were recorded by hand, and the conformance has no recorder for them.
- **Not yet:** 3.2: `Labels`, `CreateLabel` and `UpdateRef`, and `CreateIssue`'s labels, in neither dialect (`skein-forge/src/api.rs`'s `Read` and `Write`). 3.6: `Protection`'s `pulls_required` and `pulls_bypass`. 3.7, 10: GitHub's dialect answers the Actions calls, `Artifact` among them, `Tree`, `Commits`, a fast-forward-only merge and an update by rebase `Refused` (forge-github.md, 4.5). 9: no fact records that a job no runner took has no steps and no log, nor that comments take no page.
- **Departs:** 4.2: for call plans, `skein/docs/foundation/programming-model.md`, 5.4 asks for one explicit state enum per lifecycle, each state holding exactly its data, no lifecycle in flags or `Option`s beside it, exhaustive matches and one handler per cell; the plans under `skein-forge/src/forgejo/plans`, including `reads.rs`'s and `writes.rs`'s `Plan`, keep a `Stage` beside bags of fields, with stage-specific data in flags and `Option`s, and use `if let` and `let … else` on their own enums; no reason is recorded (the code's side is `skein/docs/development/debt/D021-forgejo-plan-states.md`).
