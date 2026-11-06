# ui-walk-2: the rules every task names

The request is `temper/docs/plan/ui-walk-2/inputs/request.md`: read all of it, it is short, and its *For every task of this plan* binds you and is not repeated here. Where this note and the implementer's text differ, this note wins.

## Departures from the implementer's text

- **Decide, don't ask.** A `Question:` is only for what truly blocks your task. A stored format, a protocol or a public interface may change freely, with no compatibility; say so in your report. A small edit just outside your files (a re-export, a `mod` line, a design line, a golden) is made and named in your report.
- **Design text:** this cut wrote the design lines your task builds to, and a **Not yet** entry for them in the status file beside. Your last commit removes that entry. A fix that makes another design line wrong edits that line in the same commit; name it in your report. Nothing else under `docs/design/`, but generated prints.
- **A box, Chromium and a render script run directly,** outside `heavy`: they build nothing. Every build and test still goes through `heavy`.

## What holds

- **Keep every assertion.** A test that reads what you change is changed only where the design line you build to says otherwise, and your report names it.
- **Files other plans are changing:** `temper/crates/temper-client-domain/src/pages/inbox.rs`, `pages/needs.rs` and `tests/pages/inbox.rs` (`ui-walk-large/needs-proposal`), `temper/crates/temper-client-view/src/words.rs` (several) and `temper/docs/design/client/README.status.md` (several). Add to them; move and reformat nothing you don't change.
