+++
id = "words-value"
part = "words"
kind = "interface"
needs = ["words-wire"]
files = [
  "temper/crates/temper-engine-domain-forge/src/settings.rs",
  "temper/crates/temper-engine-domain-forge/src/kinds.rs",
  "temper/crates/temper-engine-domain-forge/src/lib.rs",
  "temper/crates/temper-engine-domain-forge/src/tests/settings.rs",
  "temper/crates/temper-engine-domain-forge/src/tests/kinds.rs",
  "temper/crates/temper-protocol-store/src/records/settings.rs",
  "temper/crates/temper-protocol-store/src/tests/settings.rs",
  "temper/crates/temper-engine-smith/src/kinds.rs",
  "temper/docs/design/protocol/configuration.status.md",
  "temper/docs/design/protocol/store.status.md",
]
effort = "high"
minutes = 90
+++

# Kinds 9 and 10 as the forge's values: judged, kept, erased, stored

## Goal

A charter's words and a project's words become values the forge holds: the forge
has kinds 9 and 10, their values, their judgement and their accessors, erases a
project's words with it, counts their bytes, and the store keeps them. Nothing
declares the kinds to jig yet (`words-kind` does), so no change sets one until
then; every reader that asks gets none.

Acceptance, by `temper/docs/design/protocol/configuration.md`, 13.2 (*A setting's
words*, *Judged by the forge*, *Erased with their project*):

1. **The values,** exactly `temper/docs/plan/ui-p3e-words/notes/contracts.md`,
   section 3: `LINE_BYTES`, `HEADLINE_BYTES`, `LEDE_BYTES`, `ProjectWords`,
   `words_fit`; `Kind::CharterWords` (9) and `Kind::ProjectWords` (10) in `number`
   and `of`; `Value::CharterWords(Box<[u8]>)` and
   `Value::ProjectWords(ProjectWords)`; `Refusal::Words` and its words. Each new
   item's comment cites 13.2, as its neighbours do.
2. **The judgement** (`settings.rs`, `judge`), as section 3 of the note has it:
   kind 9 at scope 0 alone, numbers 1 to `bounds.charters`; kind 10 at a
   project's scope alone, number 1 alone; words that don't fit refused `Words`;
   a removal or a reset good; another kind's value `Value`.
3. **The accessors** (`kinds.rs`, beside `project_questions`):
   `charter_words(charter)` and `project_words(project)`, through `setting`, so a
   stock default (kind 9's, given by `charter-lines` later) answers as a change
   does and a removal answers none.
4. **Erased with its project:** `erase_project_settings` takes kind 10 beside 2,
   6 and 7; kind 9 stays, as the deployment's kinds do.
5. **Its size held:** `stored_bytes` (`lib.rs`) counts a held value's words'
   bytes, as it counts a rule's segments.
6. **The store's codec** (`temper/crates/temper-protocol-store/src/records/settings.rs`):
   `Value::CharterWords` as `SettingValue::Line` through the store's
   `CharterWords`, `Value::ProjectWords` as `SettingValue::Words` through its
   `ProjectWords`, and back; `words-wire`'s two `Undecodable` stand-ins go. A
   decoded value that doesn't fit its bound is `Undecodable`, as a rule past its
   bounds is.
7. **Smith's codec** (`temper/crates/temper-engine-smith/src/kinds.rs`): `encode`
   and `size` write and measure the two values through the client wire's
   `CharterWords` and `ProjectWords` records, as they do kind 7's
   `ProjectWays`. `decode` of kinds 9 and 10 is `words-kind`'s: leave its default
   arm.
8. **Compile-forced arms** for the new `Value` and `Kind` variants wherever a
   `match` lists them (the survey found: the forge's `kinds.rs`
   `project_questions`, `setting_charter`, `value_state`, and `lib.rs`; the root's
   `route/kinds.rs` and `route/ways.rs`; the ways crate's `ways.rs`; the
   service's `clients/translation.rs` and `stock/kinds.rs`): each treats kinds 9
   and 10 as kind 7 is treated where kind 7 is not the one asked for (excluded,
   no charter named, the value's size). One that would need another choice of
   behaviour: stop and ask.
9. **Rows,** no new test function:
   - the forge's judgement tables (`temper-engine-domain-forge/src/tests/settings.rs`,
     the project kinds' table about line 400 and the deployment kinds' beside
     it): kind 9 at a project's scope refused `Scope`; kind 9 number 0 and
     number `charters + 1` refused `Number`; a 120-byte line good, a 121-byte
     line, an empty one, one holding `\n` and one not UTF-8 refused `Words`;
     kind 10 at scope 0 refused `Scope`, number 2 refused `Number`, words at
     120 and 480 bytes good, a 481-byte lede refused `Words`; a removal and a
     reset of each good; a `Value::Ways` for kind 9 or 10 refused `Value`;
   - the forge's `project_values` table (`tests/kinds.rs`, about line 738), which
     drives deletion and keep, replace, restore and propose: a kind 10 row; and
     the questions' default test (about line 783), or the nearest table there:
     `project_words` none with none held and the held words once held,
     `charter_words` none with none held and the held line once held;
   - the store's settings round trip (`temper-protocol-store/src/tests/settings.rs`,
     about line 32): a kind 9 row and a kind 10 row at their bounds, beside kind
     7's;
   - smith's round-trip values table and its size test
     (`temper-engine-smith/src/tests/kinds.rs`), if they enumerate `Value`s: a row
     each for the two values, the size the encoding's length.
10. **Bites:** let `judge` take a 121-byte line, see its row fail; drop kind 10
    from `erase_project_settings`, see the deletion row fail; restore each.
11. **Status:** delete the bullets naming `words-value` in
    `temper/docs/design/protocol/configuration.status.md` (13.2) and
    `temper/docs/design/protocol/store.status.md` (2) (`notes/common.md`, *Status
    files*).

## Inputs

- `temper/docs/design/protocol/configuration.md`, 13.2: the kinds' table, *A
  setting's words*, *Judged by the forge*, *Erased with their project*.
- `temper/docs/design/protocol/store.md`, section 2, family 20 `Setting`.
- `temper/docs/plan/ui-p3e-words/notes/contracts.md`, sections 1 to 3.
- `temper/docs/plan/ui-p3e-words/notes/common.md`.
- The code, kind 7 end to end as the model: `temper-engine-domain-forge/src/settings.rs`
  (`Kind`, `ProjectWays`, `Value`, `judge`, `Refusal`), `src/kinds.rs`
  (`erase_project_settings`, `project_questions`, `setting`), `src/lib.rs`
  (`stored_bytes`); `temper-protocol-store/src/records/settings.rs`;
  `temper-engine-smith/src/kinds.rs`.

## Owns and may change

- Owns: the forge's kinds 9 and 10, their values, judgement, accessors, erasure
  and held size; the store's and smith's codec arms; the rows; the two status
  bullets.
- Compile-forced only (acceptance 8): the arm a `match` on `Value` or the forge's
  `Kind` needs, in any crate of temper's tree, each named in the report. Not the
  service's kind declarations, the wire's `settings_kind`, smith's `decode` and
  setting names, the root's reads (all `words-kind`'s), the stock's defaults
  (`charter-lines`) or the ways (`way-headlines`, `project-words-read`).
- If a catch-up meets ui-reaches' kind 8 (`Kind::ProjectClasses`,
  `Value::Classes`) in these files, keep both: 8 is theirs, 9 and 10 are yours.
- **Bounds:** `bounds.charters` is what `judge` already reads for kind 3; if it
  isn't jig's `tasks.charters`, say which it is in the report.

## Steps

1. Read the inputs.
2. The values, judgement, accessors, erasure and size; their rows; the bites.
3. The store's and smith's codecs and their rows; the forced arms, by `heavy cargo
   check -p PKG --tests` over the packages of step 4.
4. Runs: `heavy cargo nextest run -p temper-engine-domain-forge -p
   temper-protocol-store -p temper-engine-smith`; the drift test without writing
   (`notes/common.md`), to show no schema moved; `heavy cargo check --tests -p
   temper-engine-domain -p temper-engine-domain-ways -p temper-service -p
   temper-codecs-world -p temper-client-tools`.
5. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy
   -p temper-engine-domain-forge -p temper-protocol-store -p temper-engine-smith
   --all-targets -- -D warnings`, with every other package acceptance 8 touched.
6. Commit (why, in the message); `python3 -I -B tools/citations/citations.py --base
   origin/main`; push.

## Report

As the implementer's text says, and: the forge's surface as written,
`Refusal::Words`' words, every forced arm by file, the rows and the bites, the
runs' counts.
