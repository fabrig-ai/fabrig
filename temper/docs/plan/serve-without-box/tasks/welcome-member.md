+++
id = "welcome-member"
part = "document"
kind = "feature"
needs = []
files = ["temper/crates/temper-service/src/startup/application.rs", "temper/crates/temper-service/src/startup/members.rs", "temper/crates/temper-service/src/startup.rs", "temper/crates/temper-service/src/settings.rs", "temper/crates/temper-service/src/tests/startup.rs", "temper/crates/temper-service/src/tests/members.rs", "temper/docs/help/reference/configuration.md", "temper/docs/design/protocol/configuration.status.md"]
effort = "medium"
minutes = 30
+++

# `application.welcome`: the document says whether it shows the welcome

## Goal

A start reads `application.welcome` (contracts K1) and carries it to the engine's settings. Nothing
reads the value yet; this task gives later tasks the surface alone.

Acceptance:

- `application.welcome` `true`, `false` and absent each start, `Members.welcome` and
  `settings::Engine.welcome` being `true`, `false` and `false`.
- Any other JSON value (`"yes"`, `1`, `null`) is refused, naming `application.welcome`.
- The member has its row in `MEMBERS` (K1), `the_pages_configuration_are_what_the_tables_render`
  passes on the regenerated `temper/docs/help/reference/configuration.md`, and the members'
  agreement tests in `temper-service/src/tests/members.rs` pass.
- `temper/docs/design/protocol/configuration.status.md` loses its text
  "3: `welcome`: no decoder reads it, and an `application.welcome` is refused as an unknown
  member."

## Inputs

- `temper/docs/plan/serve-without-box/notes/common.md`; `notes/contracts.md`, K1.
- `temper/docs/design/protocol/configuration.md`, 3 (`welcome`) and 10.

## Owns and may change

- The files of the header.
- `settings::Engine` is built in `temper-service/src/startup.rs` (`let mut engine =
  settings::Engine { .. }`). Other literals of it (test fixtures in
  `temper-service/src/tests/fixtures/`, `temper/crates/temper/...`): one line each, `welcome:
  false`, under the implementer's text's compiler-forced rule.
- `the_pages_configuration_are_what_the_tables_render` renders "what init writes" from
  `init_document()`: `welcome` reads as not written until a later task. Take what the renderer
  says. If the renderer has no word for an optional member init does not write, do what it does
  for the nearest such member and say so in the report.
- If a test fixture's document list needs `welcome` to prove the row (the `APPLICATION` literal
  in `tests/members.rs`), add `"welcome":true` there.
- Do not thread the value further than `settings::Engine`: `deployment-read` does.

## Steps

1. In `startup/application.rs`, `parse`, read the member with `object.member(document, b"welcome")`
   before `read::finish`, as `landing` is read, and `jig_protocol_config::read::boolean` on its
   value; absent is `false`.

   ```rust
   pub struct Members { /* .. */ pub demo: Option<settings::Demo>, pub welcome: bool, /* .. */ }
   ```

2. `settings::Engine.welcome: bool`, `welcome: parsed.application.welcome` in `startup.rs`.
3. The `MEMBERS` row (`[Member; 108]`), beside `demo`.
4. A test beside `the_template_shaped_demo_reads_into_its_public_fields` in
   `temper-service/src/tests/startup.rs`, one document per value of the acceptance, named
   `the_welcome_member_reads_as_the_document_says`, and its refusal
   `a_welcome_that_is_no_boolean_is_refused_at_its_path`.
5. Regenerate the reference (`notes/common.md`), and the implementer's checks, packages
   `temper-service` and `temper`; commit and push.

## Report

As the implementer's text asks.
