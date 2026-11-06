# real-journeys: journeys recorded on real models, with their secrets out before anything is printed

The spec is `temper/docs/design/draft/real-journeys.md`: read it whole. **The user has since narrowed it, 2026-11-05: keep only the secrets' redaction and the last scan.** A real box already serves the recording bundle with `application.clients.bundle` set, so `temper box --bundle` is out. The user doesn't mind the account's email in a committed journey, so email redaction and the foreign-email refusal are out too; `--redact TEXT` stays if it costs nothing. Say in the design what's dropped. Everything below the next section follows that narrowing.

## In short

- `temper box --bundle BUNDLE` on real backends serves the recording bundle.
- `temper-client-journeys add` redacts before its first replay's proof: the `secret` and `stored` values, act values whose event shows `Secret { len }`, every email outside the example domains, and each `--redact TEXT`, each original to one same-length placeholder everywhere, heard pieces decoded from their `data:` base64 and encoded back, so the replay still holds.
- A last scan refuses any original, foreign email, or key or token shape (`sk-`, `eyJ` JWTs, `Bearer`, `code=`, `refresh_token`), naming record and kind, never bytes. Nothing is written then.
- Real journeys are recorded by hand, not by `again`, and marked real in `temper/docs/design/client/journeys.md`, section 9.

## What the plan must hold

**This is the guard between a real account's secrets and a public repository, so its tests carry the plan.**

- **A failing test first for each kind:** each thing replaced, in each place it can appear: a plain field, a heard piece's base64, a split across two heard pieces if one stream can split a token, and a URL's query. A token split across pieces that no single line shows is the case to think hardest about: say in the report what the scan sees of it, and if it can't be caught, refuse or say so plainly. A gap here is the user's to know before they record.
- **The refusal never prints what it found:** a test checks that no original's bytes reach stdout, stderr or a file on any path, a failed proof's output included.
- **Nothing reaches the output path until the scan has passed:** no temporary file in the repository's tree either.
- **Test data is fake:** a token's shape built in the test from pieces, so the repository's own checks and `tests/boundary/tests/outward.rs` stay clean. No test reaches a real host or account.
- **Bounded by what's known:** the scan's shapes aren't all a secret could be. The draft's `--redact` is for the rest. The help says so, in a sentence, so a person recording knows what they still have to look at.

## Size

Mostly `temper-client-journeys` and a flag on `temper box`. `box-link-first` is being built alongside, in the box's terminal and its start: this plan's line under the sign-in block goes where that plan puts the block. Name that overlap, keep its hunks small, and expect a catch-up: that plan lands first if both are ready. fabrig/ui owns the journeys' files by priority: coordinate through the plan's overlaps, and leave its journeys themselves alone. Budgets and timeouts are the user's (skein's `docs/foundation/testing-strategy.md`, 8.1). Formats change in place before v0.1.0.

## Walk

`walk = false`: the user's own recording is the first real use.
