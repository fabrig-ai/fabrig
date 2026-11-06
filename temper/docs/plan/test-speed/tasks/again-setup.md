+++
id = "again-setup"
part = "again-setup"
kind = "feature"
needs = ["harness-horizon"]
files = [
  "temper/crates/temper-client-shell-web/src/journey/setup.rs",
  "temper/crates/temper-client-shell-web/src/journey.rs",
  "temper/crates/temper-client-shell-web/src/journey/tests.rs",
  "temper/tools/journeys/src/again.rs",
  "temper/tools/journeys/src/lib.rs",
  "temper/tools/journeys/src/main.rs",
  "temper/tools/journeys/src/tests.rs",
  "temper/docs/design/client/journeys/project-landing/**",
  "temper/docs/design/client/journeys/chat-draft/**",
  "temper/docs/design/client/journeys.status.md",
]
effort = "high"
minutes = 180
+++

# `again` walks a setup file, and `chat-draft` records itself

## Goal

`chat-draft` has been re-recorded by hand three times, because `temper-client-journeys again` starts at the printed link's first page and its recording starts past a fresh box's whole set up. Give a journey an optional `setup` beside its recording, and prove it: `again chat-draft` runs alone, with no hand driver. This saves no test seconds. It ends the hand re-records.

Acceptance:

- **`journey::setup`** reads the file of `temper/docs/design/client/journeys.md`, 8.2, natively. Its tests, each failing first, in `journey/tests.rs` or a `setup`-named test module beside it: every record read back, an escape in a name among them; refused: another version, a line out of its place, a second `reload` or none, an unknown role, a `mark` before `reload`.
- **`again`** walks a setup's steps, then its `reload`, then the walk of 8.1 from `Phase::Opening`, and waits at each `mark`. Refused and named: a step not done within 60 s (its line and what it sought), a mark's text not shown within 30 s, a chain of `after`, an `after` that names no setup or the journey itself. It calls `.horizon(Duration::from_secs(600))` on the harness (`skein-fake-person-browser`'s, from `harness-horizon`).
- **Two setups committed:** `project-landing/setup` and `chat-draft/setup` (`after project-landing`), walked from each README. A native test in `tools/journeys/src/tests.rs` reads every committed setup and joins it.
- **The start stays one:** a native test replays `welcome-set-up`'s recording and finds, for each of its acts, that `project-landing`'s setup's step in that place names a control (its role and name, exactly) of which the view makes that act's event. Rename the control in a scratch copy of the setup and see it fail.
- **The proof:** `heavy cargo run -p temper-client-journeys -- again project-landing`, then `-- again chat-draft`, with `SKEIN_TEST_CHROMIUM` set, each with no other driver. Each commits its new `recording` and print. Then `heavy cargo run -p temper-client-journeys` reprints with no diff, and the journeys' tests pass.
- **The pages say what the README says:** read `chat-draft`'s pages against its list, above all `06.html` (the settled reply), `08.html` (Split after the document's write), `11.html` ("Edited", "+7 −0") and `19.html` (the settled goal). Each is the settled state it names. A page caught early gets a `mark` line, and the walk is recorded again.
- **The READMEs:** `chat-draft/README.md`'s last-but-one paragraph (the throwaway driver) says it is recorded by `again chat-draft` from its setup; `project-landing/README.md` gains the same line. The walk's words, and each page's line, stay as long as they're true.
- `journeys.status.md`'s 8.2 entry goes.

## Inputs

- `temper/docs/design/client/journeys.md`, 8.1 and 8.2; 3.1 (*One load of the page*); 3.2 (the file's form); 6 (*A setup, natively*); 9.
- `temper/docs/design/client/journeys/project-landing/README.md`, `chat-draft/README.md`, `welcome-set-up/README.md`.
- `skein/docs/design/ui-web.md`, 14.2; `skein/testing/skein-fake-person/src/lib.rs` (`Find`, `Found`, `Doing`).
- `temper/docs/plan/test-speed/notes/common.md`; the request's *For every task of this plan*, above all *Recording flakes*. The tier's sections don't apply: no tier measures this.

## Owns and may change

Owns the files in the header. `journey/setup.rs` is new; `journey/{line,record,replay,walk}.rs` are read, and may give a helper `pub(super)` (the field reader in `line.rs`).

The outer layer:

```rust
// temper-client-shell-web/src/journey/setup.rs (client/journeys.md, 8.2)
pub enum SetupStep {
    Act { role: Role, name: Box<[u8]>, order: u32, act: Act },
    See { text: Box<[u8]> },
}
pub struct MarkWait { pub mark: u32, pub text: Box<[u8]> }
pub struct Setup {
    pub after: Option<Box<[u8]>>,
    pub steps: Vec<SetupStep>,
    pub reload: Box<[u8]>,
    pub marks: Vec<MarkWait>,
}
pub fn setup(file: &[u8]) -> Result<Setup, Unreadable>;

// temper/tools/journeys/src/again.rs
/// The journey's setup, its `after` joined, or None.
pub fn setup(journeys: &Path, name: &str) -> Result<Option<journey::Setup>, Refusal>;
const HORIZON: Duration = Duration::from_secs(600);
```

Refusals join `lib.rs`'s `Refusal`, printed in `main.rs`'s arm; the walker gains phases for a setup's step, a `see` and the load. `main.rs`'s `record_again` reads the setup with `again::starting`, before anything builds.

- **The load:** `Next::Do(Doing::Go { address })` with the `reload`'s path: a whole page's load on the box's origin, the page's closing warning accepted by the face; `Phase::Opening` then waits for the new page's client (and may first have to see the old page gone). Say which you did.
- **Polling:** a step's `Found::Pending` and `Found::Absent` wait with `Next::Wait { until: Some(now + 50 ms) }`, not a spin.
- **The recorder** saves by itself on a departure in the set up (3.1), so a download may arrive with no key; the walk takes the newest after its own key, as it does.

Readers and neighbours, and what's allowed there:

- `tools/journeys/tests/suite/**`: a reader of the listing or the usage line, its lines alone.
- `.config/fixture-readers.toml` or `read-by-tests.txt`, where a gate step says a new committed file needs a line.
- `temper/docs/design/client/journeys/*.css`, `journey.js`: regenerated by the reprint only.
- Any other journey's pages, by the reprint only, and only if a change of yours shows there: then ask why first.

Options, each decided:

- **A console line the harness refuses** during the set up (an aborted fetch at a load): add it to `Setup.expected` in `again.rs`, with a comment naming it, and say so.
- **A key mark caught a moving page** (`Unwalkable::Moving`): retried up to three times, each on a fresh box. If all three fail, add a `mark` line, or widen `Step::Mark`'s gap in `Walker`'s `Download` arm, and say so.
- **`project-landing`'s sign-in:** "Continue with ChatGPT" departs and comes back on `main` today. `sign-in-new-tab` is landing a new tab for it: if your base has it, the press is the same and the kit's tabs stay behind (`skein-fake-person-browser`); a further record in 8.2's table only if a tab truly needs a step, and say so. The report's first item then is that design line, which the answer call or a later task writes.
- **A walk that no longer holds** (`Refusal::WalkDiffers` or `NoControl`, because the client moved on since the recording): read why first. A control that changed its name holds, since the walk reads events. An act whose event is truly gone is the one `Question:` here, since walking it anew by hand is what this plan ends.
- **Fake-box ports:** the pages may show new ones (`tier-threads` moves a fake box to the kernel's). That is the new recording's noise (section 8).
- **A secret in a setup:** there is none on a fake box; if a step seems to need one, stop and ask.

`chat-reading-c/reading-close` also changes `journeys/**`: a catch-up is regenerated, never hand-merged.

## Steps

1. `journey::setup` and its tests, red first.
2. `again`'s walk and refusals; the join; the tests of 6, *A setup, natively*.
3. The two setups, from the READMEs, tried on a fresh box: `again project-landing` first.
4. `again chat-draft`: read each failure's log and its kept scratch directory. Add `mark` lines until the pages read as the README says.
5. Each new test that pins a rule bites: break it by hand, see it fail, restore.
6. The READMEs; the status line. `cargo fmt`, `old_forms.py` on temper, `heavy cargo clippy -p temper-client-shell-web -p temper-client-journeys --all-targets -- -D warnings`, `heavy cargo nextest run -p temper-client-shell-web -p temper-client-journeys`, and the browser-target check of the record bundle: `heavy cargo check -p temper-client-shell-web --target wasm32-unknown-unknown --features record,loopback-navigation`.
7. Commit and push.

## Report

The setups' files; the two runs' walls and how many tries each took, with each `mark` line and why; each page you checked; anything that made you depart from the sketch; what a person now does to re-record.
