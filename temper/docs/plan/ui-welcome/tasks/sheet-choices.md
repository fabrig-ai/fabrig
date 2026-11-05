+++
id = "sheet-choices"
part = "sheet-choices"
kind = "feature"
needs = ["ready-step", "setup-intent"]
files = [
  "temper/crates/temper-client-domain/src/forge/forges.rs",
  "temper/crates/temper-client-domain/src/forge/making.rs",
  "temper/crates/temper-client-domain/src/forge/family.rs",
  "temper/crates/temper-client-domain/src/tests/connect.rs",
  "temper/crates/temper-client-domain/src/tests/making.rs",
  "temper/crates/temper-client-view/src/forge/connect.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/tests/welcome.rs",
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/tools/gallery/**",
  "temper/docs/design/client/journeys/**",
]
effort = "high"
minutes = 110
+++

# The sheet's other two choices

## Goal

Forgejo's sheet on the welcome sends all three choices: an empty repository, copies, nothing for now.
`WelcomeForgejoNotYet` and `not_yet` go. Request, section 7, *The client*, the sheet's part.

Acceptance:

- **Nothing for now:** Run sends `SetUpFirst::Nothing`, moves on to Model, the set up running on.
- **Copies:** before sending, each non-empty line reads as the mock's `parseCopies`; Run sends `SetUpFirst::Copies`
  with the addresses in `source::address`'s form, in the order written. Alerts, in the sheet: `Not an address: LINE.
  Write it as github.com/owner/name.`, `At least one address, as github.com/owner/name.`, `At most 16 addresses.`
- **Empty:** either field blank: one alert, "Both an organization and a name.", in place of "Required." under each, on
  the welcome's sheet alone. `ConnectForge` keeps "Required."
- "Set up again" asks the same choice. The Model step's pane and Ready's forge line say no repository for them.
- A render beside the mock's sheet at 1440, 1000 and 420, with each choice open.

## Inputs

- `temper/docs/plan/ui-welcome/notes/common.md`, `notes/contracts.md`, *The copies, on the client*.
- `temper/docs/plan/ui-welcome/inputs/request.md`, section 7.
- `temper/docs/design/client/README.md`, 3.5, *Forgejo's sheet*.
- `temper/docs/design/draft/repository-copies.md`, 5.4 and 7.
- The mock: `temper/docs/design/client/greenfield/app.js`, `repoSheet('welcome')`, `repoChosen()`, `parseCopies()`.

## Owns and may change

Owns the files in `files`.

May also change: `temper/crates/temper-client-domain/src/forge/form.rs`, `temper/crates/temper-client-view/src/forge/bindings.rs`,
for the alert; `temper/tests/client/tests/suite/{connect_forge,welcome_m3,welcome_m3_owned}.rs`: a step that reads the
"Not available yet" words. `temper/docs/design/client/README.status.md`: remove `Forgejo's sheet` from "3.5, *Forgejo's
sheet*, *Its copies*".

Options, pre-approved:

- A line past `source::parse` reads as `NotAnAddress`, its words the line as written, trimmed, cut to the word's hole by
  the words' own bound.
- `http://` is stripped as `https://` is, as the mock does, and the design records.

Not this task's: Ready's lead, name and act (`ready-copies`); Home.

## Steps

1. **`forge/making.rs`:** `parse_copies`, by `notes/contracts.md`: trim; strip `https://` or `http://`; strip `.git`,
   then trailing `/`; a line with 2 segments is `github.com/OWNER/NAME`, with 3 is `HOST/OWNER/NAME`; else, or a
   `source::parse` refusal, `NotAnAddress`. Past `w::FIRST_COPIES`, `TooMany`. Table test in `tests/making.rs`, its rows
   from `parseCopies`: bare, `https://`, `http://`, `.git`, trailing slash, `OWNER/NAME`, blank lines dropped, four
   segments, an empty segment, `git@host:o/n`, a host with a port, 17 lines.
2. **`forge/forges.rs`:** `RunOwned` sends by `start`: `Empty` with the fields, blank ones setting `blank`, one alert, on
   the welcome; `Copies` from `parse_copies`, a refusal kept in `Draft.copies_refused`; `Nothing`. `not_yet` goes.
3. **`forge/connect.rs`'s `sheet`:** the alerts above the foot, `Role::Alert`, `sheet-error`'s look. `setup_field` keeps
   "Required." where `Place` isn't the welcome. `WelcomeForgejoNotYet` goes from `words.rs`: enum, list, number, English.
   New words `WelcomeCopiesNotAnAddress`, `WelcomeCopiesNone`, `WelcomeCopiesTooMany`, `WelcomeSheetBlank`.
4. **Tests.** `temper-client-domain`'s `tests/connect.rs`: each Run, its `SetUp` on the wire. `temper-client-view`'s
   `tests/welcome.rs`: each alert, browser and terminal. Gallery: `welcome-sheet-copies`, `welcome-sheet-copies-refused`,
   `welcome-sheet-nothing`, in `names.txt`, `samples.rs`, `tests.rs` and `browser.md`'s 7.2 table.
5. **Check,** `notes/common.md`, and `temper-client-world`; render proof; commit and push.

## Report

The table's rows, the alerts' words, the builder and directory.

## Rebase (2026-11-05, step merge, foreman ui-welcome/sheet-choices/foreman-1)

Rebase sheet choices onto the updated trunk, preserving Goals additions and resolving phrase IDs and counts.
The merge failed with content conflicts in `temper/crates/temper-client-view/src/words.rs` and `temper/tools/gallery/src/samples.rs`.
The overlapping trunk change is `123f73e97b` (“Merge main while preserving welcome and Goals UI”), incorporating main’s `d38be47825` (“Merge ui-goals/integration”).

In `words.rs`, retain the trunk’s Goals vocabulary and existing phrase-ID assignments.
Keep this task’s removal of `WelcomeForgejoNotYet` and its four replacement alert words throughout the enum, `ALL`, ID table and English table.
The task’s IDs 3269–3272 collide with Goals phrases. Assign the four new sheet words unused IDs; on the inspected trunk, 3304–3307 are available.
Combine both capacity changes: `PHRASES = 1463`, `SEGMENTS = 2530`, and `ALL` length 867.
Retain both literal-byte additions: Goals’ 669 and sheet refusals’ 84.
Regenerate phrase declarations with the existing `SKEIN_CODEGEN_WRITE=1` declaration test and verify the resulting counts.

In `samples.rs`, retain all Goals/Home samples, routing and domain setup alongside the three new `welcome-sheet-*` samples and their setup.
Combine the sample counts: `N = 138` on the inspected trunk.
Preserve the automatically merged gallery names, assertions and browser table entries for both sets.

Through heavy, run the focused view declaration and welcome-sheet alert tests, then the gallery tests to verify combined sample names and rendering.
