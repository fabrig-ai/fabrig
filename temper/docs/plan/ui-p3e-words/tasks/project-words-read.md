+++
id = "project-words-read"
part = "how"
kind = "feature"
needs = ["way-headlines", "words-kind"]
files = [
  "temper/crates/temper-engine-domain-ways/src/ways.rs",
  "temper/crates/temper-engine-domain-ways/src/tests.rs",
  "temper/crates/temper-engine-domain/src/route/ways.rs",
  "temper/crates/temper-engine-domain/src/route/ways/tests.rs",
  "temper/crates/temper-service/src/family/ways.rs",
  "temper/crates/temper-service/src/tests/family.rs",
  "temper/crates/temper-engine-smith/src/tests/see.rs",
  "temper/tests/engine/domain/tests/suite/ways.rs",
  "temper/docs/design/protocol/configuration.status.md",
  "temper/docs/design/client/README.status.md",
]
effort = "high"
minutes = 75
+++

# The `ways` read carries the project's words

## Goal

The forge's `ways` document carries the project's words, kind 10, beside its
own path, so that How work runs heads the page with them while the project runs
no stock way, and reads nothing new. This is the meeting point of this plan's two
sides: an owner's change of kind 10, committed, reaches the document the client
draws its heading from.

Acceptance, by `temper/docs/design/protocol/configuration.md`, 13.10 (*The
forge's `ways` read*) and `temper/docs/design/client/README.md`, 4.6
(`WaysPanel`'s `words`):

1. **The document** (`temper/crates/temper-engine-domain-ways/src/ways.rs`):
   `WaysDocument` gains `pub words:
   Option<temper_engine_domain_forge::settings::ProjectWords>`, after `own`
   (`temper/docs/plan/ui-p3e-words/notes/contracts.md`, section 5).
2. **Filled:** the root (`temper-engine-domain/src/route/ways.rs`) fills it from
   `domain.forge.project_words(project)`, whatever way the project runs: the
   client chooses whose words it says.
3. **Sent:** the service's `ways_panel` (`temper-service/src/family/ways.rs`)
   sends it as the wire's `WaysPanel.words`, replacing `words-wire`'s `None`.
4. **Rows,** no new test function:
   - the engine world (`temper/tests/engine/domain/tests/suite/ways.rs`,
     `committed_settings_adoptions_and_roles_mark_the_ways_watch`, about line
     231): an owner's committed change of kind 10 for the project marks the
     watch, and the document delivered carries the words; a document before it
     carries none;
   - the root's ways tests (`route/ways/tests.rs`), where a document is built
     from a domain: words none with none held, the held words once held;
   - `tests/family.rs`' `the_worst_small_ways_document_fits_its_decision` (about
     line 3399): `words` some at their bounds, a 120-byte headline and a
     480-byte lede, so the worst document still fits its decision; and the
     panel's encode test there, if one compares the document's fields, its words
     sent.
5. **Bite:** send `None` in `ways_panel`, see the family row that reads the words
   fail; restore.
6. **Status:** delete the bullets naming `project-words-read` in
   `temper/docs/design/protocol/configuration.status.md` (13.10) and
   `temper/docs/design/client/README.status.md` (4.6) (`notes/common.md`, *Status
   files*).

## Inputs

- `temper/docs/design/protocol/configuration.md`, 13.2 (*A project's words*) and
  13.10.
- `temper/docs/design/client/README.md`, 4.6.
- `temper/docs/plan/ui-p3e-words/notes/contracts.md`, sections 2, 3 and 5.
- `temper/docs/plan/ui-p3e-words/notes/common.md`.

## Owns and may change

- Owns: the document's `words`, its filling and sending, the rows and the two
  status bullets.
- Compile-forced, the field alone, in any file of temper's tree that builds a
  `WaysDocument` (the survey found `temper-engine-smith/src/tests/see.rs`): `words:
  None` or its fixture's; name each in the report.
- **If the worst document no longer fits its decision** with the words at their
  bounds: stop with `Question:`, naming the encoded size, the allowance and each
  part's share. Never lower a bound of the design's.
- `see ways` prints no headline or words (`temper/docs/design/domain/agent.md`,
  8.4.1): leave its words as they are.
- The engine world's `ways.rs` is also look-round-25's: add rows of your own,
  change none of theirs.
- Not yours: the drawing (`how-words`), the kinds' reads and names (`words-kind`,
  merged).

## Steps

1. Read the inputs; `WaysDocument`, the root's `ways` document, `ways_panel`, and
   the rows named.
2. The field, its filling and sending; the rows; the bite.
3. Runs: the focused tests of `-p temper-engine-domain-ways`, `-p
   temper-engine-domain`, `-p temper-engine-smith`, `-p temper-service` and `-p
   temper-engine-domain-world`; `heavy cargo check --tests -p temper-client-world
   -p temper-client-gallery`.
4. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p
   temper-engine-domain-ways -p temper-engine-domain -p temper-service
   --all-targets -- -D warnings`.
5. Commit (why, in the message); `python3 -I -B tools/citations/citations.py --base
   origin/main`; push.

## Report

As the implementer's text says, and: the worst document's encoded size against its
allowance; each compile-forced file; the rows and the bite.
