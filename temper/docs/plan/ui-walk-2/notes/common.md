# ui-walk-2: the rules every task names

The request is `temper/docs/plan/ui-walk-2/inputs/request.md`: read all of it, it is short, and its *For every task of this plan* binds you and is not repeated here. Where this note and the implementer's text differ, this note wins.

## Departures from the implementer's text

- **Decide, don't ask.** A `Question:` is only for what truly blocks your task. A stored format, a protocol or a public interface may change freely, with no compatibility; say so in your report. A small edit just outside your files (a re-export, a `mod` line, a design line, a golden) is made and named in your report.
- **Design text:** this cut wrote the design lines your task builds to, and a **Not yet** entry for them in the status file beside. Your last commit removes that entry. A fix that makes another design line wrong edits that line in the same commit; name it in your report. Nothing else under `docs/design/`, but generated prints.
- **A box, Chromium and a render script run directly,** outside `heavy`: they build nothing. Every build and test still goes through `heavy`.

## What holds

- **Keep every assertion.** A test that reads what you change is changed only where the design line you build to says otherwise, and your report names it.
- **Files other plans are changing:** `temper/crates/temper-client-domain/src/pages/inbox.rs`, `pages/needs.rs` and `tests/pages/inbox.rs` (`ui-walk-large/needs-proposal`), `temper/crates/temper-client-view/src/words.rs` (several) and `temper/docs/design/client/README.status.md` (several). Add to them; move and reformat nothing you don't change.
- **Temper's client wire may change freely:** there is no supported deployment. A journey recording goes stale only if the exchange it holds changes (`temper/docs/design/client/journeys.md`, 8).

## Corrections to `login-command`'s spec

That spec went out with these errors, and a pushed spec never changes. For `login-command`, where this section and the spec differ, this section wins.

- **The domain's tests** are in `temper/crates/temper-client-domain/src/tests/pages/connect_model.rs`, which exists, not `src/tests/connect_model.rs`, in the header and in the Steps.
- **To the service:** there is no `startup::Checked` to add a method to: `CheckedStart` is private, and `settings::Engine` is built inside `startup`. Keep `Clients.program_environment: Box<[u8]>`, set beside `clients.deployment` in `system.rs`. The binary's `serve` (`temper/crates/temper/src/lib.rs`, `Command::Serve`) has its `Directories` and reaches `startup::startup_with`: take the narrowest route from one to the other (an argument of `startup_with`, or a field of a struct it already takes), and name it in your report. Its call sites and fixtures are yours to edit, for that alone.
- **The box's child** takes its environment from `temper/crates/temper/src/box_command.rs`, `std::env::vars_os()` near line 61. Check that `HOME` and the XDG variables reach it, or acceptance 4 fails. Worker-alone (`worker_alone::prepare`) serves no client, and needs nothing.
- **The sheet:** the class is `.temper-welcome-cmd`, not `.welcome-cmd`. `.temper-flow-code` already wraps. `.temper-welcome-cmd` (`temper.css`, under `/* ==== ui: welcome ==== */`) has `overflow-x: auto`: edit that rule where it stands to wrap as `.temper-flow-code` does, with the short command drawn as it is today. Prove both with a throwaway render at 1440 and at 420, by the request's *Render proofs*.
- **`setup.md`, 7,** in the pre-approved options, is `temper/docs/design/protocol/setup.md`, 7.
- **The last check** names `startup_with`, not `settings::Engine`: `heavy cargo check -p <dependent> --tests` for whatever else names `w::Read`, `ForgeRead` or `startup_with`.
